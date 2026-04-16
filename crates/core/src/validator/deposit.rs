//! Minimal deposit data implementation for Ethereum validator key generation.
//!
//! Replaces the heavy `types` and `eth2_network_config` lighthouse crates with a thin,
//! self-contained module that implements only what kittynode needs: deposit data construction,
//! SSZ tree hashing, and BLS signing — all per the Ethereum consensus spec.

use alloy_primitives::{Address, B256};
use eyre::{Result, eyre};
use sha2::{Digest, Sha256};
use std::path::Path;

/// Re-export BLS types from the `bls` crate (already a transitive dep of `eth2_keystore`).
pub use bls::{PublicKeyBytes, SecretKey, SignatureBytes};

// ---------------------------------------------------------------------------
// Network configuration
// ---------------------------------------------------------------------------

/// Minimal chain spec — only the fields kittynode actually uses.
pub struct ChainSpec {
    pub genesis_fork_version: [u8; 4],
    pub config_name: Option<String>,
}

/// Hardcoded genesis fork versions per network. These are consensus-spec constants.
const NETWORKS: &[(&str, [u8; 4])] = &[
    ("mainnet", [0x00, 0x00, 0x00, 0x00]),
    ("gnosis", [0x00, 0x00, 0x00, 0x64]),
    ("chiado", [0x00, 0x00, 0x00, 0x6f]),
    ("sepolia", [0x90, 0x00, 0x00, 0x69]),
    ("holesky", [0x01, 0x01, 0x70, 0x00]),
    ("hoodi", [0x10, 0x00, 0x09, 0x10]),
];

/// Returns the list of built-in network names.
pub fn hardcoded_net_names() -> Vec<&'static str> {
    NETWORKS.iter().map(|(name, _)| *name).collect()
}

/// Looks up a built-in network by name and returns its [`ChainSpec`].
pub fn chain_spec_for_network(name: &str) -> Option<ChainSpec> {
    NETWORKS
        .iter()
        .find(|(n, _)| *n == name)
        .map(|(n, fork_version)| ChainSpec {
            genesis_fork_version: *fork_version,
            config_name: Some(n.to_string()),
        })
}

/// Extracts the scalar value from a YAML `KEY: value # comment` line,
/// stripping inline comments and surrounding whitespace.
fn yaml_value(raw: &str) -> &str {
    let trimmed = raw.trim();
    // YAML inline comments start with ` #` (space then hash).
    match trimmed.find(" #") {
        Some(pos) => trimmed[..pos].trim_end(),
        None => trimmed,
    }
}

/// Loads a [`ChainSpec`] from a directory containing a `config.yaml` file.
///
/// Only reads `GENESIS_FORK_VERSION` and `CONFIG_NAME` — everything else is ignored.
pub fn chain_spec_from_dir(path: &Path) -> Result<ChainSpec> {
    let config_path = path.join("config.yaml");
    let contents = std::fs::read_to_string(&config_path).map_err(|e| {
        eyre!(
            "Failed to read config.yaml from {}: {e}",
            config_path.display()
        )
    })?;

    let mut genesis_fork_version: Option<[u8; 4]> = None;
    let mut config_name: Option<String> = None;

    for line in contents.lines() {
        let line = line.trim();
        if let Some(value) = line.strip_prefix("GENESIS_FORK_VERSION:") {
            let hex = yaml_value(value).trim_start_matches("0x");
            let bytes =
                hex::decode(hex).map_err(|e| eyre!("Invalid GENESIS_FORK_VERSION hex: {e}"))?;
            genesis_fork_version = Some(
                bytes
                    .try_into()
                    .map_err(|_| eyre!("GENESIS_FORK_VERSION must be exactly 4 bytes"))?,
            );
        } else if let Some(value) = line.strip_prefix("CONFIG_NAME:") {
            config_name = Some(yaml_value(value).to_string());
        }
    }

    Ok(ChainSpec {
        genesis_fork_version: genesis_fork_version
            .ok_or_else(|| eyre!("GENESIS_FORK_VERSION not found in config.yaml"))?,
        config_name,
    })
}

// ---------------------------------------------------------------------------
// Withdrawal credentials
// ---------------------------------------------------------------------------

/// ETH1 address withdrawal prefix byte (0x01), per consensus spec.
const ETH1_ADDRESS_WITHDRAWAL_PREFIX: u8 = 0x01;

/// Compounding withdrawal prefix byte (0x02), per Electra spec.
const COMPOUNDING_WITHDRAWAL_PREFIX: u8 = 0x02;

/// Builds ETH1 withdrawal credentials: `0x01 || 11_zero_bytes || address`.
pub fn eth1_withdrawal_credentials(address: Address) -> B256 {
    let mut buf = [0u8; 32];
    buf[0] = ETH1_ADDRESS_WITHDRAWAL_PREFIX;
    buf[12..].copy_from_slice(address.as_slice());
    B256::from(buf)
}

