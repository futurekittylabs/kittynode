use crate::domain::package::PackageConfig;
use crate::infra::{home::Home, package_config::PackageConfigStore};
use eyre::Result;

pub async fn get_package_config(package_name: &str) -> Result<PackageConfig> {
    let home = Home::try_default()?;
    PackageConfigStore::load_from(&home, package_name)
}
