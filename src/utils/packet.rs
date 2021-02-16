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

/// A packet, transmissible over the wire
#[derive(Serialize, Deserialize)]
pub enum Packet {
	Request(Tagged<BuildContext>),
	Acknowledge(Tagged<()>),
	Response(Tagged<BuildStatus>),
	Arbitrary(Vec<u8>),
}

impl Packet {
	/// Reads a packet from the reader.
	///
	/// Packet structure is as follows:
	///
	/// `Length of packet` | `Packet ID` | `Rest of the packet`
	/// -------------------|-------------|-------------------
	/// 4 bytes            | 1 byt  e    | 1 byte less than `Length of packet`. (The length of the packet accounts for the `Packet ID`)
	///
	/// ## Length of packet
	/// Big-endian, 32 bit, unsigned integer
	///
	/// ## Packet ID
	/// The following is the definitive list of packet IDs:
	///
	/// ID     | Type | Corresponding Enum      | Description
	/// -------|------|-------------------------|------------
	/// `0x00` | Recv | [`Packet::Request`]     | Requesting a build with a JSON build context
	/// `0x01` | Send | [`Packet::Acknowledge`] | Acknowledging a build request (ie. signal the start of the build)
	/// `0x02` | Send | [`Packet::Response`]    | The result of the build
	/// (Rest) | Recv | [`Packet::Arbitrary`]   | A packet with any other packet ID will be interpreted as a byte buffer
	///
	/// ## Rest of the packet
	/// JSON-encoded data structure of the inner types, refer to the corresponding enums above.
	pub async fn read<R: AsyncBufRead + Unpin>(read: &mut R) -> Result<Packet, Box<dyn Error>> {
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

	/// Writes the current packet into the writer.
	/// Refer to [`Packet::read`] for more information about the packet structure
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

/// Encodes the data structure into JSON in a separate, tokio, blocking, task
async fn to_json<S: Serialize + Send + Sync + 'static>(s: S) -> Result<Vec<u8>, Box<dyn Error>> {
	Ok(tokio::task::spawn_blocking(move || serde_json::to_vec(&s)).await??)
}

/// A wrapper over an object that has a UUID
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
	/// Copy the UUID while substituting the inner data structure with the supplied object
	pub fn fork<U>(&self, new_inner: U) -> Tagged<U> {
		Tagged {
			uuid: self.uuid.clone(),
			inner: new_inner,
		}
	}
}

/// Build context. Basically a list of files, base64 encoded
///
/// This blob may be very big due to the nature of files.
/// The external actor is recommended to store this context on S3 or other object storage services.
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

/// Wrapper over a base64 encoded string.
/// Provides helper functions to peek inside the buffer or write it directly into a writer.
#[derive(Serialize, Deserialize)]
pub struct Base64Encoded {
	#[serde(flatten)]
	base64_string: String,
}

impl Base64Encoded {
	/// Read the contained data into a buffer
	pub fn read_to_buffer(&self) -> Result<Vec<u8>, DecodeError> {
		base64::decode(&self.base64_string)
	}

	/// Write the held data directly into a writer
	pub async fn write_to<W: AsyncWrite + Unpin>(&self, writer: &mut W) -> Result<(), Box<dyn Error>> {
		let buf = self.read_to_buffer()?;
		writer.write(&buf).await?;

		Ok(())
	}
}
