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
    fn container_display_formats_name_and_image() {
        let container = Container {
            name: "test-container".to_string(),
            image: "nginx:latest".to_string(),
            cmd: vec![],
            port_bindings: HashMap::new(),
            volume_bindings: vec![],
            file_bindings: vec![],
        };

        let output = format!("{container}");

        assert!(output.contains("- Name: test-container"));
        assert!(output.contains("  Image: nginx:latest"));
    }

    #[test]
    fn port_binding_serializes_with_camel_case_fields() {
        let binding = PortBinding {
            host_ip: Some("127.0.0.1".to_string()),
            host_port: Some("8080".to_string()),
        };

        let json = serde_json::to_value(&binding).unwrap();

        assert_eq!(json["hostIp"], "127.0.0.1");
        assert_eq!(json["hostPort"], "8080");
    }

    #[test]
    fn port_binding_handles_none_values() {
        let binding = PortBinding {
            host_ip: None,
            host_port: None,
        };

        let json = serde_json::to_string(&binding).unwrap();
        let deserialized: PortBinding = serde_json::from_str(&json).unwrap();

        assert!(deserialized.host_ip.is_none());
        assert!(deserialized.host_port.is_none());
    }

    #[test]
    fn binding_serializes_with_camel_case_fields() {
        let binding = Binding {
            source: "/host/path".to_string(),
            destination: "/container/path".to_string(),
            options: Some("ro".to_string()),
        };

        let json = serde_json::to_value(&binding).unwrap();

        assert_eq!(json["source"], "/host/path");
        assert_eq!(json["destination"], "/container/path");
        assert_eq!(json["options"], "ro");
    }

    #[test]
    fn container_roundtrips_through_json() {
        let mut port_bindings = HashMap::new();
        port_bindings.insert(
            "80/tcp".to_string(),
            vec![PortBinding {
                host_ip: Some("0.0.0.0".to_string()),
                host_port: Some("8080".to_string()),
            }],
        );

        let original = Container {
            name: "web-server".to_string(),
            image: "nginx:alpine".to_string(),
            cmd: vec!["nginx".to_string(), "-g".to_string()],
            port_bindings,
            volume_bindings: vec![Binding {
                source: "/data".to_string(),
                destination: "/usr/share/nginx/html".to_string(),
                options: Some("ro".to_string()),
            }],
            file_bindings: vec![],
        };

        let json = serde_json::to_string(&original).unwrap();
        let deserialized: Container = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.name, original.name);
        assert_eq!(deserialized.image, original.image);
        assert_eq!(deserialized.cmd.len(), 2);
        assert_eq!(deserialized.cmd[0], "nginx");
        assert_eq!(deserialized.port_bindings.len(), 1);
        assert_eq!(deserialized.volume_bindings.len(), 1);
        assert_eq!(deserialized.file_bindings.len(), 0);
    }

    #[test]
    fn container_serializes_complex_structure() {
        let mut port_bindings = HashMap::new();
        port_bindings.insert(
            "80/tcp".to_string(),
            vec![
                PortBinding {
                    host_ip: Some("127.0.0.1".to_string()),
                    host_port: Some("8080".to_string()),
                },
                PortBinding {
                    host_ip: None,
                    host_port: Some("8081".to_string()),
                },
            ],
        );

        let container = Container {
            name: "complex".to_string(),
            image: "app:v1".to_string(),
            cmd: vec!["start".to_string()],
            port_bindings,
            volume_bindings: vec![Binding {
                source: "/vol".to_string(),
                destination: "/data".to_string(),
                options: None,
            }],
            file_bindings: vec![Binding {
                source: "/config".to_string(),
                destination: "/etc/config".to_string(),
                options: Some("rw".to_string()),
            }],
        };

        let json = serde_json::to_value(&container).unwrap();

        assert_eq!(json["name"], "complex");
        assert_eq!(json["portBindings"]["80/tcp"].as_array().unwrap().len(), 2);
        assert_eq!(json["volumeBindings"].as_array().unwrap().len(), 1);
        assert_eq!(json["fileBindings"].as_array().unwrap().len(), 1);
    }
}
