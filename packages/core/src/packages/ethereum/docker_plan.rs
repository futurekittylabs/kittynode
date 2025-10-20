use std::{collections::HashMap, iter};

use eyre::{ContextCompat, Result, eyre};

use crate::{
    domain::container::{Binding, Container, PortBinding},
    domain::package::{Package, PackageConfig},
    infra::{
        ephemery::{EPHEMERY_CHECKPOINT_URLS, EPHEMERY_NETWORK_NAME},
        file::kittynode_path,
    },
};

use super::config::{EthereumConfig, Network};

pub const ETHEREUM_NAME: &str = "ethereum";
pub const ETHEREUM_NETWORK_RESOURCE: &str = "kittynode-ethereum-network";
pub const RETH_NODE_CONTAINER_NAME: &str = "kittynode-reth-node";
pub const LIGHTHOUSE_NODE_CONTAINER_NAME: &str = "kittynode-lighthouse-node";
pub const LIGHTHOUSE_VALIDATOR_CONTAINER_NAME: &str = "kittynode-lighthouse-validator";
pub const LIGHTHOUSE_DATA_DIR: &str = "/root/.lighthouse";
pub const LIGHTHOUSE_DATA_VOLUME: &str = "kittynode-lighthouse-data";

const RETH_DATA_VOLUME: &str = "kittynode-rethdata";
const ETHEREUM_EXECUTION_NETWORKS: &[&str] = &["hoodi", "mainnet", "sepolia"];

pub fn supported_networks_iter() -> impl Iterator<Item = &'static str> {
    ETHEREUM_EXECUTION_NETWORKS
        .iter()
        .copied()
        .chain(iter::once(EPHEMERY_NETWORK_NAME))
}

pub fn supported_networks_display(delimiter: &str) -> String {
    supported_networks_iter()
        .collect::<Vec<_>>()
        .join(delimiter)
}

pub fn is_supported_network(network: &str) -> bool {
    supported_networks_iter().any(|candidate| candidate == network)
}

pub fn build_package(cfg: &EthereumConfig) -> Result<Package> {
    cfg.validate()?;

    let base_dir = kittynode_path()?;
    let package_dir = base_dir.join("packages").join(ETHEREUM_NAME);
    let jwt_path = package_dir.join("jwt.hex");
    let jwt_source = jwt_path
        .to_str()
        .map(|s| s.to_string())
        .ok_or_else(|| eyre!("Failed to convert JWT path to string"))?;

    let ephemery_metadata_path = base_dir
        .join("networks")
        .join(EPHEMERY_NETWORK_NAME)
        .join("current")
        .join("metadata");
    let ephemery_metadata = ephemery_metadata_path
        .to_str()
        .map(|s| s.to_string())
        .ok_or_else(|| eyre!("Failed to convert Ephemery metadata path to string"))?;

    let containers = build_containers(cfg, &jwt_source, &ephemery_metadata)?;

    Ok(Package {
        name: ETHEREUM_NAME.to_string(),
        description: "This package installs an Ethereum node.".to_string(),
        network_name: ETHEREUM_NETWORK_RESOURCE.to_string(),
        containers,
        default_config: PackageConfig::default(),
    })
}

fn build_containers(
    cfg: &EthereumConfig,
    jwt_source: &str,
    ephemery_metadata: &str,
) -> Result<Vec<Container>> {
    let network = cfg.network;
    let network_name = network.as_str();

    let reth_cmd = build_reth_cmd(network);
    let reth_file_bindings = build_reth_file_bindings(network, jwt_source, ephemery_metadata)?;
    let reth_container = Container {
        name: RETH_NODE_CONTAINER_NAME.to_string(),
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
            source: RETH_DATA_VOLUME.to_string(),
            destination: format!("/root/.local/share/reth/{network_name}"),
            options: None,
        }],
        file_bindings: reth_file_bindings,
    };

    let lighthouse_cmd = build_lighthouse_cmd(network)?;
    let lighthouse_file_bindings =
        build_lighthouse_file_bindings(network, jwt_source, ephemery_metadata)?;
    let lighthouse_container = Container {
        name: LIGHTHOUSE_NODE_CONTAINER_NAME.to_string(),
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
        volume_bindings: vec![Binding {
            source: LIGHTHOUSE_DATA_VOLUME.to_string(),
            destination: LIGHTHOUSE_DATA_DIR.to_string(),
            options: None,
        }],
        file_bindings: lighthouse_file_bindings,
    };

    let mut containers = vec![reth_container, lighthouse_container];

    if cfg.validator.enabled {
        let fee_recipient = cfg.validator.fee_recipient.as_ref().ok_or_else(|| {
            eyre!("Validator fee recipient missing despite validator being enabled")
        })?;
        let validator_cmd = build_validator_cmd(network, fee_recipient);
        let validator_file_bindings = build_validator_file_bindings(network, ephemery_metadata);
        containers.push(Container {
            name: LIGHTHOUSE_VALIDATOR_CONTAINER_NAME.to_string(),
            image: "sigp/lighthouse".to_string(),
            cmd: validator_cmd,
            port_bindings: HashMap::new(),
            volume_bindings: vec![Binding {
                source: LIGHTHOUSE_DATA_VOLUME.to_string(),
                destination: LIGHTHOUSE_DATA_DIR.to_string(),
                options: None,
            }],
            file_bindings: validator_file_bindings,
        });
    }

    Ok(containers)
}

