use assert_cmd::Command;
use std::fs;
use std::path::PathBuf;
use tempfile::tempdir;

#[test]
fn cli_generate_json_smoke() {
    let temp = tempdir().expect("tempdir");
    let input_path = temp.path().join("input.json");
    let output_path = temp.path().join("model.rs");

    fs::write(&input_path, r#"{"id": 1, "name": "Alice"}"#).expect("write input");

    let mut cmd = Command::cargo_bin("unistructgen").expect("binary");
    cmd.args([
        "generate",
        "--input",
        input_path.to_str().unwrap(),
        "--name",
        "User",
        "--output",
        output_path.to_str().unwrap(),
    ]);

    cmd.assert().success();

    let output = fs::read_to_string(&output_path).expect("read output");
    assert!(output.contains("pub struct User"));
    assert!(output.contains("pub id: i64"));
}

#[test]
fn cli_client_smoke() {
    let temp = tempdir().expect("tempdir");
    let output_dir = temp.path().join("client");

    let spec_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("proc-macro")
        .join("tests")
        .join("test-api.yaml");

    let mut cmd = Command::cargo_bin("unistructgen").expect("binary");
    cmd.args([
        "client",
        "--spec",
        spec_path.to_str().unwrap(),
        "--output",
        output_dir.to_str().unwrap(),
        "--name",
        "TestApi",
        "--examples",
        "false",
    ]);

    cmd.assert().success();

    let expected_files = [
        "types.rs",
        "client.rs",
        "lib.rs",
        "Cargo.toml",
        "README.md",
    ];

    for file in expected_files {
        let path = output_dir.join(file);
        assert!(path.exists(), "missing {}", file);
    }
}
