use eyre::Result;
use std::collections::HashMap;
use std::path::Path;

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
        Ok(Self::containers_for(network, kittynode_path.as_path()))
    }

    fn containers_for(network: &str, kittynode_path: &Path) -> Vec<Container> {
        let jwt_source = kittynode_path
            .join("jwt.hex")
            .to_string_lossy()
            .into_owned();
        let lighthouse_source = kittynode_path
            .join(".lighthouse")
            .to_string_lossy()
            .into_owned();
        let checkpoint_sync_url = Self::checkpoint_sync_url(network);

        vec![
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
                    source: jwt_source.clone(),
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
                    format!("/root/.lighthouse/{network}/jwt.hex"),
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
                        source: lighthouse_source,
                        destination: "/root/.lighthouse".to_string(),
                        options: None,
                    },
                    Binding {
                        source: jwt_source,
                        destination: format!("/root/.lighthouse/{network}/jwt.hex"),
                        options: Some("ro".to_string()),
                    },
                ],
            },
        ]
    }

    fn checkpoint_sync_url(network: &str) -> &'static str {
        if network == "mainnet" {
            "https://mainnet.checkpoint.sigp.io/"
        } else {
            "https://checkpoint-sync.hoodi.ethpandaops.io"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn containers_for_uses_expected_checkpoint_urls() {
        let temp = tempdir().unwrap();
        let containers = Ethereum::containers_for("mainnet", temp.path());
        let lighthouse = containers
            .iter()
            .find(|container| container.name == "lighthouse-node")
            .expect("missing lighthouse container");

        let checkpoint_flag_index = lighthouse
            .cmd
            .iter()
            .position(|arg| arg == "--checkpoint-sync-url")
            .expect("checkpoint flag missing");
        assert_eq!(
            lighthouse.cmd[checkpoint_flag_index + 1],
            "https://mainnet.checkpoint.sigp.io/"
        );

        let alt_containers = Ethereum::containers_for("hoodi", temp.path());
        let alt_lighthouse = alt_containers
            .iter()
            .find(|container| container.name == "lighthouse-node")
            .expect("missing lighthouse container");
        let alt_checkpoint_flag_index = alt_lighthouse
            .cmd
            .iter()
            .position(|arg| arg == "--checkpoint-sync-url")
            .expect("checkpoint flag missing");
        assert_eq!(
            alt_lighthouse.cmd[alt_checkpoint_flag_index + 1],
            "https://checkpoint-sync.hoodi.ethpandaops.io"
        );
    }

    #[test]
    fn containers_for_mounts_files_from_provided_kittynode_path() {
        let temp = tempdir().unwrap();
        let containers = Ethereum::containers_for("hoodi", temp.path());

        let reth = containers
            .iter()
            .find(|container| container.name == "reth-node")
            .expect("missing reth container");
        let lighthouse = containers
            .iter()
            .find(|container| container.name == "lighthouse-node")
            .expect("missing lighthouse container");

        let expected_jwt_source = temp
            .path()
            .join("jwt.hex")
            .to_string_lossy()
            .into_owned();
        let expected_lighthouse_source = temp
            .path()
            .join(".lighthouse")
            .to_string_lossy()
            .into_owned();

        assert!(reth.file_bindings.iter().any(|binding| {
            binding.source.as_str() == expected_jwt_source.as_str()
                && binding.destination.as_str() == "/root/.local/share/reth/hoodi/jwt.hex"
                && binding.options.as_deref() == Some("ro")
        }));

        assert!(lighthouse.file_bindings.iter().any(|binding| {
            binding.source.as_str() == expected_lighthouse_source.as_str()
                && binding.destination.as_str() == "/root/.lighthouse"
                && binding.options.is_none()
        }));

        assert!(lighthouse.file_bindings.iter().any(|binding| {
            binding.source.as_str() == expected_jwt_source.as_str()
                && binding.destination.as_str() == "/root/.lighthouse/hoodi/jwt.hex"
                && binding.options.as_deref() == Some("ro")
        }));
    }
}
