use crate::docker::{
    create_or_recreate_network, get_docker_instance, pull_and_start_container, remove_container,
    start_named_container, stop_named_container,
};
use crate::ethereum::{self, EPHEMERY_NETWORK_NAME, Ethereum};
use crate::packages::catalog::get_package_by_name;
use crate::packages::state::get_concrete_package;
use crate::packages::{
    InstallStatus, Package, PackageConfig, PackageConfigStore, PackageDefinition,
};
use crate::paths::{generate_jwt_secret, kittynode_path};
use bollard::errors::Error as DockerError;
use eyre::{Context, Result, eyre};
use std::{collections::HashSet, fs, io::ErrorKind, path::Path};
use tracing::{error, info, warn};

pub(crate) async fn install_package(name: &str) -> Result<()> {
    install_package_with_network(name, None).await
}

pub(crate) async fn install_package_with_network(name: &str, network: Option<&str>) -> Result<()> {
    if let Some(network) = network {
        persist_selected_network(name, network)?;
    }

    let package = get_package_by_name(name)?;
    let state = get_concrete_package(&package).await?;

    match state.install {
        InstallStatus::Installed => {
            info!("Package '{name}' already installed; skipping reinstall");
            return Ok(());
        }
        InstallStatus::PartiallyInstalled => {
            let note = describe_partial_install(&state);
            warn!(
                "Package '{name}' is partially installed ({note}). Cleaning up before reinstalling"
            );
            delete_concrete_package(&package, false, false)
                .await
                .wrap_err_with(|| format!("Failed to clean up partial installation for {name}"))?;
        }
        InstallStatus::NotInstalled => {}
    }

    ensure_jwt_secret_if_needed(name, &package)?;
    install_concrete_package(&package).await?;
    info!("Package '{name}' installed successfully");
    Ok(())
}

pub(crate) async fn start_package(name: &str) -> Result<()> {
    let package = get_package_by_name(name)?;
    start_concrete_package(&package).await?;
    info!("Package '{name}' started");
    Ok(())
}

pub(crate) async fn stop_package(name: &str) -> Result<()> {
    let package = get_package_by_name(name)?;
    stop_concrete_package(&package).await?;
    info!("Package '{name}' stopped");
    Ok(())
}

pub(crate) async fn delete_package(name: &str, include_images: bool) -> Result<()> {
    let package = get_package_by_name(name)?;
    delete_concrete_package(&package, include_images, true).await?;
    info!("Package '{}' deleted successfully.", name);
    Ok(())
}

pub(crate) async fn update_package_config(package_name: &str, config: PackageConfig) -> Result<()> {
    let pre_update_package = match get_package_by_name(package_name) {
        Ok(package) => Some(package),
        Err(error) => {
            let message = error.to_string();
            if message.contains("Unsupported Ethereum network") {
                None
            } else {
                return Err(error);
            }
        }
    };

    let mut merged = PackageConfigStore::load(package_name)?;
    for (key, value) in config.values {
        merged.values.insert(key, value);
    }
    PackageConfigStore::save(package_name, &merged)?;

    if let Some(package) = &pre_update_package {
        match delete_concrete_package(package, false, false).await {
            Ok(_) => {}
            Err(error) if is_missing_docker_resource_error(&error) => {}
            Err(error) => return Err(error),
        }
    }

    install_package(package_name).await
}

async fn install_concrete_package(package: &Package) -> Result<()> {
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
        if !config.values.contains_key("network") {
            let network_choices = ethereum::supported_networks_display("|");
            return Err(eyre!(
                "Network must be selected before installing Ethereum. Install using `kittynode package install {} --network <{}>`",
                Ethereum::NAME,
                network_choices
            ));
        }
    }

    Err(eyre!(
        "Package '{}' is not fully configured for installation",
        package.name
    ))
}

async fn stop_concrete_package(package: &Package) -> Result<()> {
    let docker = get_docker_instance().await?;

    for container in &package.containers {
        info!("Stopping container '{}'", container.name);
        stop_named_container(&docker, &container.name).await?;
        info!("Container '{}' stopped", container.name);
    }

    Ok(())
}

async fn start_concrete_package(package: &Package) -> Result<()> {
    let docker = get_docker_instance().await?;

    for container in &package.containers {
        info!("Starting container '{}'", container.name);
        start_named_container(&docker, &container.name).await?;
        info!("Container '{}' started", container.name);
    }

    Ok(())
}

