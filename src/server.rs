use std::error::Error;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

use tokio::io::{AsyncBufRead, AsyncWrite, BufStream};
use tokio::net::TcpListener;

use crate::model::packet::{BuildStatus, Packet};
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
            info!("Spawned task for socket {}", remote);

            drop_errors_or_default(process_stream(&mut BufStream::new(stream), remote).await);
        });
    }
}

/// This processes individual sessions on a separate tokio async task
/// RPC requests are executed on a per-socket basis serially.
/// That is, from the perspective of a client attached to a single socket, build requests are processed one at a time.
///
/// Parallel builds can be achieved by creating multiple RPC sessions and feeding through requests in a load-balanced fashion.
async fn process_stream<S>(stream: &mut S, remote: SocketAddr) -> Result<(), Box<dyn Error>>
where
    S: AsyncBufRead + AsyncWrite + Unpin,
{
    loop {
        let packet = Packet::read(stream).await?;

        match packet {
            Packet::Request(req) => {
                warn!("Received build request {} on {:?}", req.uuid, remote);
                Packet::Acknowledge(req.fork(())).write(stream).await?;

                let mut res = req.fork(BuildStatus::LowLevelError);

                res.inner = drop_errors_or_default(spawn(req.inner).await);

                Packet::Response(res).write(stream).await?;
            }
            _ => {
                warn!("Received malformed packet on {:?}", remote);
            }
        }
    }
}
