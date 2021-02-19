use tokio::fs;

use crate::model::packet::BuildContext;

const FILES: &[(&str, &str)] = &[
    ("index.js", "console.log(\"Hello world!\");\n"),
    ("index.scss", "body { content: \"Hello world!\"; }\n"),
];

fn read_build_context() -> BuildContext {
    serde_json::from_str(include_str!("build_context.json")).unwrap()
}

#[test]
fn deserialize() {
    read_build_context();
}

#[tokio::test]
async fn extract() {
    let ctx: BuildContext = read_build_context();

    let td = tempdir::TempDir::new("build-env-test-").unwrap();

    let validated_ok = ctx.extract_into(td.path()).await.unwrap();
    assert!(validated_ok);

    for (file, expected_content) in FILES {
        let item = td.path().join(file);
        let read_content = fs::read_to_string(item).await.unwrap();
        assert_eq!(read_content, *expected_content);
    }
}