fn build_reth_cmd(network: Network) -> Vec<String> {
    let network_name = network.as_str();
    let mut cmd = vec!["node".to_string(), "--chain".to_string()];
    if matches!(network, Network::Ephemery) {
        cmd.push("/root/networks/ephemery/genesis.json".to_string());
        cmd.push("--datadir".to_string());
        cmd.push(format!("/root/.local/share/reth/{network_name}"));
    } else {
        cmd.push(network_name.to_string());
    }
    cmd.extend([
        "--metrics".to_string(),
        "0.0.0.0:9001".to_string(),
        "--authrpc.addr".to_string(),
        "0.0.0.0".to_string(),
        "--authrpc.port".to_string(),
        "8551".to_string(),
        "--authrpc.jwtsecret".to_string(),
        format!("/root/.local/share/reth/{network_name}/jwt.hex"),
    ]);
    cmd
}

fn build_reth_file_bindings(
    network: Network,
    jwt_source: &str,
    ephemery_metadata: &str,
) -> Result<Vec<Binding>> {
    let network_name = network.as_str();
    let mut bindings = vec![Binding {
        source: jwt_source.to_string(),
        destination: format!("/root/.local/share/reth/{network_name}/jwt.hex"),
        options: Some("ro".to_string()),
    }];

    if matches!(network, Network::Ephemery) {
        bindings.push(ephemery_metadata_binding(ephemery_metadata));
    }

    Ok(bindings)
}

fn build_lighthouse_cmd(network: Network) -> Result<Vec<String>> {
    let network_name = network.as_str();
    let mut cmd = vec!["lighthouse".to_string()];
    if matches!(network, Network::Ephemery) {
        cmd.push("--testnet-dir".to_string());
        cmd.push("/root/networks/ephemery".to_string());
    } else {
        cmd.push("--network".to_string());
        cmd.push(network_name.to_string());
    }
    cmd.extend([
        "beacon".to_string(),
        "--http".to_string(),
        "--http-address".to_string(),
        "0.0.0.0".to_string(),
    ]);

    let checkpoint_url = match network {
        Network::Mainnet => "https://mainnet.checkpoint.sigp.io/".to_string(),
        Network::Sepolia => "https://checkpoint-sync.sepolia.ethpandaops.io/".to_string(),
        Network::Hoodi => "https://checkpoint-sync.hoodi.ethpandaops.io".to_string(),
        Network::Ephemery => EPHEMERY_CHECKPOINT_URLS
            .first()
            .copied()
            .context("Ephemery checkpoint list is empty")?
            .to_string(),
    };
    cmd.push("--checkpoint-sync-url".to_string());
    cmd.push(checkpoint_url);

    cmd.extend([
        "--execution-jwt".to_string(),
        format!("{LIGHTHOUSE_DATA_DIR}/{network_name}/jwt.hex"),
        "--execution-endpoint".to_string(),
        format!("http://{RETH_NODE_CONTAINER_NAME}:8551"),
    ]);
    Ok(cmd)
}

fn build_lighthouse_file_bindings(
    network: Network,
    jwt_source: &str,
    ephemery_metadata: &str,
) -> Result<Vec<Binding>> {
    let network_name = network.as_str();
    let mut bindings = vec![Binding {
        source: jwt_source.to_string(),
        destination: format!("{LIGHTHOUSE_DATA_DIR}/{network_name}/jwt.hex"),
        options: Some("ro".to_string()),
    }];

    if matches!(network, Network::Ephemery) {
        bindings.push(ephemery_metadata_binding(ephemery_metadata));
    }

    Ok(bindings)
}

fn build_validator_cmd(network: Network, fee_recipient: &str) -> Vec<String> {
    let network_name = network.as_str();
    let mut cmd = vec!["lighthouse".to_string()];
    if matches!(network, Network::Ephemery) {
        cmd.push("--testnet-dir".to_string());
        cmd.push("/root/networks/ephemery".to_string());
    } else {
        cmd.push("--network".to_string());
        cmd.push(network_name.to_string());
    }
    cmd.extend([
        "vc".to_string(),
        "--beacon-nodes".to_string(),
        format!("http://{LIGHTHOUSE_NODE_CONTAINER_NAME}:5052"),
        "--suggested-fee-recipient".to_string(),
        fee_recipient.to_string(),
    ]);
    cmd
}

fn build_validator_file_bindings(network: Network, ephemery_metadata: &str) -> Vec<Binding> {
    if matches!(network, Network::Ephemery) {
        vec![ephemery_metadata_binding(ephemery_metadata)]
    } else {
        Vec::new()
    }
}

