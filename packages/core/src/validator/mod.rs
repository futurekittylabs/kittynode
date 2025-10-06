mod deposit;
mod derive;
mod keystore;
mod mnemonic;
mod network;

use std::ops::Range;

use bls_key_derivation::{derive_child, derive_master_sk, path_to_node};
use blst::min_pk::SecretKey as BlstSecretKey;
use deposit::DepositDataJson;
use derive::biguint_to_bytes32;
use eyre::{Result, eyre};
use hex::ToHex;
use keystore::{KeystoreRequest, KeystoreResult, encrypt_keystore};
pub use mnemonic::GeneratedMnemonic;
pub use network::Network;
use num_bigint::BigUint;
use zeroize::Zeroizing;

pub use deposit::{compounding_withdrawal_credentials, eth1_withdrawal_credentials};

const VALIDATOR_PATH_PREFIX: &str = "m/12381/3600";
const VOTING_PATH_SUFFIX: &str = "/0/0";
const WITHDRAWAL_PATH_SUFFIX: &str = "/0";

pub struct KeygenRequest<'a> {
    pub seed: &'a [u8],
    pub password: &'a str,
    pub validator_indices: Range<u32>,
    pub network: Network,
    pub deposit_amount_gwei: u64,
    pub withdrawal_address: Option<[u8; 20]>,
    pub compounding: bool,
}

pub struct ValidatorArtifacts {
    pub index: u32,
    pub public_key_hex: String,
    pub keystore: KeystoreResult,
    pub deposit: DepositDataJson,
    pub derivation_path: String,
}

pub fn generate_validators(request: KeygenRequest<'_>) -> Result<Vec<ValidatorArtifacts>> {
    let KeygenRequest {
        seed,
        password,
        validator_indices,
        network,
        deposit_amount_gwei,
        withdrawal_address,
        compounding,
    } = request;

    if compounding && withdrawal_address.is_some() {
        return Err(eyre!(
            "compounding validators must omit a withdrawal address"
        ));
    }
    if !compounding && withdrawal_address.is_none() {
        return Err(eyre!(
            "a withdrawal address is required when not using compounding credentials"
        ));
    }

    let master = derive_master_sk(seed).map_err(|error| eyre!(error))?;
    let count = validator_indices.clone().count();
    let mut results = Vec::with_capacity(count);

    for index in validator_indices {
        let voting_path = format!("{VALIDATOR_PATH_PREFIX}/{index}{VOTING_PATH_SUFFIX}");
        let voting_secret = derive_path(&master, &voting_path)?;
        let secret_key = BlstSecretKey::from_bytes(&voting_secret)
            .map_err(|_| eyre!("unable to parse derived validator key"))?;
        let public_key_bytes = secret_key.sk_to_pk().to_bytes();

        let withdrawal_credentials = if compounding {
            let withdrawal_path =
                format!("{VALIDATOR_PATH_PREFIX}/{index}{WITHDRAWAL_PATH_SUFFIX}");
            let withdrawal_secret = derive_path(&master, &withdrawal_path)?;
            let withdrawal_key = BlstSecretKey::from_bytes(&withdrawal_secret)
                .map_err(|_| eyre!("unable to parse derived withdrawal key"))?;
            let withdrawal_pubkey = withdrawal_key.sk_to_pk().to_bytes();
            compounding_withdrawal_credentials(&withdrawal_pubkey)
        } else {
            eth1_withdrawal_credentials(withdrawal_address.expect("checked above"))
        };

        let keystore = encrypt_keystore(KeystoreRequest {
            secret: Zeroizing::new(voting_secret),
            password,
            path: voting_path.clone(),
            public_key_hex: format!("0x{}", public_key_bytes.encode_hex::<String>()),
        })?;

        let deposit = deposit::build(
            &secret_key,
            &public_key_bytes,
            withdrawal_credentials,
            deposit_amount_gwei,
            network,
        )?;

        results.push(ValidatorArtifacts {
            index,
            public_key_hex: public_key_bytes.encode_hex::<String>(),
            keystore,
            deposit,
            derivation_path: voting_path,
        });
    }

    Ok(results)
}

fn derive_path(master: &BigUint, path: &str) -> Result<[u8; 32]> {
    let nodes = path_to_node(path.to_string()).map_err(|error| eyre!(error))?;
    let mut current = master.clone();
    for node in nodes {
        current = derive_child(current, node);
    }
    biguint_to_bytes32(&current)
}
