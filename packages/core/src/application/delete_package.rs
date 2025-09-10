use crate::infra::package::{self, get_packages};
use eyre::Result;
use tracing::info;

pub async fn delete_package(name: &str, include_images: bool) -> Result<()> {
    let package = get_packages()?
        .get(name)
        .ok_or_else(|| eyre::eyre!("Package '{}' not found", name))?
        .clone();

    package::delete_package(&package, include_images).await?;
    info!("Package '{}' deleted successfully", name);
    Ok(())
}
