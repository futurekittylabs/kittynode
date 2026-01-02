use kittynode_core::api;
use std::{
    env,
    ffi::OsString,
    path::{Path, PathBuf},
    sync::{Mutex, MutexGuard},
};
use tempfile::TempDir;

static ENV_LOCK: Mutex<()> = Mutex::new(());

struct TempHomeGuard {
    _lock: MutexGuard<'static, ()>,
    _temp: TempDir,
    prev_home: Option<OsString>,
}

impl TempHomeGuard {
    fn new() -> Self {
        let lock = ENV_LOCK.lock().expect("env lock");
        let temp = tempfile::tempdir().expect("tempdir");
        let prev_home = env::var_os("HOME");
        unsafe {
            env::set_var("HOME", temp.path());
        }

        Self {
            _lock: lock,
            _temp: temp,
            prev_home,
        }
    }

    fn config_root(&self) -> PathBuf {
        self._temp.path().join(".config").join("kittynode")
    }

    fn home(&self) -> &Path {
        self._temp.path()
    }
}

impl Drop for TempHomeGuard {
    fn drop(&mut self) {
        match self.prev_home.take() {
            Some(value) => unsafe {
                env::set_var("HOME", value);
            },
            None => unsafe {
                env::remove_var("HOME");
            },
        }
    }
}

#[test]
fn init_kittynode_preserves_onboarding_completed() {
    let _home = TempHomeGuard::new();

    api::set_onboarding_completed(true).expect("set onboarding");
    api::set_server_url("https://node.example.com".to_string()).expect("set server url");

    api::init_kittynode().expect("init");

    let config = api::get_config().expect("load config");
    assert!(config.onboarding_completed);
    assert_eq!(config.server_url, "");
}

#[test]
fn capabilities_roundtrip_is_persistent_and_deduplicated() {
    let _home = TempHomeGuard::new();

    api::add_capability("ethereum").expect("add");
    api::add_capability("ethereum").expect("add again");
    api::add_capability("solana").expect("add second");

    let caps = api::get_capabilities().expect("get capabilities");
    assert_eq!(caps, vec!["ethereum".to_string(), "solana".to_string()]);

    api::remove_capability("ethereum").expect("remove");
    let caps = api::get_capabilities().expect("get capabilities");
    assert_eq!(caps, vec!["solana".to_string()]);
}

#[test]
fn config_toggles_persist() {
    let _home = TempHomeGuard::new();

    api::set_auto_start_docker(true).expect("toggle docker auto start");
    api::set_show_tray_icon(false).expect("toggle tray icon");

    let config = api::get_config().expect("load config");
    assert!(config.auto_start_docker);
    assert!(!config.show_tray_icon);
}

#[test]
fn delete_kittynode_is_idempotent() {
    let home = TempHomeGuard::new();

    api::init_kittynode().expect("init");
    assert!(
        home.config_root().exists(),
        "config should exist after init"
    );

    api::delete_kittynode().expect("delete");
    assert!(
        !home.config_root().exists(),
        "config dir should be removed after delete"
    );

    api::delete_kittynode().expect("delete again");
    assert!(
        !home.config_root().exists(),
        "delete should stay idempotent"
    );
}

#[test]
fn get_package_catalog_contains_ethereum() {
    let _home = TempHomeGuard::new();

    let catalog = api::get_package_catalog().expect("catalog");
    let eth = catalog.get("ethereum").expect("ethereum entry");
    assert_eq!(eth.name(), "ethereum");
}

#[tokio::test]
async fn install_ethereum_requires_network_before_docker_access() {
    let _home = TempHomeGuard::new();

    let err = api::install_package("ethereum")
        .await
        .expect_err("expected install to fail without network");
    let msg = err.to_string();
    assert!(
        msg.contains("Network must be selected"),
        "unexpected error message: {msg}"
    );
}

#[tokio::test]
async fn install_package_with_unsupported_network_errors_early() {
    let _home = TempHomeGuard::new();

    let err = api::install_package_with_network("ethereum", Some("does-not-exist"))
        .await
        .expect_err("expected validation error");
    assert!(
        err.to_string().contains("Unsupported Ethereum network"),
        "unexpected error: {err}"
    );
}

#[tokio::test]
async fn install_package_with_network_rejects_non_ethereum() {
    let _home = TempHomeGuard::new();

    let err = api::install_package_with_network("not-a-package", Some("mainnet"))
        .await
        .expect_err("expected validation error");
    assert!(
        err.to_string()
            .contains("does not support selecting a network"),
        "unexpected error: {err}"
    );
}

#[tokio::test]
async fn delete_unknown_package_is_not_found() {
    let _home = TempHomeGuard::new();

    let err = api::delete_package("does-not-exist", false)
        .await
        .expect_err("expected error");
    assert!(
        err.to_string().contains("not found"),
        "unexpected error: {err}"
    );
}

#[tokio::test]
async fn delete_unconfigured_ethereum_does_not_require_docker() {
    let home = TempHomeGuard::new();
    let base_dir = home
        .home()
        .join(".config")
        .join("kittynode")
        .join("packages")
        .join("ethereum");
    std::fs::create_dir_all(&base_dir).expect("create package dir");
    std::fs::write(base_dir.join("config.toml"), "values = {}\n").expect("write config");

    api::delete_package("ethereum", false)
        .await
        .expect("delete should succeed without docker");
}
