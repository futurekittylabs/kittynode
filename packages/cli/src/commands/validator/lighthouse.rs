use std::fs::{self, OpenOptions};
use std::io::{self, Write as _};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use eth2_keystore::{Keystore, KeystoreBuilder, keypair_from_secret};
use eth2_network_config::Eth2NetworkConfig;
use eth2_wallet::bip39::{Language, Mnemonic, Seed as Bip39Seed};
use eth2_wallet::{KeyType, recover_validator_secret_from_mnemonic};
use eyre::{ContextCompat, Result, WrapErr, eyre};
use hex::encode as hex_encode;
use serde::Serialize;
use tree_hash::TreeHash;
use types::{
    Address, ChainSpec, DepositData, Hash256, MainnetEthSpec, PublicKeyBytes, SignatureBytes,
    WithdrawalCredentials,
};
use zeroize::Zeroizing;

const DEPOSIT_CLI_VERSION: &str = "1.2.0";

struct GenerationParams<'a> {
    seed: &'a [u8],
    password: &'a Zeroizing<String>,
    deposit_gwei: u64,
    withdrawal_address: Address,
    spec: &'a ChainSpec,
    output_dir: &'a Path,
    compounding: bool,
}

pub struct KeygenConfig {
    pub mnemonic_phrase: Zeroizing<String>,
    pub validator_count: u16,
    pub withdrawal_address: Address,
    pub network: String,
    pub deposit_gwei: u64,
    pub compounding: bool,
    pub password: Zeroizing<String>,
    pub output_dir: PathBuf,
}

pub struct KeygenOutcome {
    pub keystore_paths: Vec<PathBuf>,
    pub deposit_data_path: PathBuf,
}

pub fn generate_validator_files(config: KeygenConfig) -> Result<KeygenOutcome> {
    let KeygenConfig {
        mnemonic_phrase,
        validator_count,
        withdrawal_address,
        network,
        deposit_gwei,
        compounding,
        password,
        output_dir,
    } = config;

    let mnemonic = Mnemonic::from_phrase(mnemonic_phrase.as_str(), Language::English)
        .map_err(|error| eyre!("mnemonic phrase is invalid: {error}"))?;
    let spec = load_chain_spec(&network)?;

    prepare_output_dir(&output_dir)?;
    let deposit_data_path = next_available_deposit_path(&output_dir)?;

    let seed = Bip39Seed::new(&mnemonic, "");

    let mut keystore_paths = Vec::with_capacity(validator_count as usize);
    let mut deposits = Vec::with_capacity(validator_count as usize);

    println!("Generating {validator_count} validator(s)...");
    let _ = io::stdout().flush();

    let params = GenerationParams {
        seed: seed.as_bytes(),
        password: &password,
        deposit_gwei,
        withdrawal_address,
        spec: &spec,
        output_dir: &output_dir,
        compounding,
    };

    for index in 0..validator_count {
        println!("  → Validator {} of {}", index + 1, validator_count);
        let _ = io::stdout().flush();

        let (keystore_path, deposit_entry) = produce_materials(index, &params)?;

        keystore_paths.push(keystore_path);
        deposits.push(deposit_entry);
    }

    write_json(&deposit_data_path, &deposits).wrap_err("failed to write deposit data")?;

    Ok(KeygenOutcome {
        keystore_paths,
        deposit_data_path,
    })
}

