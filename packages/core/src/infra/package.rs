use crate::domain::package::{
    InstallStatus, Package, PackageDefinition, PackageState, RuntimeStatus,
};
use crate::infra::docker::{
    container_is_running, create_or_recreate_network, find_container, get_docker_instance,
    pull_and_start_container, remove_container, start_named_container, stop_named_container,
};
use crate::infra::ephemery::EPHEMERY_NETWORK_NAME;
use crate::infra::file::kittynode_path;
use crate::manifests::ethereum::{self, Ethereum};
use bollard::{Docker, errors::Error as DockerError};
use eyre::{Context, Result};
use std::{
    collections::{HashMap, HashSet},
    fs,
    io::ErrorKind,
    path::Path,
};
use tracing::{error, info, warn};

use crate::infra::package_config::PackageConfigStore;

/// Retrieves the catalog of all available packages.
pub fn get_package_catalog() -> Result<HashMap<String, Package>> {
    let mut packages = HashMap::new();
    packages.insert(Ethereum::NAME.to_string(), Ethereum::get_package()?);
    Ok(packages)
}

/// Retrieves a single package or returns a not-found error.
///
/// Lookup is case-sensitive; callers should use the canonical lowercase id.
pub fn get_package_by_name(name: &str) -> Result<Package> {
    let mut catalog = get_package_catalog()?;
    catalog
        .remove(name)
        .ok_or_else(|| eyre::eyre!("Package '{}' not found", name))
}

