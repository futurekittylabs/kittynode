use kittynode_core::Home;
use tempfile::tempdir;

#[test]
fn add_capability_roundtrip_and_no_duplicates() {
    let dir = tempdir().unwrap();
    let home = Home::from_base(dir.path().join(".kittynode"));

    // Initially empty
    let caps = home.get_capabilities().expect("get caps");
    assert!(caps.is_empty());

    home.add_capability("ethereum").expect("add cap");
    home.add_capability("ethereum").expect("add again, no dup");

    let caps = home.get_capabilities().expect("get caps");
    assert_eq!(caps, vec!["ethereum".to_string()]);
}

#[test]
fn remove_capability_updates_config() {
    let dir = tempdir().unwrap();
    let home = Home::from_base(dir.path().join(".kittynode"));

    home.add_capability("eth").unwrap();
    home.add_capability("sol").unwrap();

    home.remove_capability("eth").unwrap();
    let caps = home.get_capabilities().unwrap();
    assert_eq!(caps, vec!["sol".to_string()]);
}

#[test]
fn set_and_get_server_url() {
    let dir = tempdir().unwrap();
    let home = Home::from_base(dir.path().join(".kittynode"));

    home.set_server_url("http://localhost:3000".to_string())
        .unwrap();
    let url = home.get_server_url().unwrap();
    assert_eq!(url, "http://localhost:3000");
}

#[test]
fn init_and_delete_kittynode() {
    use std::fs;

    let dir = tempdir().unwrap();
    let home = Home::from_base(dir.path().join(".kittynode"));

    // Initialize (creates config.toml under KITTYNODE_HOME)
    home.init_kittynode().expect("init ok");
    assert!(home.base().exists(), "home should exist after init");
    assert!(
        home.base().join("config.toml").exists(),
        "config should be created"
    );

    // Create an extra file to ensure recursive delete works
    fs::create_dir_all(home.base().join("extra")).unwrap();
    fs::write(home.base().join("extra/file.txt"), b"data").unwrap();

    // Delete kittynode directory
    home.delete_kittynode().expect("delete ok");
    assert!(!home.base().exists(), "home should be removed");
}

#[test]
fn remove_nonexistent_capability_is_noop() {
    let dir = tempdir().unwrap();
    let home = Home::from_base(dir.path().join(".kittynode"));

    home.add_capability("sol").unwrap();
    home.remove_capability("eth").unwrap();
    let caps = home.get_capabilities().unwrap();
    assert_eq!(caps, vec!["sol".to_string()]);
}

#[test]
fn server_url_empty_roundtrip() {
    let dir = tempdir().unwrap();
    let home = Home::from_base(dir.path().join(".kittynode"));

    home.set_server_url("".to_string()).unwrap();
    let url = home.get_server_url().unwrap();
    assert_eq!(url, "");
}
