use crate::{
    domain::package::{Package, PackageRuntimeState},
    infra::package::{self, get_packages},
};
use eyre::{Result, WrapErr};
use std::collections::HashMap;

pub async fn get_packages_runtime_state(
    names: &[String],
) -> Result<HashMap<String, PackageRuntimeState>> {
    let catalog = get_packages().wrap_err("Failed to retrieve packages")?;
    let requested = resolve_requested_packages(names, &catalog)?;
    let mut states = HashMap::with_capacity(requested.len());

    for (name, package) in requested {
        let state = package::get_package_runtime_state(package).await?;
        states.insert(name.clone(), state);
    }

    Ok(states)
}

fn resolve_requested_packages<'a>(
    names: &'a [String],
    catalog: &'a HashMap<String, Package>,
) -> Result<Vec<(&'a String, &'a Package)>> {
    let mut resolved = Vec::with_capacity(names.len());
    for name in names {
        let package = catalog
            .get(name)
            .ok_or_else(|| eyre::eyre!("Package '{}' not found", name))?;
        resolved.push((name, package));
    }
    Ok(resolved)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::package::{Package, PackageConfig};
    use crate::domain::container::Container;
    use std::collections::HashMap;

    fn make_package(name: &str) -> Package {
        Package {
            name: name.to_string(),
            description: format!("{name} package"),
            network_name: "local-testnet".into(),
            containers: vec![Container {
                name: format!("{name}-container"),
                image: "test-image:latest".into(),
                cmd: Vec::new(),
                port_bindings: HashMap::new(),
                volume_bindings: Vec::new(),
                file_bindings: Vec::new(),
            }],
            default_config: PackageConfig::default(),
        }
    }

    #[test]
    fn resolve_requested_packages_returns_in_request_order() {
        let mut catalog = HashMap::new();
        catalog.insert("alpha".into(), make_package("alpha"));
        catalog.insert("beta".into(), make_package("beta"));

        let names = vec!["alpha".to_string(), "beta".to_string()];
        let resolved =
            resolve_requested_packages(&names, &catalog).expect("resolution should succeed");

        assert_eq!(resolved.len(), 2);
        assert_eq!(resolved[0].0, &names[0]);
        assert_eq!(resolved[0].1.name(), "alpha");
        assert_eq!(resolved[1].0, &names[1]);
        assert_eq!(resolved[1].1.name(), "beta");
    }

    #[test]
    fn resolve_requested_packages_errors_when_package_missing() {
        let mut catalog = HashMap::new();
        catalog.insert("present".into(), make_package("present"));

        let names = vec!["missing".to_string()];
        let err = match resolve_requested_packages(&names, &catalog) {
            Ok(_) => panic!("missing package should err"),
            Err(err) => err,
        };

        assert!(
            err.to_string().contains("Package 'missing' not found"),
            "unexpected error message: {}",
            err
        );
    }

    #[test]
    fn resolve_requested_packages_preserves_duplicates() {
        let mut catalog = HashMap::new();
        catalog.insert("alpha".into(), make_package("alpha"));

        let names = vec!["alpha".to_string(), "alpha".to_string()];
        let resolved =
            resolve_requested_packages(&names, &catalog).expect("resolution should succeed");

        assert_eq!(resolved.len(), 2);
        assert!(std::ptr::eq(resolved[0].1, resolved[1].1));
    }
}
