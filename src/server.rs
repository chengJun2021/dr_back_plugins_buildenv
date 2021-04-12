use std::error::Error;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::sync::Arc;
use std::time::Duration;
use std::{env, mem};

use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{Mutex as AsyncMutex, Semaphore};
use tokio::task::JoinHandle;
use tokio::time::{interval, sleep};

use plugins_commons::model::{BuildContext, BuildQueued, BuildStatus, Packet, Tagged};

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
    let stream = Arc::new(AsyncMutex::new(stream));

    let join = handle_outgoing_tasks(Arc::clone(&stream), remote).await;

    loop {
        let packet = match try_read_packet(&stream, &remote).await {
            Ok(packet) => packet,
            Err(true) => break,
            Err(false) => {
                sleep(Duration::from_millis(50)).await;
                continue;
            }
        };

        match packet {
            Packet::Request(req) => {
                handle_request(Arc::clone(&stream), remote, req, Arc::clone(&limiter)).await;
            }
            Packet::Heartbeat => (),
            _ => {
                warn!("Received context-invalid packet on {:?}", remote);
                break;
            }
        }
    }
    info!("Shutting down ingress for {:?}", remote);

    join.await?;
    Ok(())
}

async fn handle_outgoing_tasks(
    stream: Arc<AsyncMutex<TcpStream>>,
    remote: SocketAddr,
) -> JoinHandle<()> {
    let heartbeat = {
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(30));

            loop {
                interval.tick().await;
                let write = Packet::Heartbeat.write(&mut *stream.lock().await).await;

                if let Err(err) = write {
                    info!(
                        "Shutting down heartbeat task for {:?} Reason: {:?}",
                        remote, err
                    );
                    break;
                }
            }
        })
    };

    return heartbeat;
}

async fn handle_request(
    stream: Arc<AsyncMutex<TcpStream>>,
    remote: SocketAddr,
    req: Tagged<BuildContext>,
    limiter: Arc<Semaphore>,
) {
    let uuid = req.uuid;

    info!("Received build request {} on {:?}", uuid, remote);

    let slots_available = limiter.available_permits();

    tokio::spawn(async move {
        let mut ack = Packet::Acknowledge(Tagged {
            uuid,
            inner: BuildQueued {
                queued: false,
                slots_available,
            },
        });

        let mut task = None;
        if let Ok(permit) = limiter.try_acquire() {
            if let Packet::Acknowledge(ref mut t) = &mut ack {
                t.inner.queued = true;
            }

            task = Some(async {
                let resp = tokio::task::spawn_blocking(move || {
                    let mut res = req.fork(BuildStatus::LowLevelError);
                    res.inner = drop_errors_or_default(spawn(req.inner));

                    info!("Completed build request {} on {:?}", uuid, remote);
                    Packet::Response(res)
                })
                .await
                .unwrap();

                mem::drop(permit);
                drop_errors_or_default(resp.write(&mut *stream.lock().await).await);
            })
        } else {
            info!("Rejected build request {} on {:?}", uuid, remote);
        };

        {
            drop_errors_or_default(ack.write(&mut *stream.lock().await).await);
        }

        if let Some(task) = task {
            task.await;
        }
    });
}

async fn try_read_packet(
    stream: &Arc<AsyncMutex<TcpStream>>,
    remote: &SocketAddr,
) -> Result<Packet, bool> {
    let try_read = {
        let mut stream = stream.lock().await;
        Packet::try_read(&mut *stream).await
    };

    match try_read {
        Ok(Some(packet)) => Ok(packet),
        Ok(None) => Err(false),
        Err(err) => {
            warn!(
                "Received malformed packet on {:?} Reason: {:?}",
                remote, err
            );
            Err(true)
        }
    }
}
