use crate::domain::package::{InstallStatus, Package, PackageDefinition};
use crate::infra::{file::generate_jwt_secret, package, package_config::PackageConfigStore};
use crate::manifests::ethereum::{self, Ethereum};
use eyre::{Context, Result, eyre};
use tracing::{info, warn};

fn ensure_jwt_secret_if_needed(name: &str, package: &Package) -> Result<()> {
    if package.containers.is_empty() {
        return Ok(());
    }
    generate_jwt_secret(name).wrap_err("Failed to generate JWT secret")?;
    Ok(())
}

pub async fn install_package(name: &str) -> Result<()> {
    install_package_with_network(name, None).await
}

/// Installs a package, optionally selecting a network (e.g., for Ethereum).
///
/// If the package is partially installed, logs the precise reason (missing config and/or
/// missing containers) before reinstalling.
pub async fn install_package_with_network(name: &str, network: Option<&str>) -> Result<()> {
    if let Some(network) = network {
        if name != Ethereum::NAME {
            return Err(eyre!(
                "Package '{name}' does not support selecting a network"
            ));
        }

        if !ethereum::is_supported_network(network) {
            let supported = ethereum::supported_networks_display(", ");
            return Err(eyre!(
                "Unsupported Ethereum network: {network}. Supported values: {supported}"
            ));
        }

        let mut config = PackageConfigStore::load(name)
            .wrap_err_with(|| format!("Failed to load configuration for {name}"))?;
        config
            .values
            .insert("network".to_string(), network.to_string());
        PackageConfigStore::save(name, &config)
            .wrap_err_with(|| format!("Failed to persist configuration for {name}"))?;
    }

    let package = package::get_package_by_name(name)?;

    let state = package::get_package(&package).await?;
    match state.install {
        InstallStatus::Installed => {
            info!("Package '{name}' already installed; skipping reinstall");
            return Ok(());
        }
        InstallStatus::PartiallyInstalled => {
            let mut details = Vec::new();
            if !state.config_present {
                details.push("configuration missing".to_string());
            }
            if !state.missing_containers.is_empty() {
                details.push(format!(
                    "missing containers: {}",
                    state.missing_containers.join(", ")
                ));
            }
            let note = if details.is_empty() {
                String::from("partial state detected")
            } else {
                details.join(", ")
            };
            warn!(
                "Package '{name}' is partially installed ({note}). Cleaning up before reinstalling"
            );
            package::delete_package(&package, false, false)
                .await
                .wrap_err_with(|| format!("Failed to clean up partial installation for {name}"))?;
        }
        InstallStatus::NotInstalled => {}
    }

    ensure_jwt_secret_if_needed(name, &package)?;

    package::install_package(&package).await?;
    info!("Package '{name}' installed successfully");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::ensure_jwt_secret_if_needed;
    use crate::domain::container::Container;
    use crate::domain::package::{Package, PackageConfig};
    use std::{
        env,
        ffi::OsString,
        sync::{Mutex, MutexGuard},
    };

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    struct TempHomeGuard {
        _lock: MutexGuard<'static, ()>,
        _temp: tempfile::TempDir,
        prev_home: Option<OsString>,
    }

    impl TempHomeGuard {
        fn new() -> Self {
            let lock = ENV_LOCK
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
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

    fn jwt_path(package_name: &str) -> std::path::PathBuf {
        let home = env::var_os("HOME").expect("HOME set");
        std::path::PathBuf::from(home)
            .join(".config")
            .join("kittynode")
            .join("packages")
            .join(package_name)
            .join("jwt.hex")
    }

    #[test]
    fn ensure_jwt_secret_if_needed_creates_secret_for_packages_with_containers() {
        let _home = TempHomeGuard::new();
        let package_name = "test-package";
        let pkg = Package {
            name: package_name.to_string(),
            description: "test".to_string(),
            network_name: "test".to_string(),
            containers: vec![Container {
                name: "dummy".to_string(),
                image: "dummy".to_string(),
                cmd: Vec::new(),
                port_bindings: std::collections::HashMap::new(),
                volume_bindings: Vec::new(),
                file_bindings: Vec::new(),
            }],
            default_config: PackageConfig::default(),
        };

        ensure_jwt_secret_if_needed(package_name, &pkg).expect("should create secret");

        let path = jwt_path(package_name);
        assert!(path.exists(), "jwt secret file should exist");
        let secret = std::fs::read_to_string(path).expect("secret readable");
        assert_eq!(secret.len(), 64);
        assert!(secret.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn ensure_jwt_secret_if_needed_skips_packages_without_containers() {
        let _home = TempHomeGuard::new();
        let package_name = "test-package";
        let pkg = Package {
            name: package_name.to_string(),
            description: "test".to_string(),
            network_name: "test".to_string(),
            containers: Vec::new(),
            default_config: PackageConfig::default(),
        };

        ensure_jwt_secret_if_needed(package_name, &pkg).expect("should succeed");

        let path = jwt_path(package_name);
        assert!(!path.exists(), "jwt secret file should not exist");
    }
}
