use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::{fs, process::Command};
use tempfile::tempdir;

#[test]
fn file_value_is_used_when_no_env_or_cli() {
    let dir = tempdir().expect("failed to create temp dir for test");
    let toml = r#"
            [http]
            bind = "127.0.0.1:9999"
        "#;
    fs::write(dir.path().join("gateway.toml"), toml).unwrap();

    let mut cmd = Command::cargo_bin("gateway").unwrap();
    cmd.current_dir(dir.path()).arg("--print-bind");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("127.0.0.1:9999"));
}

#[test]
fn env_override_wins_over_file() {
    let dir = tempdir().expect("failed to create temp dir for test");
    let toml = r#"
            [http]
            bind = "127.0.0.1:9999"
        "#;
    fs::write(dir.path().join("gateway.toml"), toml).unwrap();

    let mut cmd = Command::cargo_bin("gateway").unwrap();
    cmd.current_dir(dir.path())
        .env("GATEWAY__HTTP__BIND", "127.0.0.1:7000")
        .arg("--print-bind");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("127.0.0.1:7000"));
}

#[test]
fn cli_override_wins_over_env_and_file() {
    let dir = tempdir().expect("failed to create temp dir for test");
    let toml = r#"
            [http]
            bind = "127.0.0.1:9999"
        "#;
    fs::write(dir.path().join("gateway.toml"), toml).unwrap();

    let mut cmd = Command::cargo_bin("gateway").unwrap();
    cmd.current_dir(dir.path())
        .env("GATEWAY__HTTP__BIND", "127.0.0.1:7000")
        .arg("--http-bind")
        .arg("127.0.0.1:6000")
        .arg("--print-bind");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("127.0.0.1:6000"));
}
