use crate::domain::package::{InstallStatus, PackageDefinition};
use crate::infra::{file::generate_jwt_secret, package, package_config::PackageConfigStore};
use crate::manifests::ethereum::{self, Ethereum};
use eyre::{Context, Result, eyre};
use tracing::{info, warn};

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

    generate_jwt_secret(name).wrap_err("Failed to generate JWT secret")?;

    let package = package::get_package_by_name(name)?;

    let state = package::get_package(&package).await?;
    match state.install {
        InstallStatus::Installed => {
            warn!(
                "Package '{name}' already installed; refreshing containers to resolve potential drift"
            );
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
                "Package '{name}' is partially installed ({note}). Reinstalling to restore resources"
            );
        }
        InstallStatus::NotInstalled => {}
    }

    package::install_package(&package).await?;
    info!("Package '{name}' installed successfully");
    Ok(())
}