fn produce_materials(index: u16, params: &GenerationParams<'_>) -> Result<(PathBuf, DepositEntry)> {
    let (secret, path) =
        recover_validator_secret_from_mnemonic(params.seed, index as u32, KeyType::Voting)
            .map_err(|error| eyre!("failed to derive validator secret {index}: {error:?}"))?;
    let keypair = keypair_from_secret(secret.as_bytes())
        .map_err(|error| eyre!("failed to instantiate keypair {index}: {error:?}"))?;
    drop(secret);

    let derivation_path = path.to_string();
    let keystore = KeystoreBuilder::new(&keypair, params.password.as_bytes(), derivation_path)
        .map_err(|error| eyre!("failed to prepare keystore {index}: {error:?}"))?
        .build()
        .map_err(|error| eyre!("failed to finalize keystore {index}: {error:?}"))?;

    let keystore_path = params
        .output_dir
        .join(format!("keystore-{index:04}-{}.json", keystore.uuid()));
    ensure_new_file(&keystore_path)?;

    write_keystore(&keystore_path, &keystore)?;

    let withdrawal_credentials = if params.compounding {
        compounding_withdrawal_credentials(params.withdrawal_address, params.spec)
    } else {
        WithdrawalCredentials::eth1(params.withdrawal_address, params.spec).into()
    };

    let mut deposit_data = DepositData {
        pubkey: PublicKeyBytes::from(keypair.pk.clone()),
        withdrawal_credentials,
        amount: params.deposit_gwei,
        signature: SignatureBytes::empty(),
    };
    deposit_data.signature = deposit_data.create_signature(&keypair.sk, params.spec);

    let deposit_message_root = deposit_data.as_deposit_message().tree_hash_root();
    let deposit_data_root = deposit_data.tree_hash_root();

    let network_name = params
        .spec
        .config_name
        .clone()
        .context("network config name missing")?;

    let DepositData {
        pubkey,
        withdrawal_credentials,
        amount,
        signature,
    } = deposit_data;

    let deposit_entry = DepositEntry {
        pubkey: to_hex(pubkey.as_serialized()),
        withdrawal_credentials: to_hex(withdrawal_credentials.as_slice()),
        amount,
        signature: to_hex(signature.serialize()),
        deposit_message_root: to_hex(deposit_message_root.as_slice()),
        deposit_data_root: to_hex(deposit_data_root.as_slice()),
        fork_version: to_hex(params.spec.genesis_fork_version),
        network_name,
        deposit_cli_version: DEPOSIT_CLI_VERSION.to_string(),
    };

    Ok((keystore_path, deposit_entry))
}

#[derive(Serialize)]
struct DepositEntry {
    pub pubkey: String,
    pub withdrawal_credentials: String,
    pub amount: u64,
    pub signature: String,
    pub deposit_message_root: String,
    pub deposit_data_root: String,
    pub fork_version: String,
    pub network_name: String,
    pub deposit_cli_version: String,
}

fn load_chain_spec(network: &str) -> Result<ChainSpec> {
    if let Some(spec) = build_spec(network)? {
        return Ok(spec);
    }

    if network.eq_ignore_ascii_case("hoodi")
        && let Some(spec) = build_spec("holesky")?
    {
        println!("⚠️ Hoodi configuration not bundled in Lighthouse crates; using holesky spec");
        return Ok(spec);
    }

    Err(eyre!("unsupported network selection: {network}"))
}

fn build_spec(network: &str) -> Result<Option<ChainSpec>> {
    match Eth2NetworkConfig::constant(network) {
        Ok(Some(config)) => {
            Ok(Some(config.chain_spec::<MainnetEthSpec>().map_err(
                |error| eyre!("failed to build chain spec for {network}: {error}"),
            )?))
        }
        Ok(None) => Ok(None),
        Err(error) => Err(eyre!("failed to load network config {network}: {error}")),
    }
}

fn prepare_output_dir(path: &Path) -> Result<()> {
    if path.exists() {
        if !path.is_dir() {
            return Err(eyre!("{path:?} must be a directory"));
        }
    } else {
        fs::create_dir_all(path).wrap_err_with(|| format!("failed to create {path:?}"))?;
    }
    Ok(())
}

fn ensure_new_file(path: &Path) -> Result<()> {
    if path.exists() {
        return Err(eyre!("refusing to overwrite existing file {path:?}"));
    }
    Ok(())
}

fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .wrap_err_with(|| format!("failed to open {path:?}"))?;
    serde_json::to_writer(&mut file, value)
        .wrap_err_with(|| format!("failed to serialize JSON to {path:?}"))
}

fn write_keystore(path: &Path, keystore: &Keystore) -> Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .wrap_err_with(|| format!("failed to open {path:?}"))?;

    let mut json = serde_json::to_value(keystore)
        .map_err(|error| eyre!("failed to convert keystore to JSON value: {error:?}"))?;
    if let serde_json::Value::Object(ref mut map) = json {
        map.retain(|key, value| !(key == "name" && value.is_null()));
    }

    serde_json::to_writer(&mut file, &json)
        .wrap_err_with(|| format!("failed to serialize keystore JSON to {path:?}"))
}

fn to_hex(bytes: impl AsRef<[u8]>) -> String {
    hex_encode(bytes.as_ref())
}

fn compounding_withdrawal_credentials(address: Address, spec: &ChainSpec) -> Hash256 {
    let mut credentials = [0u8; 32];
    credentials[0] = spec.compounding_withdrawal_prefix_byte;
    credentials[12..].copy_from_slice(address.as_slice());
    Hash256::from_slice(&credentials)
}

