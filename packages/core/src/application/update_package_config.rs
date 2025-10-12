use crate::application::install_package;
use crate::domain::package::PackageConfig;
use crate::infra::package as infra_package;
use crate::infra::package_config::PackageConfigStore;
use eyre::Result;

pub async fn update_package_config(package_name: &str, config: PackageConfig) -> Result<()> {
    // Save the new configuration
    PackageConfigStore::save(package_name, &config)?;

    // Restart the package with new configuration
    // Remove running containers and runtime artifacts but keep Ephemery cache to avoid unnecessary re-fetch
    let package = infra_package::get_package_by_name(package_name)?;
    infra_package::delete_package(&package, false, false).await?;
    install_package(package_name).await?;

    Ok(())
}
