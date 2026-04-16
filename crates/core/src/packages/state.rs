use crate::docker::{container_is_running, find_container, get_docker_instance};
use crate::packages::catalog::get_package_catalog;
use crate::packages::{InstallStatus, Package, PackageConfigStore, PackageState, RuntimeStatus};
use crate::paths::kittynode_path;
use bollard::Docker;
use eyre::{Context, Result};
use std::path::Path;
use tracing::warn;

pub(crate) async fn get_package(name: &str) -> Result<PackageState> {
    let package = crate::packages::catalog::get_package_by_name(name)?;
    get_concrete_package(&package).await
}

pub(crate) async fn get_packages(
    names: &[&str],
) -> Result<std::collections::HashMap<String, PackageState>> {
    let catalog = get_package_catalog()?;
    let mut states = std::collections::HashMap::new();

    for name in names {
        let Some(package) = catalog.get(*name) else {
            warn!("Package '{name}' not found in catalog, skipping");
            continue;
        };

        let state = get_concrete_package(package).await?;
        states.insert((*name).to_string(), state);
    }

    Ok(states)
}

pub(crate) async fn get_installed_packages() -> Result<Vec<Package>> {
    let packages = get_package_catalog().wrap_err("Failed to retrieve packages")?;
    let docker = get_docker_instance().await?;
    let mut installed = Vec::new();

    for package in packages.values() {
        let state = get_package_with(&docker, package).await?;
        if matches!(state.install, InstallStatus::Installed) {
            installed.push(package.clone());
        }
    }

    Ok(installed)
}

pub(crate) async fn get_concrete_package(package: &Package) -> Result<PackageState> {
    if package.containers.is_empty() {
        return get_package_without_docker(package);
    }

    let docker = get_docker_instance().await?;
    get_package_with(&docker, package).await
}

async fn get_package_with(docker: &Docker, package: &Package) -> Result<PackageState> {
    let base = kittynode_path()?;
    let config_path = PackageConfigStore::config_file_path(&base, package.name());
    let config_present = config_path.exists();

    let total = package.containers.len();
    let mut missing = Vec::new();
    let mut running_count = 0usize;

    for container in &package.containers {
        let summaries = find_container(docker, &container.name).await?;
        if summaries.is_empty() {
            missing.push(container.name.clone());
            continue;
        }
        if summaries.iter().any(container_is_running) {
            running_count += 1;
        }
    }

    let install = if total == 0 {
        if config_present {
            InstallStatus::PartiallyInstalled
        } else {
            InstallStatus::NotInstalled
        }
    } else if config_present && missing.is_empty() {
        InstallStatus::Installed
    } else if !config_present && missing.len() == total {
        InstallStatus::NotInstalled
    } else {
        InstallStatus::PartiallyInstalled
    };

    let runtime = if total == 0 || running_count == 0 {
        RuntimeStatus::NotRunning
    } else if running_count == total {
        RuntimeStatus::Running
    } else {
        RuntimeStatus::PartiallyRunning
    };

    Ok(PackageState {
        install,
        runtime,
        config_present,
        missing_containers: missing,
    })
}

fn get_package_without_docker(package: &Package) -> Result<PackageState> {
    let base = kittynode_path()?;
    get_package_without_docker_at(&base, package)
}

fn get_package_without_docker_at(base: &Path, package: &Package) -> Result<PackageState> {
    let config_path = PackageConfigStore::config_file_path(&base, package.name());
    let config_present = config_path.exists();

    let install = if config_present {
        InstallStatus::PartiallyInstalled
    } else {
        InstallStatus::NotInstalled
    };

    Ok(PackageState {
        install,
        runtime: RuntimeStatus::NotRunning,
        config_present,
        missing_containers: Vec::new(),
    })
}

#[cfg(test)]
mod tests {
    use super::get_package_without_docker_at;
    use crate::packages::{
        InstallStatus, Package, PackageConfig, PackageConfigStore, RuntimeStatus,
    };
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn get_package_without_docker_reports_not_installed_when_config_missing() {
        let temp_dir = tempdir().expect("tempdir");
        let base_dir = temp_dir.path().join(".config").join("kittynode");

        let package = Package {
            name: "ethereum".to_string(),
            description: "test".to_string(),
            network_name: "test".to_string(),
            containers: Vec::new(),
            default_config: PackageConfig::default(),
        };

        let state =
            get_package_without_docker_at(&base_dir, &package).expect("state should compute");
        assert!(state.install == InstallStatus::NotInstalled);
        assert!(state.runtime == RuntimeStatus::NotRunning);
        assert!(!state.config_present);
        assert!(state.missing_containers.is_empty());
    }

    #[test]
    fn get_package_without_docker_reports_partial_install_when_config_present() {
        let temp_dir = tempdir().expect("tempdir");
        let base_dir = temp_dir.path().join(".config").join("kittynode");
        let config_path = PackageConfigStore::config_file_path(&base_dir, "ethereum");
        fs::create_dir_all(config_path.parent().expect("parent")).expect("create dirs");
        fs::write(&config_path, "values = {}\n").expect("write config");

        let package = Package {
            name: "ethereum".to_string(),
            description: "test".to_string(),
            network_name: "test".to_string(),
            containers: Vec::new(),
            default_config: PackageConfig::default(),
        };

        let state =
            get_package_without_docker_at(&base_dir, &package).expect("state should compute");
        assert!(state.install == InstallStatus::PartiallyInstalled);
        assert!(state.runtime == RuntimeStatus::NotRunning);
        assert!(state.config_present);
        assert!(state.missing_containers.is_empty());
    }
}
