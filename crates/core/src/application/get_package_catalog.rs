use crate::domain::package::Package;
use eyre::Result;
use std::collections::HashMap;

/// Returns the package catalog available to install.
pub fn get_package_catalog() -> Result<HashMap<String, Package>> {
    crate::infra::package::get_package_catalog()
}
