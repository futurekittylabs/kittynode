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

/// Installation status derived from config presence and container existence.
///
/// - `Installed`: config exists and all declared containers exist
/// - `PartiallyInstalled`: any partial artifact (config or some containers)
/// - `NotInstalled`: clean state (no config and no declared containers present)
#[derive(Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum InstallStatus {
    NotInstalled,
    PartiallyInstalled,
    Installed,
}

/// Runtime status derived from Docker states of declared containers.
///
/// - `Running`: all declared containers are running
/// - `PartiallyRunning`: some, but not all, are running
/// - `NotRunning`: none are running (or none declared)
#[derive(Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RuntimeStatus {
    NotRunning,
    PartiallyRunning,
    Running,
}

/// Full package state.
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageState {
    pub install: InstallStatus,
    pub runtime: RuntimeStatus,
    pub config_present: bool,
    pub missing_containers: Vec<String>,
}
