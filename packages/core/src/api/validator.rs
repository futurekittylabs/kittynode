use eyre::{Result, eyre};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

const DEFAULT_KEY_FILENAME: &str = "validator_key.json";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidatorKey {
    pub public_key: String,
    pub secret_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DepositData {
    pub pubkey: String,
    pub withdrawal_credentials: String,
    pub amount: u64,
    pub signature: String,
    pub deposit_message_root: String,
    pub deposit_data_root: String,
    pub fork_version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network_name: Option<String>,
}

#[derive(Debug, Clone)]
pub struct GenerateKeysParams {
    pub output_dir: PathBuf,
    pub file_name: Option<String>,
    pub entropy: String,
    pub overwrite: bool,
}

impl GenerateKeysParams {
    pub fn new(output_dir: PathBuf, entropy: String) -> Self {
        Self {
            output_dir,
            file_name: None,
            entropy,
            overwrite: false,
        }
    }

    pub fn key_path(&self) -> PathBuf {
        self.output_dir.join(self.file_name())
    }

    fn file_name(&self) -> &str {
        self.file_name.as_deref().unwrap_or(DEFAULT_KEY_FILENAME)
    }
}

impl Default for GenerateKeysParams {
    fn default() -> Self {
        Self {
            output_dir: PathBuf::new(),
            file_name: None,
            entropy: String::new(),
            overwrite: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CreateDepositDataParams {
    pub key_path: PathBuf,
    pub output_path: PathBuf,
    pub withdrawal_credentials: String,
    pub amount_gwei: u64,
    pub fork_version: [u8; 4],
    pub genesis_validators_root: String,
    pub overwrite: bool,
    pub network_name: Option<String>,
}

#[derive(Clone, Copy)]
struct NetworkConfig {
    name: &'static str,
    fork_version: [u8; 4],
    genesis_validators_root: &'static str,
}

const MAINNET_CONFIG: NetworkConfig = NetworkConfig {
    name: "mainnet",
    fork_version: [0x00, 0x00, 0x00, 0x00],
    genesis_validators_root: "0x4b363db94e286120d76eb905340fdd4e54bfe9f06bf33ff6cf5ad27f511bfe95",
};

const SEPOLIA_CONFIG: NetworkConfig = NetworkConfig {
    name: "sepolia",
    fork_version: [0x90, 0x00, 0x00, 0x69],
    genesis_validators_root: "0xd8ea171f3c94aea21ebc42a1ed61052acf3f9209c00e4efbaaddac09ed9b8078",
};

const HOODI_CONFIG: NetworkConfig = NetworkConfig {
    name: "hoodi",
    fork_version: [0x10, 0x00, 0x09, 0x10],
    genesis_validators_root: "0x212f13fc4df078b6cb7db228f1c8307566dcecf900867401a92023d7ba99cb5f",
};

const NETWORK_CONFIGS: &[NetworkConfig] = &[MAINNET_CONFIG, SEPOLIA_CONFIG, HOODI_CONFIG];

impl CreateDepositDataParams {
    pub const SUPPORTED_NETWORKS: &'static [&'static str] =
        &[MAINNET_CONFIG.name, SEPOLIA_CONFIG.name, HOODI_CONFIG.name];

    pub fn for_network(
        key_path: PathBuf,
        output_path: PathBuf,
        withdrawal_address: &str,
        amount_gwei: u64,
        network: &str,
        overwrite: bool,
    ) -> Result<Self> {
        let config = resolve_network_config(network)?;
        let withdrawal_credentials = withdrawal_credentials_from_address(withdrawal_address)?;
        Ok(Self {
            key_path,
            output_path,
            withdrawal_credentials,
            amount_gwei,
            fork_version: config.fork_version,
            genesis_validators_root: config.genesis_validators_root.to_string(),
            overwrite,
            network_name: Some(config.name.to_string()),
        })
    }
}

fn resolve_network_config(input: &str) -> Result<NetworkConfig> {
    let normalized = input.trim().to_lowercase();
    NETWORK_CONFIGS
        .iter()
        .copied()
        .find(|config| config.name == normalized)
        .ok_or_else(|| {
            eyre!(
                "unsupported network name '{input}'. Expected one of {}",
                CreateDepositDataParams::SUPPORTED_NETWORKS.join(", ")
            )
        })
}

fn withdrawal_credentials_from_address(address: &str) -> Result<String> {
    let trimmed = address.trim();
    let without_prefix = trimmed
        .strip_prefix("0x")
        .or_else(|| trimmed.strip_prefix("0X"))
        .unwrap_or(trimmed);

    if without_prefix.len() != 40 {
        return Err(eyre!(
            "withdrawal address must be 20 bytes (40 hex characters), received {}",
            address
        ));
    }

    if !without_prefix.chars().all(|ch| ch.is_ascii_hexdigit()) {
        return Err(eyre!("withdrawal address must be hex-encoded"));
    }

    let normalized = without_prefix.to_lowercase();
    let mut credentials = String::with_capacity(64);
    credentials.push_str("02");
    credentials.push_str("0000000000000000000000");
    credentials.push_str(&normalized);
    Ok(credentials)
}

pub fn generate_keys(_params: GenerateKeysParams) -> Result<ValidatorKey> {
    Err(eyre!("validator key generation is no longer available"))
}

pub fn create_deposit_data(_params: CreateDepositDataParams) -> Result<DepositData> {
    Err(eyre!(
        "validator deposit data generation is no longer available"
    ))
}
