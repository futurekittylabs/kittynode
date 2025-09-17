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

#[derive(Clone, Serialize, Deserialize)]
pub struct InstalledPackage {
    pub package: Package,
    pub is_running: bool,
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