pub(crate) async fn delete_concrete_package(
    package: &Package,
    include_images: bool,
    purge_ephemery_cache: bool,
) -> Result<()> {
    if package.containers.is_empty() {
        remove_non_container_artifacts(package, purge_ephemery_cache)?;
        return Ok(());
    }

    let docker = get_docker_instance().await?;
    let cleanup = collect_cleanup_plan(package, include_images, purge_ephemery_cache);

    for container in &package.containers {
        info!("Removing container '{}'...", container.name);
        remove_container(&docker, &container.name).await?;
        info!("Container '{}' removed successfully", container.name);
    }

    remove_images(&docker, &cleanup.image_names).await?;
    remove_bound_files(&cleanup.file_paths)?;
    remove_bound_directories(&cleanup.directory_paths)?;
    remove_volumes(&docker, &cleanup.volume_names, purge_ephemery_cache).await?;
    remove_network(&docker, &package.network_name).await?;
    remove_non_container_artifacts(package, purge_ephemery_cache)?;

    Ok(())
}

struct CleanupPlan<'a> {
    image_names: Vec<&'a str>,
    volume_names: Vec<&'a str>,
    file_paths: HashSet<&'a str>,
    directory_paths: HashSet<&'a str>,
}

fn collect_cleanup_plan<'a>(
    package: &'a Package,
    include_images: bool,
    purge_ephemery_cache: bool,
) -> CleanupPlan<'a> {
    let mut image_names = Vec::new();
    let mut volume_names = Vec::new();
    let mut file_paths = HashSet::new();
    let mut directory_paths = HashSet::new();

    for container in &package.containers {
        if include_images {
            image_names.push(container.image.as_str());
        }

        volume_names.extend(
            container
                .volume_bindings
                .iter()
                .map(|binding| binding.source.as_str()),
        );

        for binding in &container.file_bindings {
            let is_read_only = binding
                .options
                .as_deref()
                .map(|options| options.contains("ro"))
                .unwrap_or(false);

            if !(is_read_only || purge_ephemery_cache) {
                continue;
            }

            if let Ok(metadata) = fs::metadata(&binding.source) {
                if metadata.is_dir() {
                    let is_ephemery_mount = binding.destination == "/root/networks/ephemery";
                    if !is_ephemery_mount || purge_ephemery_cache {
                        directory_paths.insert(binding.source.as_str());
                    }
                } else {
                    file_paths.insert(binding.source.as_str());
                }
            }
        }
    }

    CleanupPlan {
        image_names,
        volume_names,
        file_paths,
        directory_paths,
    }
}

async fn remove_images(docker: &bollard::Docker, image_names: &[&str]) -> Result<()> {
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

    Ok(())
}

fn remove_bound_files(file_paths: &HashSet<&str>) -> Result<()> {
    for path in file_paths {
        info!("Removing file '{}'...", path);
        match fs::remove_file(path) {
            Ok(()) => info!("File '{}' removed successfully", path),
            Err(error) if error.kind() == ErrorKind::PermissionDenied => {
                warn!(
                    "Skipping removal of '{}' because permissions are insufficient",
                    path
                )
            }
            Err(error) => return Err(error.into()),
        }
    }

    Ok(())
}

fn remove_bound_directories(directory_paths: &HashSet<&str>) -> Result<()> {
    for path in directory_paths {
        info!("Removing directory '{}'...", path);
        match fs::remove_dir_all(path) {
            Ok(()) => info!("Directory '{}' removed successfully", path),
            Err(error) if error.kind() == ErrorKind::PermissionDenied => {
                warn!(
                    "Skipping removal of '{}' because permissions are insufficient",
                    path
                )
            }
            Err(error) => return Err(error.into()),
        }
    }

    Ok(())
}

async fn remove_volumes(
    docker: &bollard::Docker,
    volume_names: &[&str],
    purge_ephemery_cache: bool,
) -> Result<()> {
    if !purge_ephemery_cache {
        info!("Preserving named volumes during config update");
        return Ok(());
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
            Err(DockerError::DockerResponseServerError {
                status_code: 404, ..
            }) => {
                warn!(
                    "Skipping removal of volume '{}' because it does not exist",
                    volume
                );
            }
            Err(error) => return Err(error.into()),
        }
    }

    Ok(())
}

async fn remove_network(docker: &bollard::Docker, network_name: &str) -> Result<()> {
    info!("Removing network '{}'...", network_name);
    match docker.remove_network(network_name).await {
        Ok(_) => info!("Network '{}' removed successfully", network_name),
        Err(DockerError::DockerResponseServerError {
            status_code: 404, ..
        }) => {
            warn!(
                "Skipping removal of network '{}' because it does not exist",
                network_name
            );
        }
        Err(error) => return Err(error.into()),
    }

    Ok(())
}

