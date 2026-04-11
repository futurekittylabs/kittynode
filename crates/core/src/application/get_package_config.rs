use crate::domain::package::PackageConfig;
use crate::infra::package_config::PackageConfigStore;
use eyre::Result;

pub async fn get_package_config(package_name: &str) -> Result<PackageConfig> {
    PackageConfigStore::load(package_name)
}