/// Builds compounding withdrawal credentials: `0x02 || 11_zero_bytes || address`.
pub fn compounding_withdrawal_credentials(address: Address) -> B256 {
    let mut buf = [0u8; 32];
    buf[0] = COMPOUNDING_WITHDRAWAL_PREFIX;
    buf[12..].copy_from_slice(address.as_slice());
    B256::from(buf)
}

// ---------------------------------------------------------------------------
// SSZ tree hashing (only the fixed-size structs we need)
// ---------------------------------------------------------------------------

/// SHA-256 hash of two 32-byte chunks concatenated.
fn hash_concat(a: &[u8; 32], b: &[u8; 32]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(a);
    hasher.update(b);
    hasher.finalize().into()
}

/// SSZ hash tree root of a `uint64`: little-endian, zero-padded to 32 bytes.
fn hash_tree_root_u64(val: u64) -> [u8; 32] {
    let mut chunk = [0u8; 32];
    chunk[..8].copy_from_slice(&val.to_le_bytes());
    chunk
}

/// SSZ hash tree root of a 4-byte value: zero-padded to 32 bytes.
fn hash_tree_root_bytes4(val: &[u8; 4]) -> [u8; 32] {
    let mut chunk = [0u8; 32];
    chunk[..4].copy_from_slice(val);
    chunk
}

/// SSZ hash tree root of a 48-byte value (BLS pubkey): split into two 32-byte chunks.
fn hash_tree_root_48(bytes: &[u8; 48]) -> [u8; 32] {
    let mut chunk0 = [0u8; 32];
    let mut chunk1 = [0u8; 32];
    chunk0.copy_from_slice(&bytes[..32]);
    chunk1[..16].copy_from_slice(&bytes[32..]);
    hash_concat(&chunk0, &chunk1)
}

/// SSZ hash tree root of a 96-byte value (BLS signature): split into 3 chunks, padded to 4.
fn hash_tree_root_96(bytes: &[u8; 96]) -> [u8; 32] {
    let mut chunk0 = [0u8; 32];
    let mut chunk1 = [0u8; 32];
    let mut chunk2 = [0u8; 32];
    let chunk3 = [0u8; 32];
    chunk0.copy_from_slice(&bytes[..32]);
    chunk1.copy_from_slice(&bytes[32..64]);
    chunk2.copy_from_slice(&bytes[64..]);
    let h01 = hash_concat(&chunk0, &chunk1);
    let h23 = hash_concat(&chunk2, &chunk3);
    hash_concat(&h01, &h23)
}

// ---------------------------------------------------------------------------
// Deposit types and signing
// ---------------------------------------------------------------------------

/// Domain type constant for deposits, per consensus spec.
const DOMAIN_DEPOSIT: [u8; 4] = [0x03, 0x00, 0x00, 0x00];

/// Mirrors the consensus-spec `DepositMessage` container.
///
/// Fields (in SSZ order): `pubkey` (48), `withdrawal_credentials` (32), `amount` (u64).
struct DepositMessage {
    pubkey: [u8; 48],
    withdrawal_credentials: [u8; 32],
    amount: u64,
}

impl DepositMessage {
    /// SSZ hash tree root: 3 fields padded to 4 leaves.
    fn tree_hash_root(&self) -> [u8; 32] {
        let leaf0 = hash_tree_root_48(&self.pubkey);
        let leaf1 = self.withdrawal_credentials;
        let leaf2 = hash_tree_root_u64(self.amount);
        let leaf3 = [0u8; 32]; // padding to next power of 2
        let h01 = hash_concat(&leaf0, &leaf1);
        let h23 = hash_concat(&leaf2, &leaf3);
        hash_concat(&h01, &h23)
    }
}

/// The full deposit data container, matching the consensus-spec `DepositData`.
///
/// Fields (in SSZ order): `pubkey` (48), `withdrawal_credentials` (32), `amount` (u64),
/// `signature` (96).
pub struct DepositData {
    pub pubkey: [u8; 48],
    pub withdrawal_credentials: B256,
    pub amount: u64,
    pub signature: [u8; 96],
}

impl DepositData {
    /// Creates a new `DepositData` with an empty (zeroed) signature.
    pub fn new(pubkey: [u8; 48], withdrawal_credentials: B256, amount: u64) -> Self {
        Self {
            pubkey,
            withdrawal_credentials,
            amount,
            signature: [0u8; 96],
        }
    }

    /// SSZ hash tree root: 4 fields, already a power of 2.
    pub fn tree_hash_root(&self) -> [u8; 32] {
        let leaf0 = hash_tree_root_48(&self.pubkey);
        let leaf1: [u8; 32] = self.withdrawal_credentials.into();
        let leaf2 = hash_tree_root_u64(self.amount);
        let leaf3 = hash_tree_root_96(&self.signature);
        let h01 = hash_concat(&leaf0, &leaf1);
        let h23 = hash_concat(&leaf2, &leaf3);
        hash_concat(&h01, &h23)
    }