fn ephemery_metadata_binding(source: &str) -> Binding {
    Binding {
        source: source.to_string(),
        destination: "/root/networks/ephemery".to_string(),
        options: Some("ro".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::packages::ethereum::config::Validator;

    fn jwt_path_string() -> String {
        let base = kittynode_path().expect("kittynode path should resolve");
        base.join("packages")
            .join(ETHEREUM_NAME)
            .join("jwt.hex")
            .to_string_lossy()
            .into_owned()
    }

    #[test]
    fn build_package_mainnet_produces_execution_and_beacon_containers() {
        let cfg = EthereumConfig {
            network: Network::Mainnet,
            validator: Validator::default(),
        };
        let package = build_package(&cfg).expect("plan should build");
        assert_eq!(package.name, ETHEREUM_NAME);
        assert_eq!(package.containers.len(), 2);

        let reth = package
            .containers
            .iter()
            .find(|c| c.name == RETH_NODE_CONTAINER_NAME)
            .expect("reth container missing");
        assert_eq!(reth.image, "ghcr.io/paradigmxyz/reth");
        assert!(
            reth.cmd
                .windows(2)
                .any(|pair| pair == ["--chain", "mainnet"]),
            "reth command should target mainnet: {:?}",
            reth.cmd
        );
        assert!(
            reth.file_bindings
                .iter()
                .any(|binding| binding.source == jwt_path_string()),
            "reth should mount jwt hex"
        );

        let lighthouse = package
            .containers
            .iter()
            .find(|c| c.name == LIGHTHOUSE_NODE_CONTAINER_NAME)
            .expect("lighthouse container missing");
        assert!(
            lighthouse
                .cmd
                .windows(2)
                .any(|pair| pair == ["--network", "mainnet"]),
            "lighthouse command should include --network mainnet: {:?}",
            lighthouse.cmd
        );
        assert!(
            lighthouse
                .cmd
                .contains(&"--checkpoint-sync-url".to_string()),
            "checkpoint flag missing from lighthouse command"
        );
        assert_eq!(
            lighthouse
                .cmd
                .iter()
                .filter(|arg| arg.as_str() == "--boot-nodes")
                .count(),
            0,
            "plan builder should not inject boot node flags"
        );
    }

    #[test]
    fn build_package_ephemery_includes_validator_and_metadata_mount() {
        let cfg = EthereumConfig {
            network: Network::Ephemery,
            validator: Validator {
                enabled: true,
                fee_recipient: Some("0x1234".to_string()),
            },
        };
        let package = build_package(&cfg).expect("plan should build");
        assert_eq!(package.containers.len(), 3);

        let reth = package
            .containers
            .iter()
            .find(|c| c.name == RETH_NODE_CONTAINER_NAME)
            .expect("reth container missing");
        assert!(
            reth.cmd
                .windows(2)
                .any(|pair| pair == ["--chain", "/root/networks/ephemery/genesis.json"]),
            "reth should use ephemery genesis: {:?}",
            reth.cmd
        );
        assert!(
            !reth.cmd.iter().any(|arg| arg == "--bootnodes"),
            "plan should be pure and omit runtime bootnodes"
        );
        assert!(
            reth.file_bindings
                .iter()
                .any(|binding| binding.destination == "/root/networks/ephemery"),
            "reth should mount ephemery metadata"
        );

        let lighthouse = package
            .containers
            .iter()
            .find(|c| c.name == LIGHTHOUSE_NODE_CONTAINER_NAME)
            .expect("lighthouse container missing");
        assert!(
            lighthouse
                .cmd
                .windows(2)
                .any(|pair| pair == ["--testnet-dir", "/root/networks/ephemery"]),
            "lighthouse should use ephemery testnet dir: {:?}",
            lighthouse.cmd
        );
        assert_eq!(
            lighthouse
                .cmd
                .iter()
                .filter(|arg| arg.as_str() == "--boot-nodes")
                .count(),
            0,
            "plan builder should omit boot node flags"
        );
        let checkpoint_index = lighthouse
            .cmd
            .iter()
            .position(|arg| arg == "--checkpoint-sync-url")
            .expect("checkpoint flag missing");
        assert_eq!(
            lighthouse.cmd[checkpoint_index + 1],
            EPHEMERY_CHECKPOINT_URLS[0],
            "plan should use canonical ephemery checkpoint URL"
        );
        assert_eq!(
            lighthouse
                .cmd
                .last()
                .expect("lighthouse command should end with URL"),
            &format!("http://{RETH_NODE_CONTAINER_NAME}:8551")
        );

        let validator = package
            .containers
            .iter()
            .find(|c| c.name == LIGHTHOUSE_VALIDATOR_CONTAINER_NAME)
            .expect("validator container missing");
        assert!(
            validator
                .cmd
                .windows(2)
                .any(|pair| pair == ["--suggested-fee-recipient", "0x1234"]),
            "validator should pass fee recipient: {:?}",
            validator.cmd
        );
        assert!(
            validator
                .file_bindings
                .iter()
                .any(|binding| binding.destination == "/root/networks/ephemery"),
            "validator should mount ephemery metadata"
        );
    }
}
