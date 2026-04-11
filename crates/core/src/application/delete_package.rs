use crate::infra::package;
use eyre::Result;
use tracing::info;

pub async fn delete_package(name: &str, include_images: bool) -> Result<()> {
    let package = package::get_package_by_name(name)?;
    // Explicit uninstalls purge the Ephemery cache and associated metadata.
    package::delete_package(&package, include_images, true).await?;
    info!("Package '{}' deleted successfully.", name);
    Ok(())
}
