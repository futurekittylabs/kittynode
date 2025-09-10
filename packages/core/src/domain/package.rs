use crate::domain::container::Container;
use eyre::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

pub(crate) trait PackageDefinition {
    const NAME: &'static str;
    fn get_package() -> Result<Package>;
}

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct PackageConfig {
    pub values: HashMap<String, String>,
}

impl PackageConfig {
    #[must_use]
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Package {
    pub(crate) name: String,
    pub(crate) description: String,
    pub(crate) network_name: String,
    pub(crate) containers: Vec<Container>,
    pub(crate) default_config: PackageConfig,
}

impl fmt::Display for Package {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Package: {}", self.name)?;
        writeln!(f, "Description: {}", self.description)?;
        writeln!(f, "Containers:")?;
        for container in &self.containers {
            write!(f, "{container}")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::container::{Binding, Container};
    use bollard::models::PortBinding;
    use std::collections::HashMap;

    #[test]
    fn package_config_new_is_empty() {
        let cfg = PackageConfig::new();
        assert!(cfg.values.is_empty());
    }

    #[test]
    fn package_display_includes_containers() {
        let pkg = Package {
            name: "Sample".into(),
            description: "Example".into(),
            network_name: "net".into(),
            containers: vec![Container {
                name: "c1".into(),
                image: "alpine".into(),
                cmd: vec![],
                port_bindings: HashMap::from([(
                    "80/tcp".to_string(),
                    vec![PortBinding {
                        host_ip: Some("0.0.0.0".into()),
                        host_port: Some("80".into()),
                    }],
                )]),
                volume_bindings: vec![Binding {
                    source: "vol".into(),
                    destination: "/data".into(),
                    options: Some("ro".into()),
                }],
                file_bindings: vec![],
            }],
            default_config: PackageConfig::new(),
        };

        let s = format!("{}", pkg);
        assert!(s.contains("Package: Sample"));
        assert!(s.contains("Description: Example"));
        assert!(s.contains("Containers:"));
        assert!(s.contains("- Name: c1"));
        assert!(s.contains("Image: alpine"));
    }
}
