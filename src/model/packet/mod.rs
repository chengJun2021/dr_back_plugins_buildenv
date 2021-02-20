extern crate base64;
extern crate uuid;

use std::error::Error;
use std::ops::Deref;

use serde::Serialize;
use tokio::io::{AsyncBufRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

pub use build_context::*;
pub use build_queued::*;
pub use build_status::*;

use self::base64::DecodeError;
use self::uuid::Uuid;

/// Contents of a packet containing information about the build context
mod build_context;
/// Contents of a packet containing information about the position of the request in the queue, if it's queued
mod build_queued;
/// Contents of a packet containing information about the status of the build
mod build_status;

/// A packet, transmissible over the wire
///
/// # Packet Structure
/// Length of packet | Rest of the packet
/// -----------------|-------------------
/// 4 bytes          | `Length` bytes
///
/// ## Length of packet
/// Big-endian, 32 bit, unsigned integer
///
/// ## Rest of the packet
/// JSON-encoded data structure of the inner types, refer to the corresponding enums above.
///
/// # Packet Types
/// The following is the definitive list of packets:
///
/// Directionality | Corresponding Enum      | Description
/// ---------------|-------------------------|------------
/// Recv           | [`Packet::Request`]     | Requesting a build with a JSON build context
/// Send           | [`Packet::Acknowledge`] | Acknowledging a build request (ie. signal the start of the build)
/// Send           | [`Packet::Response`]    | The result of the build, including artefacts if successful
///
#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum Packet {
    /// RPC Client requesting a build with a JSON build context
    Request(Tagged<BuildContext>),
    /// RPC Server finishes the build and reports the result thereof
    Response(Tagged<BuildStatus>),
    /// RPC Server acknowledges the build request and starts the build
    Acknowledge(Tagged<BuildQueued>),
}

impl Packet {
    /// Reads a packet from the reader.
    pub async fn read<R: AsyncBufRead + Unpin>(read: &mut R) -> Result<Packet, Box<dyn Error>> {
        let len = read.read_u32().await? as usize;

        let mut buf = vec![0u8; len];
        let mut read_bytes = 0;

        while read_bytes < len {
            read_bytes += read.read(&mut buf[read_bytes..len]).await?;
        }

        return Ok(serde_json::from_slice(&buf)?);
    }

    /// Writes the current packet into the writer.
    pub async fn write<'a, W: AsyncWrite + Unpin>(
        self,
        write: &mut W,
    ) -> Result<(), Box<dyn Error>> {
        let encoded = to_json(self).await?;

        write.write_u32(encoded.len() as u32).await?;
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

    /// Wrap an existing object with a brand new UUIDv4
    #[cfg(test)]
    pub fn new(inner: T) -> Self {
        Tagged {
            uuid: Uuid::new_v4(),
            inner,
        }
    }
}

/// Wrapper over a base64 encoded string.
/// Provides helper functions to peek inside the buffer or write it directly into a writer.
#[derive(Serialize, Deserialize)]
pub struct Base64Encoded(pub String);

impl Base64Encoded {
    /// Read the contained data into a buffer
    pub fn read_to_buffer(&self) -> Result<Vec<u8>, DecodeError> {
        base64::decode(&self.0)
    }

    /// Write the held data directly into a writer
    pub async fn write_to<W: AsyncWrite + Unpin>(
        &self,
        writer: &mut W,
    ) -> Result<(), Box<dyn Error>> {
        let buf = self.read_to_buffer()?;
        writer.write(&buf).await?;
        writer.flush().await?;

        Ok(())
    }

    /// Creates a base64-encoded string based on the buffer
    pub fn create(buf: &[u8]) -> Self {
        Base64Encoded(base64::encode(buf))
    }
}
