use crate::{
    domain::package::InstalledPackage,
    infra::package::{self, get_packages},
};
use eyre::{Context, Result};
use tracing::info;

pub async fn get_installed_packages() -> Result<Vec<InstalledPackage>> {
    let packages = get_packages().wrap_err("Failed to retrieve packages")?;
    let installed = package::get_installed_packages(&packages).await?;
    info!("Found {} installed packages", installed.len());
    Ok(installed)
}