fn next_available_deposit_path(output_dir: &Path) -> Result<PathBuf> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let candidate = |suffix: Option<u32>| -> PathBuf {
        match suffix {
            Some(n) => output_dir.join(format!("deposit_data-{}-{}.json", timestamp, n)),
            None => output_dir.join(format!("deposit_data-{}.json", timestamp)),
        }
    };
    let path = candidate(None);
    if !path.exists() {
        return Ok(path);
    }
    for idx in 1..u32::MAX {
        let attempt = candidate(Some(idx));
        if !attempt.exists() {
            return Ok(attempt);
        }
    }
    Err(eyre!("unable to find available filename for deposit data"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use eth2_keystore::Keystore;
    use eyre::{Result, eyre};
    use serde_json::Value;
    use std::fs;
    use tempfile::tempdir;

    const MNEMONIC: &str = "upon pelican potato light kick symptom pioneer bridge wonder chief head citizen flip festival claw switch wear proud length zoo mercy foot repair ceiling";
    const KEYSTORE_PASSWORD: &str = "blackcatsarenotevil";
    const WITHDRAWAL_ADDRESS: &str = "0x48fe05daea0f8cc6958a72522db42b2edb3fda1a";

    #[test]
    fn generates_ethstaker_parity_outputs() -> Result<()> {
        let output_dir = tempdir().wrap_err("failed to create temp dir")?;
        let withdrawal_address = WITHDRAWAL_ADDRESS
            .parse()
            .wrap_err("failed to parse withdrawal address")?;
        let outcome = generate_validator_files(KeygenConfig {
            mnemonic_phrase: Zeroizing::new(MNEMONIC.to_string()),
            validator_count: 1,
            withdrawal_address,
            network: "hoodi".to_string(),
            deposit_gwei: 32_000_000_000,
            compounding: true,
            password: Zeroizing::new(KEYSTORE_PASSWORD.to_string()),
            output_dir: output_dir.path().to_path_buf(),
        })?;

        let fixture_dir =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/ethstaker");
        let fixture_deposit = fixture_dir.join("deposit_data.json");
        let fixture_keystore = fixture_dir.join("keystore.json");

        let expected_deposits = read_json_array(&fixture_deposit)?;
        let mut actual_deposits = read_json_array(&outcome.deposit_data_path)?;
        scrub_deposit_entry(&mut actual_deposits);
        let mut expected_deposits = expected_deposits;
        scrub_deposit_entry(&mut expected_deposits);
        assert_eq!(actual_deposits, expected_deposits);

        assert_eq!(outcome.keystore_paths.len(), 1);
        let keystore_path = outcome.keystore_paths.first().unwrap();
        let actual_keystore = Keystore::from_json_file(keystore_path)
            .map_err(|error| eyre!("failed to parse generated keystore: {error:?}"))?;
        let expected_keystore = Keystore::from_json_file(&fixture_keystore)
            .map_err(|error| eyre!("failed to parse ethstaker keystore: {error:?}"))?;

        assert_eq!(
            actual_keystore.path().as_deref(),
            Some("m/12381/3600/0/0/0"),
            "keystore derivation path should match ethstaker output"
        );
        assert_eq!(
            actual_keystore.pubkey(),
            expected_keystore.pubkey(),
            "generated keystore pubkey should match ethstaker keystore"
        );

        verify_secret_key_parity(&actual_keystore, &expected_keystore)?;

        Ok(())
    }

    fn read_json_array(path: &Path) -> Result<Vec<Value>> {
        let file = fs::File::open(path).wrap_err_with(|| format!("failed to open {path:?}"))?;
        serde_json::from_reader(file)
            .wrap_err_with(|| format!("failed to parse JSON array from {path:?}"))
    }

    fn scrub_deposit_entry(entries: &mut [Value]) {
        for entry in entries {
            if let Value::Object(map) = entry {
                map.remove("deposit_cli_version");
            }
        }
    }

    fn verify_secret_key_parity(actual: &Keystore, expected: &Keystore) -> Result<()> {
        let actual_keypair = actual
            .decrypt_keypair(KEYSTORE_PASSWORD.as_bytes())
            .map_err(|error| eyre!("failed to decrypt generated keystore: {error:?}"))?;
        let expected_keypair = expected
            .decrypt_keypair(KEYSTORE_PASSWORD.as_bytes())
            .map_err(|error| eyre!("failed to decrypt ethstaker keystore: {error:?}"))?;

        if actual_keypair.pk != expected_keypair.pk {
            return Err(eyre!("public keys derived from keystores do not match"));
        }
        let actual_secret = actual_keypair.sk.serialize();
        let expected_secret = expected_keypair.sk.serialize();
        if actual_secret.as_bytes() != expected_secret.as_bytes() {
            return Err(eyre!("secret keys derived from keystores do not match"));
        }
        Ok(())
    }
}