fn remove_non_container_artifacts(package: &Package, purge_ephemery_cache: bool) -> Result<()> {
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
            Err(error) => {
                if error.kind() == ErrorKind::PermissionDenied {
                    error!(
                        "Failed to remove '{}' because permissions are insufficient",
                        config_path.display()
                    );
                }
                return Err(error).wrap_err_with(|| {
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
            Err(error) => {
                if error.kind() == ErrorKind::PermissionDenied {
                    error!(
                        "Failed to remove '{}' because permissions are insufficient",
                        package_dir.display()
                    );
                }
                return Err(error).wrap_err_with(|| {
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

fn persist_selected_network(name: &str, network: &str) -> Result<()> {
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
        .wrap_err_with(|| format!("Failed to persist configuration for {name}"))
}

fn describe_partial_install(state: &crate::packages::PackageState) -> String {
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

    if details.is_empty() {
        "partial state detected".to_string()
    } else {
        details.join(", ")
    }
}

fn ensure_jwt_secret_if_needed(name: &str, package: &Package) -> Result<()> {
    if !package_requires_jwt_secret(package) {
        return Ok(());
    }

    generate_jwt_secret(name).wrap_err("Failed to generate JWT secret")?;
    Ok(())
}

fn package_requires_jwt_secret(package: &Package) -> bool {
    !package.containers.is_empty()
}

fn is_missing_docker_resource_error(error: &eyre::Report) -> bool {
    let message = error.to_string().to_lowercase();
    message.contains("no such volume")
        || (message.contains("volume") && message.contains("not found"))
        || message.contains("no such network")
        || (message.contains("network") && message.contains("not found"))
}

#[cfg(test)]
mod tests {
    use super::{
        package_requires_jwt_secret, remove_package_config_artifacts, validate_package_installable,
    };
    use crate::packages::{Container, Package, PackageConfig};
    use std::{collections::HashMap, fs};
    use tempfile::tempdir;

    #[test]
    fn remove_package_config_artifacts_removes_directory_when_empty() {
        let temp_dir = tempdir().expect("failed to create temp dir");
        let base_dir = temp_dir.path().join(".config").join("kittynode");
        let package_dir = crate::packages::PackageConfigStore::package_dir(&base_dir, "ethereum");
        fs::create_dir_all(&package_dir).expect("failed to create package dir");
        let config_path = package_dir.join("config.toml");
        fs::write(&config_path, "network = \"ephemery\"").expect("failed to write config");

        remove_package_config_artifacts(&base_dir, "ethereum")
            .expect("config artifacts should be removed");

        assert!(!config_path.exists());
        assert!(!package_dir.exists());
    }

    #[test]
    fn remove_package_config_artifacts_removes_directory_when_non_empty() {
        let temp_dir = tempdir().expect("failed to create temp dir");
        let base_dir = temp_dir.path().join(".config").join("kittynode");
        let package_dir = crate::packages::PackageConfigStore::package_dir(&base_dir, "ethereum");
        fs::create_dir_all(&package_dir).expect("failed to create package dir");
        let config_path = package_dir.join("config.toml");
        let jwt_path = package_dir.join("jwt.hex");
        fs::write(&config_path, "network = \"ephemery\"").expect("failed to write config");
        fs::write(&jwt_path, "abc123").expect("failed to write jwt secret");

        remove_package_config_artifacts(&base_dir, "ethereum")
            .expect("config artifacts should be removed");

        assert!(!config_path.exists());
        assert!(!package_dir.exists());
        assert!(!jwt_path.exists());
    }

    #[test]
    fn validate_package_installable_allows_packages_with_containers() {
        let package = Package {
            name: "ethereum".to_string(),
            description: "test".to_string(),
            network_name: "test".to_string(),
            containers: vec![Container {
                name: "dummy".to_string(),
                image: "dummy".to_string(),
                cmd: Vec::new(),
                port_bindings: HashMap::new(),
                volume_bindings: Vec::new(),
                file_bindings: Vec::new(),
            }],
            default_config: PackageConfig::default(),
        };

        validate_package_installable(&package).expect("should be installable");
    }

    #[test]
    fn package_requires_jwt_secret_for_packages_with_containers() {
        let package = Package {
            name: "test-package".to_string(),
            description: "test".to_string(),
            network_name: "test".to_string(),
            containers: vec![Container {
                name: "dummy".to_string(),
                image: "dummy".to_string(),
                cmd: Vec::new(),
                port_bindings: HashMap::new(),
                volume_bindings: Vec::new(),
                file_bindings: Vec::new(),
            }],
            default_config: PackageConfig::default(),
        };

        assert!(package_requires_jwt_secret(&package));
    }

    #[test]
    fn package_requires_jwt_secret_skips_packages_without_containers() {
        let package = Package {
            name: "test-package".to_string(),
            description: "test".to_string(),
            network_name: "test".to_string(),
            containers: Vec::new(),
            default_config: PackageConfig::default(),
        };

        assert!(!package_requires_jwt_secret(&package));
    }
}
