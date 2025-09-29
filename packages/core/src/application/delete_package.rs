use crate::infra::package;
use eyre::Result;
use tracing::info;

pub async fn delete_package(name: &str, include_images: bool) -> Result<()> {
    let package = package::get_package_by_name(name)?;

    package::delete_package(&package, include_images).await?;
    info!("Package '{}' deleted successfully.", name);
    Ok(())
}
