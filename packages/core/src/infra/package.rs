use crate::domain::package::{Package, PackageDefinition, PackageRuntimeState};
use crate::infra::docker::{
    container_is_running, create_or_recreate_network, find_container, get_docker_instance,
    pull_and_start_container, remove_container, start_named_container, stop_named_container,
};
use crate::infra::ephemery::EPHEMERY_NETWORK_NAME;
use crate::infra::file::kittynode_path;
use crate::manifests::ethereum::Ethereum;
use eyre::Result;
use std::{
    collections::{HashMap, HashSet},
    fs,
};
use tracing::info;

/// Retrieves a `HashMap` of all available packages.
pub fn get_packages() -> Result<HashMap<String, Package>> {
    let mut packages = HashMap::new();
    packages.insert(Ethereum::NAME.to_string(), Ethereum::get_package()?);
    Ok(packages)
}

/// Retrieves a single package or returns a not-found error.
pub fn get_package_by_name(name: &str) -> Result<Package> {
    let mut catalog = get_packages()?;
    catalog
        .remove(name)
        .ok_or_else(|| eyre::eyre!("Package '{}' not found", name))
}

/// Gets a list of installed packages by checking their container states
pub async fn get_installed_packages(packages: &HashMap<String, Package>) -> Result<Vec<Package>> {
    let docker = get_docker_instance().await?;
    let mut installed = Vec::new();

    for package in packages.values() {
        let mut all_containers_exist = true;

        for container in &package.containers {
            info!("Checking container '{}'...", container.name);
            if find_container(&docker, &container.name).await?.is_empty() {
                all_containers_exist = false;
                break;
            }
        }

        if all_containers_exist {
            installed.push(package.clone());
        }
    }

    Ok(installed)
}

/// Installs a package with the given network configuration
pub async fn install_package(package: &Package, network: Option<&str>) -> Result<()> {
    let docker = get_docker_instance().await?;
    let containers = match package.name.as_str() {
        "Ethereum" => Ethereum::get_containers(network.unwrap_or("hoodi"))?,
        _ => package.containers.clone(),
    };

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

pub async fn resume_package(package: &Package) -> Result<()> {
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

    // Clean up containers and collect resources to remove
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
            // Determine if mount is read-only; read-write mounts are persistent user data.
            let is_read_only = binding
                .options
                .as_deref()
                .map(|opts| opts.contains("ro"))
                .unwrap_or(false);

            // During a config restart (purge_ephemery_cache == false), only clean up RO mounts.
            // During an explicit uninstall (purge_ephemery_cache == true), also remove RW mounts
            // so that all Kittynode-created data is deleted.
            let should_consider = is_read_only || purge_ephemery_cache;
            if !should_consider {
                continue;
            }

            if let Ok(metadata) = fs::metadata(&binding.source) {
                if metadata.is_dir() {
                    // Skip Ephemery metadata mount during config restart; purge on explicit uninstall
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

    // Clean up images if requested
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

    // Clean up files and directories
    for path in file_paths {
        info!("Removing file '{}'...", path);
        fs::remove_file(path)?;
        info!("File '{}' removed successfully", path);
    }
    for path in directory_paths {
        info!("Removing directory '{}'...", path);
        fs::remove_dir_all(path)?;
        info!("Directory '{}' removed successfully", path);
    }

    // Clean up Docker volumes and network
    for volume in volume_names {
        info!("Removing volume '{}'...", volume);
        docker
            .remove_volume(
                volume,
                None::<bollard::query_parameters::RemoveVolumeOptions>,
            )
            .await?;
        info!("Volume '{}' removed successfully", volume);
    }

    info!("Removing network '{}'...", package.network_name);
    docker.remove_network(&package.network_name).await?;
    info!("Network '{}' removed successfully", package.network_name);

    // Optionally purge the Ephemery cache on explicit uninstall
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
