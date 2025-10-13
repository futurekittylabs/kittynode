use crate::infra::{file::generate_jwt_secret, package, package_config::PackageConfigStore};
use eyre::{Context, Result};
use tracing::info;

pub async fn install_package(name: &str) -> Result<()> {
    generate_jwt_secret().wrap_err("Failed to generate JWT secret")?;

    let package = package::get_package_by_name(name)?;
    // Load package config explicitly to surface TOML parse errors to the caller.
    // We don't need the values here; this ensures malformed configs don't get
    // silently ignored by manifest defaults.
    let _ = PackageConfigStore::load(name)?;

    package::install_package(&package).await?;
    info!("Package '{}' installed successfully.", name);
    Ok(())
}
