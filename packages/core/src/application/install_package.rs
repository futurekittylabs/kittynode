use crate::domain::package::InstallStatus;
use crate::infra::{file::generate_jwt_secret, package};
use crate::manifests::ethereum;
use crate::packages::ethereum::{
    config::{EthereumConfig, Network, Validator},
    config_store,
};
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
        if name != ethereum::ETHEREUM_NAME {
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

        let parsed_network: Network = network
            .parse()
            .wrap_err_with(|| format!("Failed to parse Ethereum network '{network}'"))?;
        let mut config = config_store::load()
            .wrap_err_with(|| format!("Failed to load configuration for {name}"))?
            .unwrap_or_else(|| EthereumConfig {
                network: parsed_network,
                validator: Validator::default(),
            });
        config.network = parsed_network;
        config
            .validate()
            .wrap_err_with(|| format!("Invalid Ethereum configuration for {name}"))?;
        config_store::save(&config)
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

    generate_jwt_secret(name).wrap_err("Failed to generate JWT secret")?;
    let package = package::get_package_by_name(name)?;

    package::install_package(&package).await?;
    info!("Package '{name}' installed successfully");
    Ok(())
}
