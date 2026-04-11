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

#[cfg(test)]
mod tests {
    use super::*;

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
}
