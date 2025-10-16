use crate::domain::package::PackageDefinition;
use crate::infra::{file::generate_jwt_secret, package, package_config::PackageConfigStore};
use crate::manifests::ethereum::{self, Ethereum};
use eyre::{Context, Result, eyre};
use tracing::info;

pub async fn install_package(name: &str) -> Result<()> {
    install_package_with_network(name, None).await
}

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

    generate_jwt_secret().wrap_err("Failed to generate JWT secret")?;

    let package = package::get_package_by_name(name)?;

    package::install_package(&package).await?;
    info!("Package '{name}' installed successfully");
    Ok(())
}
