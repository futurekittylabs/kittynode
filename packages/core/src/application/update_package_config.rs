use crate::application::delete_package;
use crate::application::install_package;
use crate::domain::package::PackageConfig;
use crate::infra::{home::Home, package_config::PackageConfigStore};
use eyre::Result;

pub async fn update_package_config(package_name: &str, config: PackageConfig) -> Result<()> {
    // Save the new configuration
    let home = Home::try_default()?;
    PackageConfigStore::save_to(&home, package_name, &config)?;

    // Restart the package with new configuration
    delete_package(package_name, false).await?;
    install_package(package_name).await?;

    Ok(())
}
