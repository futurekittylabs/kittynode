use crate::{domain::package::PackageRuntimeState, infra::package};
use eyre::Result;
use std::collections::HashMap;

pub async fn get_packages_runtime_state(
    names: &[String],
) -> Result<HashMap<String, PackageRuntimeState>> {
    let mut states = HashMap::new();

    for name in names {
        let package = package::get_package_by_name(name)?;
        let state = package::get_package_runtime_state(&package).await?;
        states.insert(name.clone(), state);
    }

    Ok(states)
}
