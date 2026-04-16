#[path = "packages/catalog.rs"]
mod catalog;
#[path = "packages/lifecycle.rs"]
mod lifecycle;
#[path = "packages/state.rs"]
mod state;
#[path = "packages/store.rs"]
mod store;
#[path = "packages/types.rs"]
mod types;

pub use store::PackageConfigStore;
pub use types::{
    Binding, Container, InstallStatus, Package, PackageConfig, PackageState, PortBinding,
    RuntimeStatus,
};

pub(crate) use types::PackageDefinition;

use eyre::Result;
use std::collections::HashMap;

pub fn get_package_catalog() -> Result<HashMap<String, Package>> {
    catalog::get_package_catalog()
}

pub async fn get_package(name: &str) -> Result<PackageState> {
    state::get_package(name).await
}

pub async fn get_packages(names: &[&str]) -> Result<HashMap<String, PackageState>> {
    state::get_packages(names).await
}

pub async fn get_installed_packages() -> Result<Vec<Package>> {
    state::get_installed_packages().await
}

pub async fn get_package_config(package_name: &str) -> Result<PackageConfig> {
    PackageConfigStore::load(package_name)
}

pub async fn install_package(name: &str) -> Result<()> {
    lifecycle::install_package(name).await
}

pub async fn install_package_with_network(name: &str, network: Option<&str>) -> Result<()> {
    lifecycle::install_package_with_network(name, network).await
}

pub async fn start_package(name: &str) -> Result<()> {
    lifecycle::start_package(name).await
}

pub async fn stop_package(name: &str) -> Result<()> {
    lifecycle::stop_package(name).await
}

pub async fn delete_package(name: &str, include_images: bool) -> Result<()> {
    lifecycle::delete_package(name, include_images).await
}

pub async fn update_package_config(package_name: &str, config: PackageConfig) -> Result<()> {
    lifecycle::update_package_config(package_name, config).await
}
