use crate::infra::package;
use eyre::Result;
use tracing::info;

pub async fn stop_package(name: &str) -> Result<()> {
    let package = package::get_package_by_name(name)?;

    package::stop_package(&package).await?;
    info!("Package '{}' stopped", name);
    Ok(())
}
