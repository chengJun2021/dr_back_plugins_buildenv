extern crate base64;
extern crate uuid;

use std::collections::HashMap;
use std::error::Error;
use std::ops::Deref;
use std::path::Path;

use serde::Serialize;
use tokio::fs;
use tokio::io::{AsyncBufRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use crate::spawner::BuildStatus;

use self::base64::DecodeError;
use self::uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub enum Packet {
	Request(Tagged<BuildContext>),
	Acknowledge(Tagged<()>),
	Response(Tagged<BuildStatus>),
	Arbitrary(Vec<u8>),
}

impl Packet {
	pub async fn read_packet<R: AsyncBufRead + Unpin>(read: &mut R) -> Result<Packet, Box<dyn Error>> {
		let mut len = read.read_u32().await? as usize;

		let packet_type = read.read_u8().await?;
		len -= 1;

		let mut buf = vec![0u8; len];
		read.read_buf(&mut buf).await?;

		return Ok(match packet_type {
			0x00 => Packet::Request(serde_json::from_slice(&buf)?),
			0x01 => Packet::Acknowledge(serde_json::from_slice(&buf)?),
			_ => {
				buf.insert(0, packet_type);

				Packet::Arbitrary(buf)
			}
		});
	}

	pub async fn write<'a, W: AsyncWrite + Unpin>(self, write: &mut W) -> Result<(), Box<dyn Error>> {
		let id = match &self {
			Packet::Request(_) => 0x00,
			Packet::Acknowledge(_) => 0x01,
			Packet::Response(_) => 0x02,
			Packet::Arbitrary(_) => 0xFF,
		};

		let encoded = match self {
			Packet::Request(inner) => to_json(inner).await?,
			Packet::Acknowledge(inner) => to_json(inner).await?,
			Packet::Response(inner) => to_json(inner).await?,
			Packet::Arbitrary(buf) => buf
		};


		write.write_u32(encoded.len() as u32 + 1).await?;
		write.write_u8(id).await?;
		write.write(&encoded).await?;

		write.flush().await?;
		Ok(())
	}
}

async fn to_json<S: Serialize + Send + Sync + 'static>(s: S) -> Result<Vec<u8>, Box<dyn Error>> {
	Ok(tokio::task::spawn_blocking(move || serde_json::to_vec(&s)).await??)
}

#[derive(Serialize, Deserialize)]
pub struct Tagged<T> {
	pub uuid: Uuid,
	#[serde(flatten)]
	pub(crate) inner: T,
}

impl<T> Deref for Tagged<T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		&self.inner
	}
}

impl<T> Tagged<T> {
	pub fn fork<U>(&self, new_inner: U) -> Tagged<U> {
		Tagged {
			uuid: self.uuid.clone(),
			inner: new_inner,
		}
	}
}

#[derive(Serialize, Deserialize)]
pub struct BuildContext {
	files: HashMap<String, Base64Encoded>,
}

impl BuildContext {
	pub async fn extract_into(&self, src_path: &Path) -> Result<bool, Box<dyn Error>> {
		for (location, bytes) in &self.files {
			let dest = src_path.join(location).canonicalize()?;
			if !dest.starts_with(src_path) {
				return Ok(false);
			}

			let mut file = fs::OpenOptions::new()
				.write(true)
				.create_new(true)
				.open(dest).await?;

			bytes.write_to(&mut file).await?;
		}

		Ok(true)
	}
}

#[derive(Serialize, Deserialize)]
pub struct Base64Encoded {
	#[serde(flatten)]
	base64_string: String,
}

impl Base64Encoded {
	pub fn read_to_buffer(&self) -> Result<Vec<u8>, DecodeError> {
		base64::decode(&self.base64_string)
	}

	pub async fn write_to<W: AsyncWrite + Unpin>(&self, writer: &mut W) -> Result<(), Box<dyn Error>> {
		let buf = self.read_to_buffer()?;
		writer.write(&buf).await?;

		Ok(())
	}
}
