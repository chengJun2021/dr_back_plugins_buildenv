use std::error::Error;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;

use plugins_commons::model::{BuildQueued, BuildStatus, Packet};

use crate::spawner::spawn;
use crate::utils::error::drop_errors_or_default;

/// Listen to RPC on the supplied port.
/// The application binds to `0.0.0.0` as it expects to be protected by upstream firewalls.
/// Such as kubernetes network filters.
pub async fn listen(port: u16) -> Result<(), Box<dyn Error>> {
    let server = TcpListener::bind(SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), port)).await?;

    info!("Listening on 0.0.0.0:{}", port);

    loop {
        let (stream, remote) = server.accept().await?;

        tokio::spawn(async move {
            info!("Handling incoming socket {}", remote);

            drop_errors_or_default(process_stream(stream, remote).await);
        });
    }
}

/// This processes individual sessions on a separate tokio async task
/// RPC requests are executed on a per-socket basis serially.
/// That is, from the perspective of a client attached to a single socket, build requests are processed one at a time.
///
/// Parallel builds can be achieved by creating multiple RPC sessions and feeding through requests in a load-balanced fashion.
async fn process_stream(stream: TcpStream, remote: SocketAddr) -> Result<(), Box<dyn Error>> {
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
                    let ack = Packet::Acknowledge(req.fork(BuildQueued { queued: true }));
                    if tx.send(ack).is_err() {
                        break;
                    }

                    let tx = tx.clone();
                    tokio::task::spawn_blocking(move || {
                        let mut res = req.fork(BuildStatus::LowLevelError);

                        res.inner = drop_errors_or_default(spawn(req.inner));

                        let _ = tx.send(Packet::Response(res));
                    });
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
