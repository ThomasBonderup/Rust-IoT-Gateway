#![cfg(unix)]

use assert_cmd::prelude::*;
use nix::sys::signal::{Signal, kill};
use nix::unistd::Pid;
use reqwest::blocking::Client;
use std::io::{BufRead, BufReader};
use std::os::unix::process::ExitStatusExt;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};
use tempfile::tempdir;
use wait_timeout::ChildExt; // brings .wait_timeout into scope

fn wait_for_status(url: &str, want: u16, timeout: Duration) -> bool {
    let client = Client::new();
    let start = Instant::now();
    while start.elapsed() < timeout {
        if let Ok(resp) = client.get(url).send() {
            if resp.status().as_u16() == want {
                return true;
            }
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    false
}

#[test]
fn flips_readyz_to_503_on_sigterm() {
    // 1) temp dir with config
    let dir = tempdir().unwrap();
    std::fs::write(
        dir.path().join("gateway.toml"),
        r#"[http]
bind = "127.0.0.1:0"

[storage]
db_path = "./data.db"
min_free_bytes = 1000
"#,
    )
    .unwrap();

    // 2) spawn the binary; capture stdout to read "listening on …"
    let mut cmd = Command::cargo_bin("gateway").unwrap();
    cmd.current_dir(dir.path())
        .env_remove("RUST_LOG") // keep output predictable
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit());

    let mut child = cmd.spawn().expect("failed to spawn gateway");
    let stdout = child.stdout.take().expect("no stdout captured");
    let mut reader = BufReader::new(stdout);

    // 3) read lines until we see the bound addr
    let mut line = String::new();
    let start = Instant::now();
    let addr = loop {
        line.clear();
        if reader.read_line(&mut line).unwrap_or(0) == 0 {
            // no line yet; give it a moment
            if start.elapsed() > Duration::from_secs(5) {
                panic!("timed out waiting for 'listening on …'");
            }
            std::thread::sleep(Duration::from_millis(20));
            continue;
        }
        if let Some(rest) = line.trim().strip_prefix("listening on ") {
            break rest.to_string();
        }
        if start.elapsed() > Duration::from_secs(5) {
            panic!("did not see 'listening on …'; last line: {line}");
        }
    };

    let readyz = format!("http://{}/readyz", addr);

    // 4) wait until /readyz is 200
    assert!(
        wait_for_status(&readyz, 200, Duration::from_secs(5)),
        "readyz never became 200 OK"
    );

    // 5) send SIGTERM
    let pid = child.id();
    kill(Pid::from_raw(pid as i32), Signal::SIGTERM).expect("failed to send SIGTERM");

    // 6) expect /readyz to flip to 503 during graceful drain
    assert!(
        wait_for_status(&readyz, 503, Duration::from_secs(5)),
        "readyz did not flip to 503 after SIGTERM"
    );

    // 7) process should exit shortly after
    match child
        .wait_timeout(Duration::from_secs(5))
        .expect("wait_timeout failed")
    {
        Some(status) => assert!(
            status.success() || status.signal().is_some(),
            "unexpected exit status: {status:?}"
        ),
        None => {
            // Still running → kill and fail
            let _ = child.kill();
            panic!("process did not exit within timeout after SIGTERM");
        }
    }
}
