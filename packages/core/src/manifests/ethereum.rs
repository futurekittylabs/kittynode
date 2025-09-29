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

#[cfg(all(test, not(target_family = "wasm")))]
mod tests {
    use super::*;

    fn find_checkpoint_url(container: &Container) -> Option<&str> {
        container
            .cmd
            .windows(2)
            .find(|window| window[0] == "--checkpoint-sync-url")
            .map(|window| window[1].as_str())
    }

    fn find_jwt_binding(bindings: &[Binding]) -> Option<&Binding> {
        bindings
            .iter()
            .find(|binding| binding.destination.ends_with("/jwt.hex"))
    }

    #[test]
    fn mainnet_containers_use_mainnet_checkpoint() {
        let kittynode_root = kittynode_path().expect("expected kittynode path");
        let containers = Ethereum::get_containers("mainnet").expect("expected mainnet containers");

        let lighthouse = containers
            .iter()
            .find(|container| container.name == "lighthouse-node")
            .expect("lighthouse container missing");
        assert_eq!(
            find_checkpoint_url(lighthouse),
            Some("https://mainnet.checkpoint.sigp.io/")
        );

        let jwt_binding = find_jwt_binding(&lighthouse.file_bindings).expect("jwt binding missing");
        let expected_source = kittynode_root.join("jwt.hex").to_string_lossy().to_string();
        assert_eq!(jwt_binding.source, expected_source);

        let reth = containers
            .iter()
            .find(|container| container.name == "reth-node")
            .expect("reth container missing");
        assert!(
            reth.volume_bindings
                .iter()
                .any(|binding| binding.destination == "/root/.local/share/reth/mainnet"),
            "reth data directory should be namespaced to mainnet",
        );
    }

    #[test]
    fn non_mainnet_containers_use_testnet_checkpoint() {
        let kittynode_root = kittynode_path().expect("expected kittynode path");
        let network = "holesky";
        let containers =
            Ethereum::get_containers(network).expect("expected non-mainnet containers");

        let lighthouse = containers
            .iter()
            .find(|container| container.name == "lighthouse-node")
            .expect("lighthouse container missing");
        assert_eq!(
            find_checkpoint_url(lighthouse),
            Some("https://checkpoint-sync.hoodi.ethpandaops.io")
        );

        let jwt_binding = find_jwt_binding(&lighthouse.file_bindings).expect("jwt binding missing");
        assert_eq!(
            jwt_binding.destination,
            format!("/root/.lighthouse/{network}/jwt.hex")
        );

        let reth = containers
            .iter()
            .find(|container| container.name == "reth-node")
            .expect("reth container missing");
        let reth_binding = find_jwt_binding(&reth.file_bindings).expect("reth jwt binding missing");
        assert_eq!(
            reth_binding.destination,
            format!("/root/.local/share/reth/{network}/jwt.hex")
        );
        let expected_source = kittynode_root.join("jwt.hex").to_string_lossy().to_string();
        assert_eq!(reth_binding.source, expected_source);
    }
}
