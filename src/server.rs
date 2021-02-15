use std::error::Error;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

use tokio::io::{AsyncBufRead, AsyncWrite, BufStream};
use tokio::net::TcpListener;

use crate::spawner::spawn;
use crate::utils::error::drop_errors_or_default;
use crate::utils::packet::Packet;

pub async fn listen(port: u16) -> Result<(), Box<dyn Error>> {
	let server = TcpListener::bind(SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), port)).await?;

	loop {
		let (stream, remote) = server.accept().await?;

		tokio::spawn(async move {
			info!("Spawned task for socket {}", remote);

			drop_errors_or_default(process_stream(&mut BufStream::new(stream), remote).await);
		});
	}
}

async fn process_stream<S>(stream: &mut S, remote: SocketAddr) -> Result<(), Box<dyn Error>>
	where S: AsyncBufRead + AsyncWrite + Unpin {
	loop {
		let packet = Packet::read_packet(stream).await?;

		match packet {
			Packet::Request(req) => {
				warn!("Received build request {} on {:?}", req.uuid, remote);
				Packet::Acknowledge(req.fork(())).write(stream).await?;

				let status = spawn(req.inner).await?;
			}
			_ => {
				warn!("Received malformed packet on {:?}", remote);
			}
		}
	}
}