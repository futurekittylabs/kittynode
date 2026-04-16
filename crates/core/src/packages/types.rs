use eyre::Result;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt};

pub(crate) trait PackageDefinition {
    const NAME: &'static str;
    fn get_package() -> Result<Package>;
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Container {
    pub(crate) name: String,
    pub(crate) image: String,
    pub(crate) cmd: Vec<String>,
    pub(crate) port_bindings: HashMap<String, Vec<PortBinding>>,
    pub(crate) volume_bindings: Vec<Binding>,
    pub(crate) file_bindings: Vec<Binding>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PortBinding {
    pub(crate) host_ip: Option<String>,
    pub(crate) host_port: Option<String>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Binding {
    pub(crate) source: String,
    pub(crate) destination: String,
    pub(crate) options: Option<String>,
}

impl fmt::Display for Container {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(formatter, "- Name: {}", self.name)?;
        writeln!(formatter, "  Image: {}", self.image)
    }
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
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(formatter, "Package: {}", self.name)?;
        writeln!(formatter, "Description: {}", self.description)?;
        writeln!(formatter, "Containers:")?;
        for container in &self.containers {
            write!(formatter, "{container}")?;
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

#[derive(Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum InstallStatus {
    NotInstalled,
    PartiallyInstalled,
    Installed,
}

#[derive(Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RuntimeStatus {
    NotRunning,
    PartiallyRunning,
    Running,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageState {
    pub install: InstallStatus,
    pub runtime: RuntimeStatus,
    pub config_present: bool,
    pub missing_containers: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::{Binding, Container};
    use std::collections::HashMap;

    #[test]
    fn display_includes_name_and_image() {
        let container = Container {
            name: "kittynode-test".to_string(),
            image: "example/image:latest".to_string(),
            cmd: vec!["echo".to_string(), "hi".to_string()],
            port_bindings: HashMap::new(),
            volume_bindings: Vec::new(),
            file_bindings: Vec::new(),
        };

        assert_eq!(
            container.to_string(),
            "- Name: kittynode-test\n  Image: example/image:latest\n"
        );
    }

    #[test]
    fn binding_keeps_fields() {
        let binding = Binding {
            source: "/tmp/a".to_string(),
            destination: "/root/a".to_string(),
            options: Some("ro".to_string()),
        };
        assert_eq!(binding.source, "/tmp/a");
        assert_eq!(binding.destination, "/root/a");
        assert_eq!(binding.options.as_deref(), Some("ro"));
    }
}
