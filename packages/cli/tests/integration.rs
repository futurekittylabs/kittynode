use assert_cmd::Command;
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use tempfile::tempdir;

#[test]
fn get_packages_outputs_known_package() {
    let mut cmd = Command::cargo_bin("kittynode").unwrap();
    let output = cmd
        .args(["package", "list"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let stdout = String::from_utf8_lossy(&output);
    assert!(
        stdout.contains("ethereum"),
        "expected CLI output to list ethereum package, got {stdout}"
    );
}

#[test]
fn get_config_outputs_readable_text() {
    let mut cmd = Command::cargo_bin("kittynode").unwrap();
    let output = cmd
        .args(["config", "show"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let stdout = String::from_utf8_lossy(&output);
    assert!(
        stdout.contains("Server URL:"),
        "expected config output to include Server URL, got {stdout}"
    );
}

#[test]
fn web_start_and_stop_roundtrip() {
    let temp_home = tempdir().expect("failed to create temp home directory");
    let sandbox = WebServiceSandbox::new(temp_home.path().to_path_buf());
    let port = find_free_port();

    let start_output = Command::cargo_bin("kittynode")
        .unwrap()
        .env("HOME", sandbox.home())
        .args(["web", "start", "--port", &port.to_string()])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let start_stdout = String::from_utf8_lossy(&start_output);
    assert!(
        start_stdout.contains("started"),
        "expected start output to mention service starting, got {start_stdout}"
    );
    assert!(
        sandbox.state_path().exists(),
        "expected web state file to exist after start"
    );

    let status_running = Command::cargo_bin("kittynode")
        .unwrap()
        .env("HOME", sandbox.home())
        .args(["web", "status"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let status_running_stdout = String::from_utf8_lossy(&status_running);
    assert!(
        status_running_stdout.to_lowercase().contains("running"),
        "expected status output to mention running state, got {status_running_stdout}"
    );

    let stop_output = Command::cargo_bin("kittynode")
        .unwrap()
        .env("HOME", sandbox.home())
        .args(["web", "stop"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let stop_stdout = String::from_utf8_lossy(&stop_output);
    assert!(
        stop_stdout.contains("stopped"),
        "expected stop output to mention service stopping, got {stop_stdout}"
    );
    assert!(
        !sandbox.state_path().exists(),
        "expected web state file to be removed after stop"
    );

    let status_stopped = Command::cargo_bin("kittynode")
        .unwrap()
        .env("HOME", sandbox.home())
        .args(["web", "status"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let status_stopped_stdout = String::from_utf8_lossy(&status_stopped);
    assert!(
        status_stopped_stdout.to_lowercase().contains("not running"),
        "expected status output to mention not running state, got {status_stopped_stdout}"
    );
}

fn find_free_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .expect("failed to bind to probe port")
        .local_addr()
        .expect("failed to read local address")
        .port()
}

struct WebServiceSandbox {
    home: PathBuf,
}

impl WebServiceSandbox {
    fn new(home: PathBuf) -> Self {
        Self { home }
    }

    fn home(&self) -> &Path {
        &self.home
    }

    fn state_path(&self) -> PathBuf {
        self.home
            .join(".config")
            .join("kittynode")
            .join("runtime")
            .join("kittynode-web.json")
    }

    fn stop(&self) {
        if let Ok(mut cmd) = Command::cargo_bin("kittynode") {
            let _ = cmd.env("HOME", self.home()).args(["web", "stop"]).output();
        }
    }
}

impl Drop for WebServiceSandbox {
    fn drop(&mut self) {
        self.stop();
    }
}
