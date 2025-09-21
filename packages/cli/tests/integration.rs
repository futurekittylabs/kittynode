use assert_cmd::Command;
use serde_json::Value;

#[test]
fn get_packages_json_contains_ethereum() {
    let mut cmd = Command::cargo_bin("kittynode").unwrap();
    let output = cmd
        .args(["--format", "json", "get-packages"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let value: Value = serde_json::from_slice(&output).unwrap();
    assert!(
        value.get("Ethereum").is_some(),
        "expected Ethereum package to be present"
    );
}

#[test]
fn json_alias_flag_produces_json() {
    let mut cmd = Command::cargo_bin("kittynode").unwrap();
    let output = cmd
        .args(["--json", "get-config"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let value: Value = serde_json::from_slice(&output).unwrap();
    assert!(value.get("server_url").is_some());
}
