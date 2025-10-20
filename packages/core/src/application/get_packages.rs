use crate::{domain::package::PackageState, infra::package};
use eyre::Result;
use std::collections::HashMap;
use tracing::warn;

/// Returns the full state for the requested packages.
pub async fn get_packages(names: &[&str]) -> Result<HashMap<String, PackageState>> {
    let catalog = package::get_package_catalog()?;
    let mut states = HashMap::new();
    for name in names {
        let Some(pkg) = catalog.get(*name) else {
            warn!("Package '{name}' not found in catalog, skipping");
            continue;
        };
        let state = package::get_package(pkg).await?;
        states.insert((*name).to_string(), state);
    }
    Ok(states)
}
