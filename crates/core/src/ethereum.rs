#[path = "ethereum/containers.rs"]
mod containers;
#[path = "ethereum/ephemery.rs"]
mod ephemery;
#[path = "ethereum/settings.rs"]
mod settings;

pub use ephemery::{
    EPHEMERY_CHECKPOINT_URLS, EPHEMERY_NETWORK_NAME, EphemeryConfig, ensure_ephemery_config,
};

use crate::docker::{find_container, get_docker_instance};
use crate::packages::{Container, Package, PackageConfig, PackageConfigStore, PackageDefinition};
use crate::paths::{generate_jwt_secret, kittynode_path};
use eyre::{Context, Result, eyre};
use std::iter;

pub const ETHEREUM_EXECUTION_NETWORKS: &[&str] = &["hoodi", "mainnet", "sepolia"];

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
    supported_networks_iter().any(|value| value == network)
}

pub(crate) struct Ethereum;

const ETHEREUM_NAME: &str = "ethereum";
pub const LIGHTHOUSE_DATA_DIR: &str = "/root/.lighthouse";
pub const LIGHTHOUSE_DATA_VOLUME: &str = "kittynode-lighthouse-data";
const ETHEREUM_NETWORK_RESOURCE: &str = "kittynode-ethereum-network";
pub const LIGHTHOUSE_VALIDATOR_CONTAINER_NAME: &str = "kittynode-lighthouse-validator";

impl PackageDefinition for Ethereum {
    const NAME: &'static str = ETHEREUM_NAME;

    fn get_package() -> Result<Package> {
        let saved_config = PackageConfigStore::load(ETHEREUM_NAME)?;
        let containers = match settings::selected_network(&saved_config) {
            Some(network) => build_ethereum_package_containers(&saved_config, network)?,
            None => Vec::new(),
        };

        Ok(Package {
            name: ETHEREUM_NAME.to_string(),
            description: "This package installs an Ethereum node.".to_string(),
            network_name: ETHEREUM_NETWORK_RESOURCE.to_string(),
            default_config: PackageConfig::new(),
            containers,
        })
    }
}

fn build_ethereum_package_containers(
    config: &PackageConfig,
    network: &str,
) -> Result<Vec<Container>> {
    if !is_supported_network(network) {
        return Err(eyre!("Unsupported Ethereum network: {network}"));
    }

    let settings = settings::ethereum_settings_from_config(config);
    if settings.runs_local_node() {
        generate_jwt_secret(ETHEREUM_NAME)
            .wrap_err("Failed to ensure JWT secret for Ethereum package")?;
    }

    let package_root = PackageConfigStore::package_dir(&kittynode_path()?, ETHEREUM_NAME);
    let resources = containers::EthereumResourcePaths {
        jwt_source_path: package_root.join("jwt.hex").display().to_string(),
    };
    let ephemery = if network == EPHEMERY_NETWORK_NAME {
        Some(ensure_ephemery_config()?)
    } else {
        None
    };

    containers::build_ethereum_containers(network, &settings, &resources, ephemery.as_ref())
}

pub async fn is_validator_installed() -> Result<bool> {
    let docker = get_docker_instance().await?;
    let containers = find_container(&docker, LIGHTHOUSE_VALIDATOR_CONTAINER_NAME).await?;
    Ok(!containers.is_empty())
}
