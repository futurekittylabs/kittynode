use eyre::Result;
use std::collections::HashMap;

use crate::{
    domain::container::{Binding, Container, PortBinding},
    domain::package::{Package, PackageConfig, PackageDefinition},
    infra::{
        ephemery::{EPHEMERY_CHECKPOINT_URLS, EPHEMERY_NETWORK_NAME, ensure_ephemery_config},
        file::kittynode_path,
    },
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

        let ephemery = if network == EPHEMERY_NETWORK_NAME {
            Some(ensure_ephemery_config()?)
        } else {
            None
        };

        let mut reth_cmd = vec!["node".to_string(), "--chain".to_string()];
        if ephemery.is_some() {
            // For Ephemery, Reth expects a geth-style genesis JSON, not a parity chainspec.
            reth_cmd.push("/root/networks/ephemery/genesis.json".to_string());
        } else {
            reth_cmd.push(network.to_string());
        }
        reth_cmd.extend([
            "--metrics".to_string(),
            "0.0.0.0:9001".to_string(),
            "--authrpc.addr".to_string(),
            "0.0.0.0".to_string(),
            "--authrpc.port".to_string(),
            "8551".to_string(),
            "--authrpc.jwtsecret".to_string(),
            format!("/root/.local/share/reth/{network}/jwt.hex"),
        ]);
        if let Some(config) = &ephemery
            && !config.execution_bootnodes.is_empty()
        {
            reth_cmd.push("--bootnodes".to_string());
            reth_cmd.push(config.execution_bootnodes.join(","));
        }

        let mut reth_file_bindings = vec![Binding {
            source: jwt_path.display().to_string(),
            destination: format!("/root/.local/share/reth/{network}/jwt.hex"),
            options: Some("ro".to_string()),
        }];
        if let Some(config) = &ephemery {
            reth_file_bindings.push(Binding {
                source: config.metadata_dir.to_string_lossy().to_string(),
                destination: "/root/networks/ephemery".to_string(),
                options: Some("ro".to_string()),
            });
        }

        let mut lighthouse_cmd = vec!["lighthouse".to_string()];
        if ephemery.is_some() {
            lighthouse_cmd.push("--testnet-dir".to_string());
            lighthouse_cmd.push("/root/networks/ephemery".to_string());
        } else {
            lighthouse_cmd.push("--network".to_string());
            lighthouse_cmd.push(network.to_string());
        }
        lighthouse_cmd.extend([
            "beacon".to_string(),
            "--http".to_string(),
            "--http-address".to_string(),
            "0.0.0.0".to_string(),
        ]);
        if let Some(url) = ephemery
            .as_ref()
            .and_then(|_| EPHEMERY_CHECKPOINT_URLS.first().copied())
        {
            lighthouse_cmd.push("--checkpoint-sync-url".to_string());
            lighthouse_cmd.push(url.to_string());
        } else {
            let checkpoint_sync_url = if network == "mainnet" {
                "https://mainnet.checkpoint.sigp.io/"
            } else {
                "https://checkpoint-sync.hoodi.ethpandaops.io"
            };
            lighthouse_cmd.push("--checkpoint-sync-url".to_string());
            lighthouse_cmd.push(checkpoint_sync_url.to_string());
        }
        lighthouse_cmd.extend([
            "--execution-jwt".to_string(),
            format!("/root/.lighthouse/{network}/jwt.hex"),
            "--execution-endpoint".to_string(),
            "http://reth-node:8551".to_string(),
        ]);
        if let Some(config) = &ephemery
            && !config.consensus_bootnodes.is_empty()
        {
            // Lighthouse expects `--boot-nodes` (hyphenated), not `--bootnodes`.
            lighthouse_cmd.push("--boot-nodes".to_string());
            lighthouse_cmd.push(config.consensus_bootnodes.join(","));
        }

        let mut lighthouse_file_bindings = vec![
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
        ];
        if let Some(config) = &ephemery {
            lighthouse_file_bindings.push(Binding {
                source: config.metadata_dir.to_string_lossy().to_string(),
                destination: "/root/networks/ephemery".to_string(),
                options: Some("ro".to_string()),
            });
        }

        Ok(vec![
            Container {
                name: "reth-node".to_string(),
                image: "ghcr.io/paradigmxyz/reth".to_string(),
                cmd: reth_cmd,
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
                file_bindings: reth_file_bindings,
            },
            Container {
                name: "lighthouse-node".to_string(),
                image: "sigp/lighthouse".to_string(),
                cmd: lighthouse_cmd,
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
                file_bindings: lighthouse_file_bindings,
            },
        ])
    }
}
