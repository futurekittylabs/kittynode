use crate::infra::{file::generate_jwt_secret, package, package_config::PackageConfigStore};
use eyre::{Context, Result};
use tracing::info;

pub async fn install_package(name: &str) -> Result<()> {
    generate_jwt_secret().wrap_err("Failed to generate JWT secret")?;

    let package = package::get_package_by_name(name)?;

    let config = PackageConfigStore::load(name)?;
    let network = config.values.get("network");

    package::install_package(&package, network.map(String::as_str)).await?;
    info!("Package '{}' installed successfully.", name);
    Ok(())
}
