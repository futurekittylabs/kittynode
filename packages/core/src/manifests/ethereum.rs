use eyre::Result;
use std::collections::HashMap;

use crate::{
    domain::container::{Binding, Container, PortBinding},
    domain::package::{Package, PackageConfig, PackageDefinition},
    infra::file::kittynode_path,
};

pub(crate) struct Ethereum;

const ETHEREUM_NAME: &str = "Ethereum";

impl PackageDefinition for Ethereum {
    const NAME: &'static str = ETHEREUM_NAME;

    fn get_package() -> Result<Package> {
        let mut default_config = PackageConfig::new();
        default_config
            .values
            .insert("network".to_string(), "hoodi".to_string());

        Ok(Package {
            name: ETHEREUM_NAME.to_string(),
            description: "This package installs an Ethereum node.".to_string(),
            network_name: "ethereum-network".to_string(),
            default_config,
            containers: Ethereum::get_containers("hoodi")?,
        })
    }
}

impl Ethereum {
    pub(crate) fn get_containers(network: &str) -> Result<Vec<Container>> {
        let kittynode_path = kittynode_path()?;
        let jwt_path = kittynode_path.join("jwt.hex");

        let checkpoint_sync_url = if network == "mainnet" {
            "https://mainnet.checkpoint.sigp.io/"
        } else {
            "https://checkpoint-sync.hoodi.ethpandaops.io"
        };

        Ok(vec![
            Container {
                name: "reth-node".to_string(),
                image: "ghcr.io/paradigmxyz/reth".to_string(),
                cmd: vec![
                    "node".to_string(),
                    "--chain".to_string(),
                    network.to_string(),
                    "--metrics".to_string(),
                    "0.0.0.0:9001".to_string(),
                    "--authrpc.addr".to_string(),
                    "0.0.0.0".to_string(),
                    "--authrpc.port".to_string(),
                    "8551".to_string(),
                ],
                port_bindings: HashMap::from([
                    (
                        "9001/tcp".to_string(),
                        vec![PortBinding {
                            host_ip: Some("0.0.0.0".to_string()),
                            host_port: Some("9001".to_string()),
                        }],
                    ),
                    (
                        "30303/tcp".to_string(),
                        vec![PortBinding {
                            host_ip: Some("0.0.0.0".to_string()),
                            host_port: Some("30303".to_string()),
                        }],
                    ),
                    (
                        "30303/udp".to_string(),
                        vec![PortBinding {
                            host_ip: Some("0.0.0.0".to_string()),
                            host_port: Some("30303".to_string()),
                        }],
                    ),
                ]),
                volume_bindings: vec![Binding {
                    source: "rethdata".to_string(),
                    destination: format!("/root/.local/share/reth/{network}"),
                    options: None,
                }],
                file_bindings: vec![Binding {
                    source: jwt_path.display().to_string(),
                    destination: format!("/root/.local/share/reth/{network}/jwt.hex"),
                    options: Some("ro".to_string()),
                }],
            },
            Container {
                name: "lighthouse-node".to_string(),
                image: "sigp/lighthouse".to_string(),
                cmd: vec![
                    "lighthouse".to_string(),
                    "--network".to_string(),
                    network.to_string(),
                    "beacon".to_string(),
                    "--http".to_string(),
                    "--http-address".to_string(),
                    "0.0.0.0".to_string(),
                    "--checkpoint-sync-url".to_string(),
                    checkpoint_sync_url.to_string(),
                    "--execution-jwt".to_string(),
                    format!("/root/.lighthouse/{network}/jwt.hex").to_string(),
                    "--execution-endpoint".to_string(),
                    "http://reth-node:8551".to_string(),
                ],
                port_bindings: HashMap::from([
                    (
                        "9000/tcp".to_string(),
                        vec![PortBinding {
                            host_ip: Some("0.0.0.0".to_string()),
                            host_port: Some("9000".to_string()),
                        }],
                    ),
                    (
                        "9000/udp".to_string(),
                        vec![PortBinding {
                            host_ip: Some("0.0.0.0".to_string()),
                            host_port: Some("9000".to_string()),
                        }],
                    ),
                    (
                        "9001/udp".to_string(),
                        vec![PortBinding {
                            host_ip: Some("0.0.0.0".to_string()),
                            host_port: Some("9001".to_string()),
                        }],
                    ),
                    (
                        "5052/tcp".to_string(),
                        vec![PortBinding {
                            host_ip: Some("127.0.0.1".to_string()),
                            host_port: Some("5052".to_string()),
                        }],
                    ),
                ]),
                volume_bindings: vec![],
                file_bindings: vec![
                    Binding {
                        source: kittynode_path
                            .join(".lighthouse")
                            .to_string_lossy()
                            .to_string(),
                        destination: "/root/.lighthouse".to_string(),
                        options: None,
                    },
                    Binding {
                        source: jwt_path.to_string_lossy().to_string(),
                        destination: format!("/root/.lighthouse/{network}/jwt.hex"),
                        options: Some("ro".to_string()),
                    },
                ],
            },
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infra::file::kittynode_path;

    fn container<'a>(containers: &'a [Container], name: &str) -> &'a Container {
        containers
            .iter()
            .find(|container| container.name == name)
            .unwrap_or_else(|| panic!("missing container {}", name))
    }

    fn flag_value<'a>(cmd: &'a [String], flag: &str) -> Option<&'a str> {
        cmd.iter()
            .position(|value| value == flag)
            .and_then(|idx| cmd.get(idx + 1))
            .map(String::as_str)
    }

    #[test]
    fn get_package_sets_expected_defaults() {
        let package = Ethereum::get_package().expect("package should be constructed");

        assert_eq!(package.name(), "Ethereum");
        assert_eq!(package.network_name(), "ethereum-network");
        assert_eq!(
            package.description(),
            "This package installs an Ethereum node."
        );

        let default_config = &package.default_config;
        let network = default_config
            .values
            .get("network")
            .expect("network should be present");
        assert_eq!(network, "hoodi");

        assert_eq!(package.containers.len(), 2);
        let reth = container(&package.containers, "reth-node");

        assert_eq!(flag_value(&reth.cmd, "--chain"), Some("hoodi"));
        assert!(reth.port_bindings.contains_key("30303/tcp"));
        assert!(reth.port_bindings.contains_key("30303/udp"));

        assert_eq!(reth.volume_bindings.len(), 1);
        assert_eq!(
            reth.volume_bindings[0].destination,
            "/root/.local/share/reth/hoodi"
        );
    }

    #[test]
    fn get_containers_apply_network_specific_configuration() {
        let containers =
            Ethereum::get_containers("mainnet").expect("mainnet containers should load");
        let reth = container(&containers, "reth-node");
        let lighthouse = container(&containers, "lighthouse-node");

        let expected_jwt_source = kittynode_path()
            .expect("home dir should resolve")
            .join("jwt.hex")
            .to_string_lossy()
            .to_string();

        let reth_jwt_binding = reth
            .file_bindings
            .iter()
            .find(|binding| binding.destination.ends_with("/mainnet/jwt.hex"))
            .expect("reth jwt binding");
        assert_eq!(reth_jwt_binding.source, expected_jwt_source);
        assert_eq!(reth_jwt_binding.options.as_deref(), Some("ro"));

        assert_eq!(
            flag_value(&lighthouse.cmd, "--checkpoint-sync-url"),
            Some("https://mainnet.checkpoint.sigp.io/")
        );

        let lighthouse_jwt_binding = lighthouse
            .file_bindings
            .iter()
            .find(|binding| binding.destination.ends_with("/mainnet/jwt.hex"))
            .expect("lighthouse jwt binding");
        assert_eq!(lighthouse_jwt_binding.source, expected_jwt_source);
        assert_eq!(lighthouse_jwt_binding.options.as_deref(), Some("ro"));
    }
}
