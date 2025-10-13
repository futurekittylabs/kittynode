use crate::infra::{file::generate_jwt_secret, package};
use eyre::{Context, Result};
use tracing::info;

pub async fn install_package(name: &str) -> Result<()> {
    generate_jwt_secret().wrap_err("Failed to generate JWT secret")?;

    let package = package::get_package_by_name(name)?;

    package::install_package(&package).await?;
    info!("Package '{}' installed successfully.", name);
    Ok(())
}
