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

    fn assert_cmd_value(cmd: &[String], flag: &str, expected: &str) {
        let mut iter = cmd.iter();
        while let Some(item) = iter.next() {
            if item == flag {
                let value = iter
                    .next()
                    .unwrap_or_else(|| panic!("missing value for flag '{flag}'"));
                assert_eq!(value, expected, "unexpected value for {flag}");
                return;
            }
        }
        panic!("flag '{flag}' not found in command: {cmd:?}");
    }

    #[test]
    fn get_containers_mainnet_sets_checkpoint_sync_url() {
        let containers = Ethereum::get_containers("mainnet").expect("failed to get containers");
        let lighthouse = containers
            .iter()
            .find(|container| container.name == "lighthouse-node")
            .expect("lighthouse container missing");
        assert_cmd_value(
            &lighthouse.cmd,
            "--checkpoint-sync-url",
            "https://mainnet.checkpoint.sigp.io/",
        );

        let jwt_binding = lighthouse
            .file_bindings
            .iter()
            .find(|binding| binding.destination.ends_with("jwt.hex"))
            .expect("jwt binding missing");
        let expected_jwt_path = kittynode_path()
            .expect("failed to resolve kittynode path")
            .join("jwt.hex")
            .display()
            .to_string();
        assert_eq!(jwt_binding.source, expected_jwt_path);

        let reth = containers
            .iter()
            .find(|container| container.name == "reth-node")
            .expect("reth container missing");
        assert_cmd_value(&reth.cmd, "--chain", "mainnet");
    }

    #[test]
    fn get_containers_non_mainnet_uses_default_checkpoint_url() {
        let containers = Ethereum::get_containers("hoodi").expect("failed to get containers");
        let lighthouse = containers
            .iter()
            .find(|container| container.name == "lighthouse-node")
            .expect("lighthouse container missing");
        assert_cmd_value(
            &lighthouse.cmd,
            "--checkpoint-sync-url",
            "https://checkpoint-sync.hoodi.ethpandaops.io",
        );

        let reth = containers
            .iter()
            .find(|container| container.name == "reth-node")
            .expect("reth container missing");
        let jwt_destination = reth
            .file_bindings
            .iter()
            .find(|binding| binding.destination.ends_with("jwt.hex"))
            .expect("jwt binding missing")
            .destination
            .clone();
        assert!(
            jwt_destination.contains("/hoodi/"),
            "expected jwt destination to include network segment, got {jwt_destination}"
        );
    }
}