    /// Returns the `DepositMessage` (everything except the signature).
    fn as_deposit_message(&self) -> DepositMessage {
        DepositMessage {
            pubkey: self.pubkey,
            withdrawal_credentials: self.withdrawal_credentials.into(),
            amount: self.amount,
        }
    }

    /// Signs this deposit data per the consensus spec and writes the signature.
    ///
    /// Computes: `signing_root(DepositMessage, deposit_domain)` then BLS-signs it.
    pub fn sign(&mut self, secret_key: &SecretKey, spec: &ChainSpec) {
        let domain = compute_deposit_domain(spec.genesis_fork_version);
        let message_root = self.as_deposit_message().tree_hash_root();
        let signing_root = compute_signing_root(&message_root, &domain);
        let sig = secret_key.sign(B256::from(signing_root));
        self.signature = SignatureBytes::from(sig).serialize();
    }

    /// Returns the `DepositMessage` tree hash root (used in deposit data JSON output).
    pub fn deposit_message_root(&self) -> [u8; 32] {
        self.as_deposit_message().tree_hash_root()
    }
}

/// Computes the deposit domain for a given fork version.
///
/// `domain = DOMAIN_DEPOSIT || fork_data_root[0..28]`
///
/// For deposits, `genesis_validators_root` is always zero.
fn compute_deposit_domain(fork_version: [u8; 4]) -> [u8; 32] {
    let fork_data_root = compute_fork_data_root(fork_version);
    let mut domain = [0u8; 32];
    domain[..4].copy_from_slice(&DOMAIN_DEPOSIT);
    domain[4..].copy_from_slice(&fork_data_root[..28]);
    domain
}

/// Tree hash root of `ForkData { current_version, genesis_validators_root }`.
///
/// For deposits, `genesis_validators_root` is always zero.
fn compute_fork_data_root(fork_version: [u8; 4]) -> [u8; 32] {
    let leaf0 = hash_tree_root_bytes4(&fork_version);
    let leaf1 = [0u8; 32]; // genesis_validators_root = zero for deposits
    hash_concat(&leaf0, &leaf1)
}

/// `signing_root = hash_tree_root(SigningData { object_root, domain })`
///
/// Both fields are 32 bytes, so this is just `sha256(object_root || domain)`.
fn compute_signing_root(object_root: &[u8; 32], domain: &[u8; 32]) -> [u8; 32] {
    hash_concat(object_root, domain)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_hardcoded_networks_have_specs() {
        for name in hardcoded_net_names() {
            assert!(
                chain_spec_for_network(name).is_some(),
                "missing spec for {name}"
            );
        }
    }

    #[test]
    fn mainnet_fork_version_is_correct() {
        let spec = chain_spec_for_network("mainnet").unwrap();
        assert_eq!(spec.genesis_fork_version, [0x00, 0x00, 0x00, 0x00]);
    }

    #[test]
    fn eth1_withdrawal_credentials_format() {
        let addr: Address = "0x48fe05daea0f8cc6958a72522db42b2edb3fda1a"
            .parse()
            .unwrap();
        let creds = eth1_withdrawal_credentials(addr);
        assert_eq!(creds[0], 0x01);
        assert_eq!(&creds[1..12], &[0u8; 11]);
        assert_eq!(&creds[12..], addr.as_slice());
    }

    #[test]
    fn compounding_withdrawal_credentials_format() {
        let addr: Address = "0x48fe05daea0f8cc6958a72522db42b2edb3fda1a"
            .parse()
            .unwrap();
        let creds = compounding_withdrawal_credentials(addr);
        assert_eq!(creds[0], 0x02);
        assert_eq!(&creds[1..12], &[0u8; 11]);
        assert_eq!(&creds[12..], addr.as_slice());
    }

    #[test]
    fn yaml_value_strips_inline_comments() {
        assert_eq!(yaml_value(" testnet # ephemery rotation"), "testnet");
        assert_eq!(yaml_value(" 0x10000910 # hoodi"), "0x10000910");
        assert_eq!(yaml_value(" plain_value"), "plain_value");
        assert_eq!(yaml_value(" hash#tag"), "hash#tag"); // no space before #
    }

    #[test]
    fn chain_spec_from_dir_strips_yaml_comments() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(
            tmp.path().join("config.yaml"),
            "GENESIS_FORK_VERSION: 0x10000910 # hoodi\nCONFIG_NAME: testnet # ephemery\n",
        )
        .unwrap();
        let spec = chain_spec_from_dir(tmp.path()).unwrap();
        assert_eq!(spec.genesis_fork_version, [0x10, 0x00, 0x09, 0x10]);
        assert_eq!(spec.config_name.as_deref(), Some("testnet"));
    }

    #[test]
    fn signing_root_is_sha256_of_root_and_domain() {
        let root = [0xAA; 32];
        let domain = [0xBB; 32];
        let result = compute_signing_root(&root, &domain);
        // Verify it's SHA-256(root || domain)
        let mut hasher = Sha256::new();
        hasher.update(root);
        hasher.update(domain);
        let expected: [u8; 32] = hasher.finalize().into();
        assert_eq!(result, expected);
    }
}
