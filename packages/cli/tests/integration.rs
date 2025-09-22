use assert_cmd::Command;

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
        stdout.contains("Ethereum"),
        "expected CLI output to list Ethereum package, got {stdout}"
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
