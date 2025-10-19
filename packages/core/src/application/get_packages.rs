use crate::{domain::package::PackageState, infra::package};
use eyre::Result;
use std::collections::HashMap;

/// Returns the full state for the requested packages.
pub async fn get_packages(names: &[String]) -> Result<HashMap<String, PackageState>> {
    let mut states = HashMap::new();
    for name in names {
        let pkg = package::get_package_by_name(name)?;
        let state = package::get_package(&pkg).await?;
        states.insert(name.clone(), state);
    }
    Ok(states)
}
