use crate::model::packet::{Base64Encoded, BuildStatus, WebpackOutputs};

#[test]
fn serialize() {
    let status = BuildStatus::WebpackExit {
        code: 1,
        webpack_outputs: WebpackOutputs {
            stdout: Base64Encoded::create("test".as_bytes()),
            stderr: Base64Encoded::create("test".as_bytes()),
        },
    };

    let serialized = serde_json::to_string(&status).unwrap();
    assert_eq!(serialized, "{\"type\":\"WebpackExit\",\"code\":1,\"webpackOutputs\":{\"stdout\":\"dGVzdA==\",\"stderr\":\"dGVzdA==\"}}");
}

#[test]
fn serialize_default() {
    let status = BuildStatus::default();

    let serialized = serde_json::to_string(&status).unwrap();
    assert_eq!(serialized, "{\"type\":\"LowLevelError\"}");
}
