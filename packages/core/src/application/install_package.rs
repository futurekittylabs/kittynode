use crate::infra::{
    file::generate_jwt_secret,
    home::Home,
    package::{self, get_packages},
    package_config::PackageConfigStore,
};
use eyre::{Context, Result};
use tracing::info;

pub async fn install_package(name: &str) -> Result<()> {
    generate_jwt_secret().wrap_err("Failed to generate JWT secret")?;

    let package = get_packages()?
        .remove(name)
        .ok_or_else(|| eyre::eyre!("Package '{}' not found", name))?;

    let home = Home::try_default()?;
    let config = PackageConfigStore::load_from(&home, name)?;
    let network = config.values.get("network");

    package::install_package(&package, network.map(String::as_str)).await?;
    info!("Package '{}' installed successfully", name);
    Ok(())
}
