use std::convert::TryFrom;

use blst::min_pk::SecretKey as BlstSecretKey;
use eyre::{Result, eyre};
use serde::Serialize;
use sha2::{Digest, Sha256};
use ssz_rs::prelude::*;

use super::network::Network;

const DOMAIN_DEPOSIT: [u8; 4] = [0x03, 0x00, 0x00, 0x00];
const BLS_DST: &[u8] = b"BLS_SIG_BLS12381G2_XMD:SHA-256_SSWU_RO_NUL_";

#[derive(Default, SimpleSerialize)]
struct DepositMessage {
    pub pubkey: Vector<u8, 48>,
    pub withdrawal_credentials: Vector<u8, 32>,
    pub amount: u64,
}

#[derive(Default, SimpleSerialize)]
struct DepositData {
    pub pubkey: Vector<u8, 48>,
    pub withdrawal_credentials: Vector<u8, 32>,
    pub amount: u64,
    pub signature: Vector<u8, 96>,
}

#[derive(Default, SimpleSerialize)]
struct SigningData {
    pub object_root: Node,
    pub domain: Vector<u8, 32>,
}

#[derive(Clone, Serialize)]
pub struct DepositDataJson {
    pub pubkey: String,
    pub withdrawal_credentials: String,
    pub amount: u64,
    pub signature: String,
    pub fork_version: String,
    pub network_name: String,
    pub deposit_message_root: String,
    pub deposit_data_root: String,
    pub deposit_cli_version: String,
}

pub fn build(
    secret_key: &BlstSecretKey,
    public_key_bytes: &[u8; 48],
    withdrawal_credentials: [u8; 32],
    amount: u64,
    network: Network,
) -> Result<DepositDataJson> {
    let pubkey_vector = Vector::<u8, 48>::try_from(public_key_bytes.to_vec())
        .map_err(|_| eyre!("invalid public key length"))?;
    let credentials_vector = Vector::<u8, 32>::try_from(withdrawal_credentials.to_vec())
        .map_err(|_| eyre!("invalid withdrawal credentials length"))?;

    let mut deposit_message = DepositMessage {
        pubkey: pubkey_vector.clone(),
        withdrawal_credentials: credentials_vector.clone(),
        amount,
    };

    let deposit_message_root = deposit_message.hash_tree_root()?;
    let domain = compute_domain(network.fork_version());
    let mut signing_data = SigningData {
        object_root: deposit_message_root,
        domain: Vector::<u8, 32>::try_from(domain.to_vec()).expect("domain is 32 bytes"),
    };
    let signing_root = signing_data.hash_tree_root()?;

    let signature = secret_key
        .sign(signing_root.as_ref(), BLS_DST, &[])
        .to_bytes();

    let mut deposit_data = DepositData {
        pubkey: pubkey_vector,
        withdrawal_credentials: credentials_vector,
        amount,
        signature: Vector::<u8, 96>::try_from(signature.to_vec())
            .map_err(|_| eyre!("invalid signature length"))?,
    };

    let deposit_data_root = deposit_data.hash_tree_root()?;

    Ok(DepositDataJson {
        pubkey: hex::encode(public_key_bytes),
        withdrawal_credentials: hex::encode(withdrawal_credentials),
        amount,
        signature: hex::encode(signature),
        fork_version: hex::encode(network.fork_version()),
        network_name: network.as_str().to_string(),
        deposit_message_root: hex::encode(deposit_message_root.as_ref()),
        deposit_data_root: hex::encode(deposit_data_root.as_ref()),
        deposit_cli_version: network.deposit_cli_version().to_string(),
    })
}

pub fn compounding_withdrawal_credentials(public_key: &[u8; 48]) -> [u8; 32] {
    let hash = Sha256::digest(public_key);
    let mut credentials = [0u8; 32];
    credentials[0] = 0x02;
    credentials[1..].copy_from_slice(&hash[1..]);
    credentials
}

pub fn eth1_withdrawal_credentials(address: [u8; 20]) -> [u8; 32] {
    let mut credentials = [0u8; 32];
    credentials[0] = 0x01;
    credentials[12..].copy_from_slice(&address);
    credentials
}

fn compute_domain(fork_version: [u8; 4]) -> [u8; 32] {
    let mut domain = [0u8; 32];
    domain[..4].copy_from_slice(&DOMAIN_DEPOSIT);
    domain[4..8].copy_from_slice(&fork_version);
    domain
}
