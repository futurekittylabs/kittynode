use crate::ethereum::{
    EPHEMERY_CHECKPOINT_URLS, EphemeryConfig, LIGHTHOUSE_DATA_DIR, LIGHTHOUSE_DATA_VOLUME,
    LIGHTHOUSE_VALIDATOR_CONTAINER_NAME,
};
use crate::packages::{Binding, Container, PortBinding};
use eyre::{Result, eyre};
use std::collections::HashMap;

const RETH_DATA_VOLUME: &str = "kittynode-rethdata";
const RETH_NODE_CONTAINER_NAME: &str = "kittynode-reth-node";
const LIGHTHOUSE_NODE_CONTAINER_NAME: &str = "kittynode-lighthouse-node";

pub(crate) struct EthereumResourcePaths {
    pub jwt_source_path: String,
}

pub(crate) fn build_ethereum_containers(
    network: &str,
    settings: &super::settings::EthereumSettings,
    resources: &EthereumResourcePaths,
    ephemery: Option<&EphemeryConfig>,
) -> Result<Vec<Container>> {
    let mut containers = Vec::new();

    if settings.runs_local_node() {
        containers.push(build_reth_container(network, resources, ephemery));
        containers.push(build_lighthouse_beacon_container(
            network, resources, ephemery,
        )?);
    }

    if let Some(validator) = &settings.validator {
        containers.push(build_lighthouse_validator_container(
            network, validator, settings, ephemery,
        ));
    }

    Ok(containers)
}

fn build_reth_container(
    network: &str,
    resources: &EthereumResourcePaths,
    ephemery: Option<&EphemeryConfig>,
) -> Container {
    let mut command = vec!["node".to_string(), "--chain".to_string()];
    if ephemery.is_some() {
        command.push("/root/networks/ephemery/genesis.json".to_string());
        command.push("--datadir".to_string());
        command.push(format!("/root/.local/share/reth/{network}"));
    } else {
        command.push(network.to_string());
    }
    command.extend([
        "--metrics".to_string(),
        "0.0.0.0:9001".to_string(),
        "--authrpc.addr".to_string(),
        "0.0.0.0".to_string(),
        "--authrpc.port".to_string(),
        "8551".to_string(),
        "--authrpc.jwtsecret".to_string(),
        format!("/root/.local/share/reth/{network}/jwt.hex"),
    ]);
    if let Some(ephemery) = ephemery
        && !ephemery.execution_bootnodes.is_empty()
    {
        command.push("--bootnodes".to_string());
        command.push(ephemery.execution_bootnodes.join(","));
    }

    Container {
        name: RETH_NODE_CONTAINER_NAME.to_string(),
        image: "ghcr.io/paradigmxyz/reth".to_string(),
        cmd: command,
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
            destination: format!("/root/.local/share/reth/{network}"),
            options: None,
        }],
        file_bindings: jwt_and_ephemery_bindings(
            &resources.jwt_source_path,
            &format!("/root/.local/share/reth/{network}/jwt.hex"),
            ephemery,
        ),
    }
}

fn build_lighthouse_beacon_container(
    network: &str,
    resources: &EthereumResourcePaths,
    ephemery: Option<&EphemeryConfig>,
) -> Result<Container> {
    let lighthouse_jwt_path = format!("{LIGHTHOUSE_DATA_DIR}/{network}/jwt.hex");

    let mut command = vec!["lighthouse".to_string()];
    if ephemery.is_some() {
        command.push("--testnet-dir".to_string());
        command.push("/root/networks/ephemery".to_string());
    } else {
        command.push("--network".to_string());
        command.push(network.to_string());
    }
    command.extend([
        "beacon".to_string(),
        "--http".to_string(),
        "--http-address".to_string(),
        "0.0.0.0".to_string(),
        "--checkpoint-sync-url".to_string(),
        checkpoint_sync_url(network, ephemery)?.to_string(),
        "--execution-jwt".to_string(),
        lighthouse_jwt_path.clone(),
        "--execution-endpoint".to_string(),
        format!("http://{RETH_NODE_CONTAINER_NAME}:8551"),
    ]);
    if let Some(ephemery) = ephemery
        && !ephemery.consensus_bootnodes.is_empty()
    {
        command.push("--boot-nodes".to_string());
        command.push(ephemery.consensus_bootnodes.join(","));
    }

    Ok(Container {
        name: LIGHTHOUSE_NODE_CONTAINER_NAME.to_string(),
        image: "sigp/lighthouse".to_string(),
        cmd: command,
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
        file_bindings: jwt_and_ephemery_bindings(
            &resources.jwt_source_path,
            &lighthouse_jwt_path,
            ephemery,
        ),
    })
}

