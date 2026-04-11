use crate::{domain::package::PackageState, infra::package};
use eyre::Result;

/// Returns the full state for a single package.
pub async fn get_package(name: &str) -> Result<PackageState> {
    let pkg = package::get_package_by_name(name)?;
    package::get_package(&pkg).await
}