/// Returns packages considered installed.
///
/// A package is installed when its config exists and all declared containers exist.
pub async fn get_installed_packages() -> Result<Vec<Package>> {
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
/// Installs a package using its current, concrete container definition.
///
/// Callers are responsible for ensuring required configuration has been
/// provided beforehand (e.g., selecting a network for Ethereum). When the
/// package is not fully configured, installation fails with a clear error.
pub async fn install_package(package: &Package) -> Result<()> {
    validate_package_installable(package)?;
    let docker = get_docker_instance().await?;

    let containers = package.containers.clone();

    info!("Creating network '{}'...", package.network_name);
    create_or_recreate_network(&docker, &package.network_name).await?;

    for container in &containers {
        info!("Starting container '{}'...", container.name);
        pull_and_start_container(&docker, container, &package.network_name).await?;
        info!("Container '{}' started successfully", container.name);
    }

    Ok(())
}

fn validate_package_installable(package: &Package) -> Result<()> {
    if !package.containers.is_empty() {
        return Ok(());
    }

    if package.name == Ethereum::NAME {
        let config = PackageConfigStore::load(Ethereum::NAME)?;
        let network_missing = !config.values.contains_key("network");
        if network_missing {
            let network_choices = ethereum::supported_networks_display("|");
            return Err(eyre::eyre!(
                "Network must be selected before installing Ethereum. Install using `kittynode package install {} --network <{}>`",
                Ethereum::NAME,
                network_choices
            ));
        }
    }

    Err(eyre::eyre!(
        "Package '{}' is not fully configured for installation",
        package.name
    ))
}

pub async fn stop_package(package: &Package) -> Result<()> {
    let docker = get_docker_instance().await?;

    for container in &package.containers {
        info!("Stopping container '{}'", container.name);
        stop_named_container(&docker, &container.name).await?;
        info!("Container '{}' stopped", container.name);
    }

    Ok(())
}

pub async fn start_package(package: &Package) -> Result<()> {
    let docker = get_docker_instance().await?;

    for container in &package.containers {
        info!("Starting container '{}'", container.name);
        start_named_container(&docker, &container.name).await?;
        info!("Container '{}' started", container.name);
    }

    Ok(())
}

/// Deletes a package and its associated resources.
/// When `purge_ephemery_cache` is true (explicit uninstall), all file bindings
/// including persistent RW mounts (e.g., ~/.config/kittynode/.lighthouse) are removed.
/// When false (config restart), only ephemeral RO mounts are cleaned up and
/// persistent user data is preserved.
pub async fn delete_package(
    package: &Package,
    include_images: bool,
    purge_ephemery_cache: bool,
) -> Result<()> {
    if package.containers.is_empty() {
        if purge_ephemery_cache
            && package.name == Ethereum::NAME
            && let Ok(root) = kittynode_path()
        {
            let root_dir = root
                .join("packages")
                .join("ethereum")
                .join("networks")
                .join(EPHEMERY_NETWORK_NAME);
            if root_dir.exists() {
                info!("Removing directory '{}'...", root_dir.display());
                fs::remove_dir_all(&root_dir)?;
                info!("Directory '{}' removed successfully", root_dir.display());
            }
        }

        if purge_ephemery_cache && let Ok(config_root) = kittynode_path() {
            remove_package_config_artifacts(&config_root, package.name())?;
        }

        return Ok(());
    }

    let docker = get_docker_instance().await?;

    // Track every resource that needs cleanup after containers are removed.
    let mut image_names = Vec::new();
    let mut file_paths = HashSet::new();
    let mut directory_paths = HashSet::new();

    let mut volume_names = Vec::new();

    for container in &package.containers {
        if include_images {
            image_names.push(&container.image);
        }

        volume_names.extend(container.volume_bindings.iter().map(|b| &b.source));

        for binding in &container.file_bindings {
            let is_read_only = binding
                .options
                .as_deref()
                .map(|opts| opts.contains("ro"))
                .unwrap_or(false);

            // Config restarts retain RW mounts unless we are explicitly purging user data.
            let should_consider = is_read_only || purge_ephemery_cache;
            if !should_consider {
                continue;
            }

            if let Ok(metadata) = fs::metadata(&binding.source) {
                if metadata.is_dir() {
                    // Ephemery metadata is preserved during config restarts but purged on uninstall.
                    let is_ephemery_mount = binding.destination == "/root/networks/ephemery";
                    if !is_ephemery_mount || purge_ephemery_cache {
                        directory_paths.insert(&binding.source);
                    }
                } else {
                    file_paths.insert(&binding.source);
                }
            }
        }

        info!("Removing container '{}'...", container.name);
        remove_container(&docker, &container.name).await?;
        info!("Container '{}' removed successfully", container.name);
    }

    for image in image_names {
        info!("Removing image '{}'...", image);
        docker
            .remove_image(
                image,
                None::<bollard::query_parameters::RemoveImageOptions>,
                None,
            )
            .await?;
        info!("Image '{}' removed successfully", image);
    }

    for path in file_paths {
        info!("Removing file '{}'...", path);
        match fs::remove_file(path) {
            Ok(()) => info!("File '{}' removed successfully", path),
            Err(err) if err.kind() == ErrorKind::PermissionDenied => warn!(
                "Skipping removal of '{}' because permissions are insufficient",
                path
            ),
            Err(err) => return Err(err.into()),
        }
    }
    for path in directory_paths {
        info!("Removing directory '{}'...", path);
        match fs::remove_dir_all(path) {
            Ok(()) => info!("Directory '{}' removed successfully", path),
            Err(err) if err.kind() == ErrorKind::PermissionDenied => warn!(
                "Skipping removal of '{}' because permissions are insufficient",
                path
            ),
            Err(err) => return Err(err.into()),
        }
    }

    // Preserve persistent data (e.g., Lighthouse keystores) during config restarts.
    // Only remove named volumes on explicit uninstalls/purges.
    if purge_ephemery_cache {
        for volume in volume_names {
            info!("Removing volume '{}'...", volume);
            match docker
                .remove_volume(
                    volume,
                    None::<bollard::query_parameters::RemoveVolumeOptions>,
                )
                .await
            {
                Ok(_) => info!("Volume '{}' removed successfully", volume),
                Err(DockerError::DockerResponseServerError {
                    status_code: 404, ..
                }) => {
                    warn!(
                        "Skipping removal of volume '{}' because it does not exist",
                        volume
                    );
                    continue;
                }
                Err(err) => return Err(err.into()),
            }
        }
    } else {
        info!("Preserving named volumes during config update");
    }

    info!("Removing network '{}'...", package.network_name);
    match docker.remove_network(&package.network_name).await {
        Ok(_) => info!("Network '{}' removed successfully", package.network_name),
        Err(DockerError::DockerResponseServerError {
            status_code: 404, ..
        }) => {
            warn!(
                "Skipping removal of network '{}' because it does not exist",
                package.network_name
            );
        }
        Err(err) => return Err(err.into()),
    }

    if purge_ephemery_cache
        && package.name == Ethereum::NAME
        && let Ok(root) = kittynode_path()
    {
        let root_dir = root
            .join("packages")
            .join("ethereum")
            .join("networks")
            .join(EPHEMERY_NETWORK_NAME);
        if root_dir.exists() {
            info!("Removing directory '{}'...", root_dir.display());
            fs::remove_dir_all(&root_dir)?;
            info!("Directory '{}' removed successfully", root_dir.display());
        }
    }

    if purge_ephemery_cache && let Ok(config_root) = kittynode_path() {
        remove_package_config_artifacts(&config_root, package.name())?;
    }

    Ok(())
}

fn remove_package_config_artifacts(base_dir: &Path, package_name: &str) -> Result<()> {
    let config_path = PackageConfigStore::config_file_path(base_dir, package_name);
    if config_path.exists() {
        info!(
            "Removing package configuration '{}'...",
            config_path.display()
        );
        match fs::remove_file(&config_path) {
            Ok(()) => info!(
                "Package configuration '{}' removed successfully",
                config_path.display()
            ),
            Err(err) => {
                if err.kind() == ErrorKind::PermissionDenied {
                    error!(
                        "Failed to remove '{}' because permissions are insufficient",
                        config_path.display()
                    );
                }
                return Err(err).wrap_err_with(|| {
                    format!(
                        "Failed to remove package configuration '{}'",
                        config_path.display()
                    )
                });
            }
        }
    }

    let package_dir = PackageConfigStore::package_dir(base_dir, package_name);
    if package_dir.exists() {
        info!("Removing package directory '{}'...", package_dir.display());
        match fs::remove_dir_all(&package_dir) {
            Ok(()) => info!(
                "Package directory '{}' removed successfully",
                package_dir.display()
            ),
            Err(err) => {
                if err.kind() == ErrorKind::PermissionDenied {
                    error!(
                        "Failed to remove '{}' because permissions are insufficient",
                        package_dir.display()
                    );
                }
                return Err(err).wrap_err_with(|| {
                    format!(
                        "Failed to remove package directory '{}'",
                        package_dir.display()
                    )
                });
            }
        }
    }

    Ok(())
}

/// Computes full package state.
pub async fn get_package(package: &Package) -> Result<PackageState> {
    if package.containers.is_empty() {
        return get_package_without_docker(package);
    }
    let docker = get_docker_instance().await?;
    get_package_with(&docker, package).await
}

/// Internal helper to compute state using a provided Docker client.
async fn get_package_with(docker: &Docker, package: &Package) -> Result<PackageState> {
    let base = kittynode_path()?;
    let cfg = PackageConfigStore::config_file_path(&base, package.name());
    let config_present = cfg.exists();

    let total = package.containers.len();
    let mut missing = Vec::new();
    let mut running_count = 0usize;

    for c in &package.containers {
        let summaries = find_container(docker, &c.name).await?;
        if summaries.is_empty() {
            missing.push(c.name.clone());
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
    let cfg = PackageConfigStore::config_file_path(&base, package.name());
    let config_present = cfg.exists();
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
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn get_package_by_name_is_case_sensitive_with_lowercase_canonical() {
        // Canonical key is lowercase
        let catalog = get_package_catalog().expect("catalog should load");
        assert!(catalog.contains_key("ethereum"));

        // Exact lowercase match succeeds
        assert!(get_package_by_name("ethereum").is_ok());

        // Mixed case should not match
        assert!(get_package_by_name("Ethereum").is_err());

        // Non-existent package should report a not found error
        let missing = get_package_by_name("does-not-exist");
        assert!(missing.is_err());
    }

    #[test]
    fn remove_package_config_artifacts_removes_directory_when_empty() {
        let temp_dir = tempdir().expect("failed to create temp dir");
        let base_dir = temp_dir.path().join(".config").join("kittynode");
        let package_dir = PackageConfigStore::package_dir(&base_dir, "ethereum");
        fs::create_dir_all(&package_dir).expect("failed to create package dir");
        let config_path = package_dir.join("config.toml");
        fs::write(&config_path, "network = \"ephemery\"").expect("failed to write config");

        remove_package_config_artifacts(&base_dir, "ethereum")
            .expect("config artifacts should be removed");

        assert!(
            !config_path.exists(),
            "config file should be deleted during removal"
        );
        assert!(!package_dir.exists(), "package directory should be deleted");
    }

    #[test]
    fn remove_package_config_artifacts_removes_directory_when_non_empty() {
        let temp_dir = tempdir().expect("failed to create temp dir");
        let base_dir = temp_dir.path().join(".config").join("kittynode");
        let package_dir = PackageConfigStore::package_dir(&base_dir, "ethereum");
        fs::create_dir_all(&package_dir).expect("failed to create package dir");
        let config_path = package_dir.join("config.toml");
        let jwt_path = package_dir.join("jwt.hex");
        fs::write(&config_path, "network = \"ephemery\"").expect("failed to write config");
        fs::write(&jwt_path, "abc123").expect("failed to write jwt secret");

        remove_package_config_artifacts(&base_dir, "ethereum")
            .expect("config artifacts should be removed");

        assert!(
            !config_path.exists(),
            "config file should be deleted during removal"
        );
        assert!(
            !package_dir.exists(),
            "package directory should be deleted even when other artifacts are present"
        );
        assert!(
            !jwt_path.exists(),
            "all artifacts in the package directory should be removed"
        );
    }
}
