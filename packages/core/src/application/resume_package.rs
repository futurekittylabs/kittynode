use crate::infra::package::{self, get_packages};
use eyre::Result;
use tracing::info;

pub async fn resume_package(name: &str) -> Result<()> {
    let package = get_packages()?
        .get(name)
        .ok_or_else(|| eyre::eyre!("Package '{}' not found", name))?
        .clone();

    package::resume_package(&package).await?;
    info!("Package '{}' resumed", name);
    Ok(())
}
