use std::error::Error;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::sync::Arc;
use std::time::Duration;
use std::{env, mem};

use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, Semaphore};
use tokio::time::interval;

use plugins_commons::model::{BuildQueued, BuildStatus, Packet, Tagged};

use crate::spawner::spawn;
use crate::utils::error::drop_errors_or_default;

/// Listen to RPC on the supplied port.
/// The application binds to `0.0.0.0` as it expects to be protected by upstream firewalls.
/// Such as kubernetes network filters.
pub async fn listen(port: u16) -> Result<(), Box<dyn Error>> {
    let server = TcpListener::bind(SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), port)).await?;

    info!("Listening on 0.0.0.0:{}", port);

    let budget = env::var("BUILD_LIMITS")
        .map(|i| i.parse().unwrap())
        .unwrap_or(16);
    let limiter = Arc::new(Semaphore::new(budget));

    loop {
        let (stream, remote) = server.accept().await?;

        // Drop new connections if 3/4 of the design capacity is occupied
        if limiter.available_permits() <= budget / 4 {
            continue;
        }

        let limiter = Arc::clone(&limiter);
        tokio::spawn(async move {
            info!("Handling incoming socket {}", remote);

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

            let egress_join = tokio::spawn(async move {
                while let Some(packet) = rx.recv().await {
                    if packet.write(&mut egress).await.is_err() {
                        break;
                    }
                }

                info!("Shutting down egress for {:?}", remote);
            });

            let heartbeat_join = {
                let tx = tx.clone();
                tokio::spawn(async move {
                    let mut interval = interval(Duration::from_secs(30));

                    interval.tick().await;
                    while let Ok(()) = tx.send(Packet::Heartbeat) {
                        interval.tick().await;
                    }

                    info!("Shutting down heartbeat task for {:?}", remote)
                })
            };

            let join = tokio::spawn(async move {
                let _ = egress_join.await;
                let _ = heartbeat_join.await;
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
                    let uuid = req.uuid;

                    info!("Received build request {} on {:?}", uuid, remote);

                    let slots_available = limiter.available_permits();

                    let tx = tx.clone();
                    let limiter = Arc::clone(&limiter);
                    tokio::spawn(async move {
                        let mut ack = Packet::Acknowledge(Tagged {
                            uuid,
                            inner: BuildQueued {
                                queued: false,
                                slots_available,
                            },
                        });

                        if let Ok(permit) = limiter.try_acquire() {
                            if let Packet::Acknowledge(ref mut t) = &mut ack {
                                t.inner.queued = true;
                            }
                            let _ = tx.send(ack);

                            tokio::task::spawn_blocking(move || {
                                let mut res = req.fork(BuildStatus::LowLevelError);
                                res.inner = drop_errors_or_default(spawn(req.inner));

                                info!("Completed build request {} on {:?}", uuid, remote);
                                let _ = tx.send(Packet::Response(res));
                            })
                            .await
                            .unwrap();
                            mem::drop(permit);
                        } else {
                            info!("Rejected build request {} on {:?}", uuid, remote);
                            let _ = tx.send(ack);
                        }
                    });
                }
                Packet::Heartbeat => (),
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
