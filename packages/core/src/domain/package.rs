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
#[serde(rename_all = "camelCase")]
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
#[serde(rename_all = "camelCase")]
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

impl Package {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn network_name(&self) -> &str {
        &self.network_name
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageRuntimeState {
    pub running: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn package_config_new_creates_empty_values() {
        let config = PackageConfig::new();

        assert_eq!(config.values.len(), 0);
    }

    #[test]
    fn package_config_default_creates_empty_values() {
        let config = PackageConfig::default();

        assert_eq!(config.values.len(), 0);
    }

    #[test]
    fn package_config_serializes_values_as_camel_case() {
        let mut config = PackageConfig::new();
        config
            .values
            .insert("key1".to_string(), "value1".to_string());
        config
            .values
            .insert("key2".to_string(), "value2".to_string());

        let json = serde_json::to_value(&config).unwrap();

        assert_eq!(json["values"]["key1"], "value1");
        assert_eq!(json["values"]["key2"], "value2");
    }

    #[test]
    fn package_config_roundtrips_through_json() {
        let mut original = PackageConfig::new();
        original.values.insert("foo".to_string(), "bar".to_string());

        let json = serde_json::to_string(&original).unwrap();
        let deserialized: PackageConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.values.len(), 1);
        assert_eq!(deserialized.values.get("foo"), Some(&"bar".to_string()));
    }

    #[test]
    fn package_accessors_return_correct_values() {
        let package = Package {
            name: "ethereum".to_string(),
            description: "Ethereum node".to_string(),
            network_name: "eth-network".to_string(),
            containers: vec![],
            default_config: PackageConfig::new(),
        };

        assert_eq!(package.name(), "ethereum");
        assert_eq!(package.description(), "Ethereum node");
        assert_eq!(package.network_name(), "eth-network");
    }

    #[test]
    fn package_display_formats_metadata() {
        let package = Package {
            name: "test-package".to_string(),
            description: "A test package".to_string(),
            network_name: "test-net".to_string(),
            containers: vec![],
            default_config: PackageConfig::new(),
        };

        let output = format!("{package}");

        assert!(output.contains("Package: test-package"));
        assert!(output.contains("Description: A test package"));
        assert!(output.contains("Containers:"));
    }

    #[test]
    fn package_display_includes_containers() {
        let package = Package {
            name: "web-app".to_string(),
            description: "Web application".to_string(),
            network_name: "web-net".to_string(),
            containers: vec![Container {
                name: "nginx".to_string(),
                image: "nginx:latest".to_string(),
                cmd: vec![],
                port_bindings: HashMap::new(),
                volume_bindings: vec![],
                file_bindings: vec![],
            }],
            default_config: PackageConfig::new(),
        };

        let output = format!("{package}");

        assert!(output.contains("Package: web-app"));
        assert!(output.contains("- Name: nginx"));
        assert!(output.contains("  Image: nginx:latest"));
    }

    #[test]
    fn package_serializes_with_camel_case_fields() {
        let package = Package {
            name: "app".to_string(),
            description: "desc".to_string(),
            network_name: "net".to_string(),
            containers: vec![],
            default_config: PackageConfig::new(),
        };

        let json = serde_json::to_value(&package).unwrap();

        assert_eq!(json["name"], "app");
        assert_eq!(json["description"], "desc");
        assert_eq!(json["networkName"], "net");
        assert!(json["containers"].is_array());
        assert!(json["defaultConfig"].is_object());
    }

    #[test]
    fn package_runtime_state_serializes_with_camel_case() {
        let state = PackageRuntimeState { running: true };

        let json = serde_json::to_value(&state).unwrap();

        assert_eq!(json["running"], true);
    }

    #[test]
    fn package_runtime_state_roundtrips_through_json() {
        let original = PackageRuntimeState { running: false };

        let json = serde_json::to_string(&original).unwrap();
        let deserialized: PackageRuntimeState = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.running, original.running);
    }
}
