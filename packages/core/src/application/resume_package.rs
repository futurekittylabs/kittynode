use crate::infra::package;
use eyre::Result;
use tracing::info;

pub async fn resume_package(name: &str) -> Result<()> {
    let package = package::get_package_by_name(name)?;

    package::resume_package(&package).await?;
    info!("Package '{}' resumed", name);
    Ok(())
}
