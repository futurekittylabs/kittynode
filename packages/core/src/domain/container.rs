use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

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
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "- Name: {}", self.name)?;
        writeln!(f, "  Image: {}", self.image)
    }
}
