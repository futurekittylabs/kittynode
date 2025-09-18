use crate::domain::package::{Package, PackageDefinition};
use crate::infra::docker::{
    create_or_recreate_network, find_container, get_docker_instance, pull_and_start_container,
    remove_container,
};
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
        "Ethereum" => Ethereum::get_containers(network.unwrap_or("holesky"))?,
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

/// Deletes a package and its associated resources
pub async fn delete_package(package: &Package, include_images: bool) -> Result<()> {
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
            if let Ok(metadata) = fs::metadata(&binding.source) {
                if metadata.is_dir() {
                    directory_paths.insert(&binding.source);
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

    Ok(())
}
