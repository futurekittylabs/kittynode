use crate::{domain::package::Package, infra::package};
use eyre::{Context, Result};
use tracing::info;

pub async fn get_installed_packages() -> Result<Vec<Package>> {
    let installed = package::get_installed_packages()
        .await
        .wrap_err("Failed to list installed packages")?;
    info!("Found {} installed packages", installed.len());
    Ok(installed)
}
