use crate::domain::package::{Package, PackageDefinition, PackageRuntimeState};
use crate::infra::docker::{
    container_is_running, create_or_recreate_network, find_container, get_docker_instance,
    pull_and_start_container, remove_container, start_named_container, stop_named_container,
};
use crate::infra::ephemery::EPHEMERY_NETWORK_NAME;
use crate::infra::file::kittynode_path;
use crate::manifests::ethereum::{self, Ethereum};
use bollard::Docker;
use eyre::{Context, Result};
use std::{
    collections::{HashMap, HashSet},
    fs,
    io::ErrorKind,
};
use tracing::{info, warn};

/// Represents whether the Docker resources for a package are fully present.
#[derive(Debug, Clone)]
pub enum PackageInstallState {
    /// Every declared container for the package is present in Docker.
    Installed,
    /// Some containers are missing, leaving the package in an inconsistent state.
    PartiallyInstalled {
        /// Containers that were not found in Docker.
        missing_containers: Vec<String>,
    },
    /// None of the required containers are present.
    NotInstalled,
}

/// Retrieves a `HashMap` of all available packages.
pub fn get_packages() -> Result<HashMap<String, Package>> {
    let mut packages = HashMap::new();
    packages.insert(Ethereum::NAME.to_string(), Ethereum::get_package()?);
    Ok(packages)
}

/// Retrieves a single package or returns a not-found error.
///
/// Lookup is case-sensitive; callers should use the canonical lowercase id.
pub fn get_package_by_name(name: &str) -> Result<Package> {
    let mut catalog = get_packages()?;
    catalog
        .remove(name)
        .ok_or_else(|| eyre::eyre!("Package '{}' not found", name))
}

/// Returns every package whose containers currently exist in Docker.
/// A package is included when each declared container has at least one matching Docker instance.
pub async fn get_installed_packages() -> Result<Vec<Package>> {
    let packages = get_packages().wrap_err("Failed to retrieve packages")?;
    let docker = get_docker_instance().await?;
    let mut installed = Vec::new();

    for package in packages.values() {
        match package_install_state(&docker, package).await? {
            PackageInstallState::Installed => installed.push(package.clone()),
            PackageInstallState::PartiallyInstalled { .. } | PackageInstallState::NotInstalled => {}
        }
    }

    Ok(installed)
}

/// Reports whether all containers required by the package are present in Docker.
/// Returns [`PackageInstallState::Installed`] when every container exists, [`PackageInstallState::NotInstalled`]
/// when none exist, and [`PackageInstallState::PartiallyInstalled`] when only a subset is present.
pub async fn is_package_installed(package: &Package) -> Result<PackageInstallState> {
    let docker = get_docker_instance().await?;
    package_install_state(&docker, package).await
}

/// Installs a package using its current, concrete container definition.
///
/// Callers are responsible for ensuring required configuration has been
/// provided beforehand (e.g., selecting a network for Ethereum). When the
/// package is not fully configured, installation fails with a clear error.
pub async fn install_package(package: &Package) -> Result<()> {
    let docker = get_docker_instance().await?;

    if package.containers.is_empty() {
        if package.name == Ethereum::NAME {
            let network_choices = ethereum::supported_networks_display("|");
            return Err(eyre::eyre!(
                "Network must be selected before installing Ethereum. Install using `kittynode package install {} --network <{}>`",
                Ethereum::NAME,
                network_choices
            ));
        }
        return Err(eyre::eyre!(
            "Package '{}' is not fully configured for installation",
            package.name
        ));
    }

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

pub async fn get_package_runtime_state(package: &Package) -> Result<PackageRuntimeState> {
    let docker = get_docker_instance().await?;
    if package.containers.is_empty() {
        return Ok(PackageRuntimeState { running: false });
    }
    let mut running = true;

    for container in &package.containers {
        let summaries = find_container(&docker, &container.name).await?;
        let container_running = summaries.iter().any(container_is_running);

        if !container_running {
            running = false;
        }
    }

    Ok(PackageRuntimeState { running })
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
            Err(err) => {
                let msg = err.to_string().to_lowercase();
                let missing = msg.contains("no such volume")
                    || (msg.contains("volume") && msg.contains("not found"));
                if missing {
                    warn!(
                        "Skipping removal of volume '{}' because it does not exist",
                        volume
                    );
                    continue;
                }
                return Err(err.into());
            }
        }
    }

    info!("Removing network '{}'...", package.network_name);
    docker.remove_network(&package.network_name).await?;
    info!("Network '{}' removed successfully", package.network_name);

    if purge_ephemery_cache
        && package.name == Ethereum::NAME
        && let Ok(root) = kittynode_path()
    {
        let root_dir = root.join("networks").join(EPHEMERY_NETWORK_NAME);
        if root_dir.exists() {
            info!("Removing directory '{}'...", root_dir.display());
            fs::remove_dir_all(&root_dir)?;
            info!("Directory '{}' removed successfully", root_dir.display());
        }
    }

    Ok(())
}

/// Inspects Docker for each declared container, noting missing names to decide whether the package is fully installed,
/// partially installed, or absent.
async fn package_install_state(docker: &Docker, package: &Package) -> Result<PackageInstallState> {
    if package.containers.is_empty() {
        return Ok(PackageInstallState::NotInstalled);
    }

    let mut missing = Vec::new();

    for container in &package.containers {
        info!("Checking container '{}'...", container.name);
        if find_container(docker, &container.name).await?.is_empty() {
            missing.push(container.name.clone());
        }
    }

    if missing.is_empty() {
        return Ok(PackageInstallState::Installed);
    }

    if missing.len() == package.containers.len() {
        return Ok(PackageInstallState::NotInstalled);
    }

    Ok(PackageInstallState::PartiallyInstalled {
        missing_containers: missing,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_package_by_name_is_case_sensitive_with_lowercase_canonical() {
        // Canonical key is lowercase
        let catalog = get_packages().expect("catalog should load");
        assert!(catalog.contains_key("ethereum"));

        // Exact lowercase match succeeds
        assert!(get_package_by_name("ethereum").is_ok());

        // Mixed case should not match
        assert!(get_package_by_name("Ethereum").is_err());

        // Non-existent package should report a not found error
        let missing = get_package_by_name("does-not-exist");
        assert!(missing.is_err());
    }
}
