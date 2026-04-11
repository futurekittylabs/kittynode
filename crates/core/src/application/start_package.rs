use crate::infra::package;
use eyre::Result;
use tracing::info;

pub async fn start_package(name: &str) -> Result<()> {
    let package = package::get_package_by_name(name)?;

    package::start_package(&package).await?;
    info!("Package '{name}' started");
    Ok(())
}
