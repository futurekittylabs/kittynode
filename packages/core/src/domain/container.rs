use bollard::models::PortBinding;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

#[derive(Clone, Serialize, Deserialize)]
pub struct Container {
    pub(crate) name: String,
    pub(crate) image: String,
    pub(crate) cmd: Vec<String>,
    pub(crate) port_bindings: HashMap<String, Vec<PortBinding>>,
    pub(crate) volume_bindings: Vec<Binding>,
    pub(crate) file_bindings: Vec<Binding>,
}

#[derive(Clone, Serialize, Deserialize)]
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
    use bollard::models::PortBinding;
    use std::collections::HashMap;

    fn sample_container() -> Container {
        Container {
            name: "example".to_string(),
            image: "alpine:latest".to_string(),
            cmd: vec!["echo".into(), "hi".into()],
            port_bindings: HashMap::from([(
                "8080/tcp".to_string(),
                vec![PortBinding {
                    host_ip: Some("127.0.0.1".into()),
                    host_port: Some("8080".into()),
                }],
            )]),
            volume_bindings: vec![Binding {
                source: "/data".into(),
                destination: "/mnt".into(),
                options: None,
            }],
            file_bindings: vec![],
        }
    }

    #[test]
    fn display_shows_name_and_image() {
        let c = sample_container();
        let s = format!("{}", c);
        assert!(s.contains("- Name: example"), "output: {s}");
        assert!(s.contains("Image: alpine:latest"), "output: {s}");
    }
}
