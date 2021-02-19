use std::io::Cursor;

use tokio::io::BufWriter;

use crate::model::packet::{Base64Encoded, BuildStatus, Tagged};

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
async fn tagged_fieldless_deserialization() {
    let tagged = Tagged::new(BuildStatus::LowLevelError);

    assert_eq!(
        serde_json::to_string(&tagged).unwrap(),
        format!(
            "{{\"uuid\":\"{}\",\"type\":\"LowLevelError\"}}",
            tagged.uuid
        )
    );
}
