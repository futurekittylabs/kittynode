use crate::{domain::package::PackageRuntimeState, infra::package};
use eyre::Result;

pub async fn get_package_runtime_state(name: &str) -> Result<PackageRuntimeState> {
    let package = package::get_package_by_name(name)?;

    package::get_package_runtime_state(&package).await
}
