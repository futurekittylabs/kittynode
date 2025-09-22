use crate::{
    domain::package::PackageRuntimeState,
    infra::package::{self, get_packages},
};
use eyre::{Result, WrapErr};
use std::collections::HashMap;

pub async fn get_packages_runtime_state(
    names: &[String],
) -> Result<HashMap<String, PackageRuntimeState>> {
    let catalog = get_packages().wrap_err("Failed to retrieve packages")?;
    let mut states = HashMap::new();

    for name in names {
        let package = catalog
            .get(name)
            .ok_or_else(|| eyre::eyre!("Package '{}' not found", name))?;
        let state = package::get_package_runtime_state(package).await?;
        states.insert(name.clone(), state);
    }

    Ok(states)
}
