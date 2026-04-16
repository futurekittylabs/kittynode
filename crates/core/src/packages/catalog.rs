use crate::ethereum::Ethereum;
use crate::packages::{Package, PackageDefinition};
use eyre::{Result, eyre};
use std::collections::HashMap;

pub(crate) fn get_package_catalog() -> Result<HashMap<String, Package>> {
    let mut packages = HashMap::new();
    packages.insert(Ethereum::NAME.to_string(), Ethereum::get_package()?);
    Ok(packages)
}

pub(crate) fn get_package_by_name(name: &str) -> Result<Package> {
    let mut catalog = get_package_catalog()?;
    catalog
        .remove(name)
        .ok_or_else(|| eyre!("Package '{}' not found", name))
}

#[cfg(test)]
mod tests {
    use super::{get_package_by_name, get_package_catalog};

    #[test]
    fn get_package_by_name_is_case_sensitive_with_lowercase_canonical() {
        let catalog = get_package_catalog().expect("catalog should load");
        assert!(catalog.contains_key("ethereum"));
        assert!(get_package_by_name("ethereum").is_ok());
        assert!(get_package_by_name("Ethereum").is_err());
        assert!(get_package_by_name("does-not-exist").is_err());
    }
}
