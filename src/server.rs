use std::error::Error;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::sync::Arc;
use std::{env, mem};

use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, Semaphore};

use plugins_commons::model::{BuildQueued, BuildStatus, Packet};

use crate::spawner::spawn;
use crate::utils::error::drop_errors_or_default;

/// Listen to RPC on the supplied port.
/// The application binds to `0.0.0.0` as it expects to be protected by upstream firewalls.
/// Such as kubernetes network filters.
pub async fn listen(port: u16) -> Result<(), Box<dyn Error>> {
    let server = TcpListener::bind(SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), port)).await?;

    info!("Listening on 0.0.0.0:{}", port);

    let mut limiter = Arc::new(Semaphore::new(
        env::var("PORT_LIMITS")
            .map(|i| i.parse().unwrap())
            .unwrap_or(8),
    ));

    loop {
        let (stream, remote) = server.accept().await?;

        tokio::spawn(async move {
            info!("Handling incoming socket {}", remote);

            let limiter = Arc::clone(&limiter);
            drop_errors_or_default(process_stream(stream, remote, limiter).await);
        });
    }
}

/// This processes individual sessions on a separate tokio async task
/// RPC requests are executed in parallel.
///
/// Each stream can introduce parallel tasks and they may be returned in arbitrary order.
/// There is also a potential of getting tasks rejected when the pod is overloaded.
async fn process_stream(
    stream: TcpStream,
    remote: SocketAddr,
    limiter: Arc<Semaphore>,
) -> Result<(), Box<dyn Error>> {
    let (mut ingress, mut egress) = stream.into_split();

    let join = {
        let (join, tx) = {
            let (tx, mut rx) = mpsc::unbounded_channel::<Packet>();

            let join = tokio::spawn(async move {
                while let Some(packet) = rx.recv().await {
                    if packet.write(&mut egress).await.is_err() {
                        break;
                    }
                }

                info!("Shutting down egress for {:?}", remote);
            });

            (join, tx)
        };

        loop {
            let packet = if let Ok(packet) = Packet::read(&mut ingress).await {
                packet
            } else {
                warn!("Received malformed packet on {:?}", remote);

                break;
            };

            match packet {
                Packet::Request(req) => {
                    info!("Received build request {} on {:?}", req.uuid, remote);
                    let queued = if let Ok(permit) = limiter.try_acquire() {
                        let tx = tx.clone();
                        tokio::task::spawn_blocking(move || {
                            let mut res = req.fork(BuildStatus::LowLevelError);

                            res.inner = drop_errors_or_default(spawn(req.inner));

                            mem::drop(permit);
                            let _ = tx.send(Packet::Response(res));
                        });

                        true
                    } else {
                        false
                    };

                    let ack = Packet::Acknowledge(req.fork(BuildQueued {
                        queued,
                        slots_available: limiter.available_permits(),
                    }));
                    if tx.send(ack).is_err() {
                        break;
                    }
                }
                _ => {
                    warn!("Received context-invalid packet on {:?}", remote);
                    break;
                }
            }
        }
        info!("Shutting down ingress for {:?}", remote);

        join
    };

    join.await?;
    Ok(())
}