fn build_lighthouse_validator_container(
    network: &str,
    validator: &super::settings::ValidatorSettings,
    settings: &super::settings::EthereumSettings,
    ephemery: Option<&EphemeryConfig>,
) -> Container {
    let mut command = vec!["lighthouse".to_string()];
    if ephemery.is_some() {
        command.push("--testnet-dir".to_string());
        command.push("/root/networks/ephemery".to_string());
    } else {
        command.push("--network".to_string());
        command.push(network.to_string());
    }

    let beacon_endpoint = settings
        .consensus_endpoint
        .clone()
        .unwrap_or_else(|| format!("http://{LIGHTHOUSE_NODE_CONTAINER_NAME}:5052"));

    command.extend([
        "vc".to_string(),
        "--beacon-nodes".to_string(),
        beacon_endpoint,
        "--suggested-fee-recipient".to_string(),
        validator.fee_recipient.clone(),
    ]);

    let mut file_bindings = Vec::new();
    if let Some(ephemery) = ephemery {
        file_bindings.push(ephemery_binding(ephemery));
    }

    Container {
        name: LIGHTHOUSE_VALIDATOR_CONTAINER_NAME.to_string(),
        image: "sigp/lighthouse".to_string(),
        cmd: command,
        port_bindings: HashMap::new(),
        volume_bindings: vec![Binding {
            source: LIGHTHOUSE_DATA_VOLUME.to_string(),
            destination: LIGHTHOUSE_DATA_DIR.to_string(),
            options: None,
        }],
        file_bindings,
    }
}

fn jwt_and_ephemery_bindings(
    jwt_source_path: &str,
    jwt_destination_path: &str,
    ephemery: Option<&EphemeryConfig>,
) -> Vec<Binding> {
    let mut bindings = vec![Binding {
        source: jwt_source_path.to_string(),
        destination: jwt_destination_path.to_string(),
        options: Some("ro".to_string()),
    }];
    if let Some(ephemery) = ephemery {
        bindings.push(ephemery_binding(ephemery));
    }
    bindings
}

fn ephemery_binding(ephemery: &EphemeryConfig) -> Binding {
    Binding {
        source: ephemery.metadata_dir.to_string_lossy().to_string(),
        destination: "/root/networks/ephemery".to_string(),
        options: Some("ro".to_string()),
    }
}

fn checkpoint_sync_url<'a>(network: &str, ephemery: Option<&'a EphemeryConfig>) -> Result<&'a str> {
    if ephemery.is_some() {
        return EPHEMERY_CHECKPOINT_URLS
            .first()
            .copied()
            .ok_or_else(|| eyre!("Ephemery checkpoint sync URL is missing"));
    }

    match network {
        "mainnet" => Ok("https://mainnet.checkpoint.sigp.io/"),
        "sepolia" => Ok("https://checkpoint-sync.sepolia.ethpandaops.io/"),
        "hoodi" => Ok("https://checkpoint-sync.hoodi.ethpandaops.io"),
        other => Err(eyre!("Unsupported network for checkpoint sync: {other}")),
    }
}
