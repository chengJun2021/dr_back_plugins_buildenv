use std::io::Cursor;

use tokio::io::{AsyncReadExt, BufReader, BufWriter};

use crate::model::packet::{Base64Encoded, BuildStatus, Packet, Tagged};

mod build_context;
mod build_status;

#[test]
fn base64_read_to_buf() {
    let b64 = Base64Encoded::create("test".as_ref());

    let read = b64.read_to_buffer().unwrap();
    assert_eq!(String::from_utf8(read).unwrap(), "test");
}

#[tokio::test]
async fn base64_write_to_stream() {
    let b64 = Base64Encoded::create("test".as_ref());

    let mut buf = vec![];
    let mut write = BufWriter::new(Cursor::new(&mut buf));

    b64.write_to(&mut write).await.unwrap();
    assert_eq!(String::from_utf8(buf).unwrap(), "test");
}

#[tokio::test]
async fn tagged_blank_serialization() {
    let tagged = Tagged::new(());

    assert_eq!(
        serde_json::to_string(&tagged).unwrap(),
        format!("{{\"uuid\":\"{}\"}}", tagged.uuid)
    );
}

#[tokio::test]
async fn tagged_fieldless_enum_deserialization() {
    let tagged = Tagged::new(BuildStatus::LowLevelError);

    assert_eq!(
        serde_json::to_string(&tagged).unwrap(),
        format!(
            "{{\"uuid\":\"{}\",\"type\":\"LowLevelError\"}}",
            tagged.uuid
        )
    );
}

#[tokio::test]
async fn packet_read_write() {
    let status = Tagged::new(BuildStatus::LowLevelError);
    let uuid = status.uuid;

    let mut bytes_over_wire = vec![];

    // Create a packet using the write function
    {
        let packet = Packet::Response(status);
        packet
            .write(&mut BufWriter::new(&mut Cursor::new(&mut bytes_over_wire)))
            .await
            .unwrap();
    }

    bytes_over_wire.extend_from_slice("Polluting the buffer with extra bytes.".as_bytes());

    // Read a packet using the reference implementation
    {
        let mut read = Cursor::new(&mut bytes_over_wire);

        let len = read.read_u32().await.unwrap() as usize;
        let mut buf = vec![0u8; len];
        let read_len = read.read(&mut buf).await.unwrap();

        assert_eq!(read_len, len);

        assert_eq!(
            String::from_utf8(buf).unwrap(),
            format!("{{\"uuid\":\"{}\",\"type\":\"LowLevelError\"}}", uuid)
        );
    }

    // Read a packet using the reader
    {
        let packet = Packet::read(&mut BufReader::new(&mut Cursor::new(&mut bytes_over_wire)))
            .await
            .unwrap();

        assert_eq!(
            serde_json::to_string(&packet).unwrap(),
            format!("{{\"uuid\":\"{}\",\"type\":\"LowLevelError\"}}", uuid)
        );
    }
}
