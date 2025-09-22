use crate::{
    domain::package::PackageRuntimeState,
    infra::package::{self, get_packages},
};
use eyre::Result;

pub async fn get_package_runtime_state(name: &str) -> Result<PackageRuntimeState> {
    let package = get_packages()?
        .get(name)
        .ok_or_else(|| eyre::eyre!("Package '{}' not found", name))?
        .clone();

    package::get_package_runtime_state(&package).await
}
