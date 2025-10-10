use std::fs::{self, OpenOptions};
use std::io::{self, Write as _};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use bip39::{Language, Mnemonic, Seed as Bip39Seed};
use eth2_key_derivation::DerivedKey;
use eth2_keystore::{Keystore, KeystoreBuilder, keypair_from_secret};
use eth2_network_config::{Eth2NetworkConfig, HARDCODED_NET_NAMES};
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
    // Shared unix timestamp (seconds) used to pair filenames
    timestamp: u64,
    // Optional suffix paired with deposit filename when collisions occur
    suffix: Option<u32>,
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
        .wrap_err("Mnemonic phrase is invalid")?;
    let spec = load_chain_spec(&network)?;

    prepare_output_dir(&output_dir)?;

    // On non-Unix platforms, we cannot tighten POSIX filesystem permissions
    // for the output directory or keystore files. Inform the user once.
    #[cfg(not(unix))]
    {
        println!(
            "Warning: This platform does not support enforcing POSIX file permissions for keystores. Ensure the output directory is protected"
        );
    }

    // Use a single timestamp to pair deposit_data and keystore filenames
    let timestamp = secs_since_unix_epoch(SystemTime::now())?;
    // Choose a unique deposit_data filename and capture any suffix to pair with keystore names.
    let (deposit_data_path, suffix) = next_available_deposit_path(&output_dir, timestamp)?;

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
        timestamp,
        suffix,
    };

    for index in 0..validator_count {
        println!("  → Validator {} of {}", index + 1, validator_count);
        let _ = io::stdout().flush();

        let (keystore_path, deposit_entry) = produce_materials(index, &params)?;

        keystore_paths.push(keystore_path);
        deposits.push(deposit_entry);
    }

    write_json(&deposit_data_path, &deposits).wrap_err("Failed to write deposit data")?;

    Ok(KeygenOutcome {
        keystore_paths,
        deposit_data_path,
    })
}

fn produce_materials(index: u16, params: &GenerationParams<'_>) -> Result<(PathBuf, DepositEntry)> {
    let (secret_bytes, derivation_path) = derive_validator_secret(params.seed, index as u32)
        .map_err(|error| eyre!("Failed to derive validator secret {index}: {error}"))?;
    let secret_bytes = Zeroizing::new(secret_bytes);
    let keypair = keypair_from_secret(&secret_bytes)
        .map_err(|error| eyre!("Failed to instantiate keypair {index}: {error:?}"))?;
    drop(secret_bytes);

    let builder = KeystoreBuilder::new(&keypair, params.password.as_bytes(), derivation_path)
        .map_err(|error| eyre!("Failed to prepare keystore {index}: {error:?}"))?;

    #[cfg(test)]
    let builder = builder.kdf(fast_test_kdf());

    let keystore = builder
        .build()
        .map_err(|error| eyre!("Failed to finalize keystore {index}: {error:?}"))?;

    // Match ethstaker deposit-cli filename scheme and pair suffix with deposit_data when needed:
    // keystore-m_12381_3600_{index}_0_0-{timestamp}[-{suffix}].json
    let keystore_filename = match params.suffix {
        Some(n) => format!(
            "keystore-m_12381_3600_{}_0_0-{}-{}.json",
            index, params.timestamp, n
        ),
        None => format!(
            "keystore-m_12381_3600_{}_0_0-{}.json",
            index, params.timestamp
        ),
    };
    let keystore_path = params.output_dir.join(keystore_filename);
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
        .context("Network config name missing")?;

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

fn derive_validator_secret(seed: &[u8], index: u32) -> Result<(Vec<u8>, String)> {
    // EIP-2334 path m/12381/3600/{index}/0/0 for validator signing key
    let master = DerivedKey::from_seed(seed).map_err(|_| eyre!("empty seed provided"))?;
    let nodes = [12381u32, 3600, index, 0, 0];
    let dest = nodes.into_iter().fold(master, |dk, i| dk.child(i));
    let secret = dest.secret().to_vec();
    let path_str = format!("m/12381/3600/{index}/0/0");
    Ok((secret, path_str))
}

fn secs_since_unix_epoch(now: SystemTime) -> Result<u64> {
    Ok(now
        .duration_since(UNIX_EPOCH)
        .wrap_err("System time is invalid")?
        .as_secs())
}

#[cfg(test)]
fn fast_test_kdf() -> eth2_keystore::json_keystore::Kdf {
    use eth2_keystore::json_keystore::{HexBytes, Kdf, Pbkdf2, Prf};
    use eth2_keystore::{DKLEN, SALT_SIZE};
    // Fast, test-only PBKDF2 parameters to avoid scrypt cost in debug tests.
    Kdf::Pbkdf2(Pbkdf2 {
        dklen: DKLEN,
        c: 4096,
        prf: Prf::HmacSha256,
        salt: HexBytes::from(vec![0u8; SALT_SIZE]),
    })
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

    // Do not silently fall back to a different network. This would produce
    // deposit data and signatures for the wrong chain and could strand funds.
    let available = HARDCODED_NET_NAMES.join(", ");
    Err(eyre!(
        "Unsupported or unavailable network: {network}. Available in this Lighthouse build: {available}. \
Please upgrade Lighthouse (and this CLI if needed) if your desired network is missing"
    ))
}

fn build_spec(network: &str) -> Result<Option<ChainSpec>> {
    match Eth2NetworkConfig::constant(network) {
        Ok(Some(config)) => {
            Ok(Some(config.chain_spec::<MainnetEthSpec>().map_err(
                |error| eyre!("Failed to build chain spec for {network}: {error}"),
            )?))
        }
        Ok(None) => Ok(None),
        Err(error) => Err(eyre!("Failed to load network config {network}: {error}")),
    }
}

fn prepare_output_dir(path: &Path) -> Result<()> {
    if path.exists() {
        if !path.is_dir() {
            return Err(eyre!("Path must be a directory: {path:?}"));
        }
        // Enforce secure permissions on existing directories as well (Unix only).
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(path)
                .wrap_err_with(|| format!("Failed to stat {path:?}"))?
                .permissions();
            perms.set_mode(0o700);
            fs::set_permissions(path, perms)
                .wrap_err_with(|| format!("Failed to set permissions on {path:?}"))?;
        }
    } else {
        fs::create_dir_all(path).wrap_err_with(|| format!("Failed to create {path:?}"))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(path)
                .wrap_err_with(|| format!("Failed to stat {path:?}"))?
                .permissions();
            perms.set_mode(0o700);
            fs::set_permissions(path, perms)
                .wrap_err_with(|| format!("Failed to set permissions on {path:?}"))?;
        }
    }
    Ok(())
}

fn ensure_new_file(path: &Path) -> Result<()> {
    if path.exists() {
        return Err(eyre!("Refusing to overwrite existing file {path:?}"));
    }
    Ok(())
}

fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .wrap_err_with(|| format!("Failed to open {path:?}"))?;
    serde_json::to_writer(&mut file, value)
        .wrap_err_with(|| format!("Failed to serialize JSON to {path:?}"))
}

fn write_keystore(path: &Path, keystore: &Keystore) -> Result<()> {
    let mut open_opts = OpenOptions::new();
    open_opts.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        // Ensure keystore is created with owner-only read/write permissions.
        open_opts.mode(0o600);
    }
    let mut file = open_opts
        .open(path)
        .wrap_err_with(|| format!("Failed to open {path:?}"))?;

    let mut json = serde_json::to_value(keystore)
        .map_err(|error| eyre!("Failed to convert keystore to JSON value: {error:?}"))?;
    if let serde_json::Value::Object(ref mut map) = json {
        // Remove null 'name' field for compatibility with ethstaker deposit tool format
        map.retain(|key, value| !(key == "name" && value.is_null()));
    }

    serde_json::to_writer(&mut file, &json)
        .wrap_err_with(|| format!("Failed to serialize keystore JSON to {path:?}"))
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

fn next_available_deposit_path(
    output_dir: &Path,
    timestamp: u64,
) -> Result<(PathBuf, Option<u32>)> {
    let candidate = |suffix: Option<u32>| -> PathBuf {
        match suffix {
            Some(n) => output_dir.join(format!("deposit_data-{}-{}.json", timestamp, n)),
            None => output_dir.join(format!("deposit_data-{}.json", timestamp)),
        }
    };
    let path = candidate(None);
    if !path.exists() {
        return Ok((path, None));
    }
    for idx in 1..=1000 {
        let attempt = candidate(Some(idx));
        if !attempt.exists() {
            return Ok((attempt, Some(idx)));
        }
    }
    Err(eyre!("Unable to find available filename for deposit data"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use eth2_keystore::json_keystore::Kdf;
    use eth2_keystore::{Keystore, default_kdf};
    use eyre::{Result, eyre};
    use serde_json::Value;
    use std::fs;
    use std::time::Duration;
    use tempfile::tempdir;

    const MNEMONIC: &str = "upon pelican potato light kick symptom pioneer bridge wonder chief head citizen flip festival claw switch wear proud length zoo mercy foot repair ceiling";
    const KEYSTORE_PASSWORD: &str = "blackcatsarenotevil";
    const WITHDRAWAL_ADDRESS: &str = "0x48fe05daea0f8cc6958a72522db42b2edb3fda1a";

    #[test]
    fn generates_ethstaker_parity_outputs() -> Result<()> {
        let tmp = tempdir().wrap_err("failed to create temp dir")?;
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
            output_dir: tmp.path().join("keys"),
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

        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn generated_keystore_has_owner_only_permissions() -> Result<()> {
        use std::os::unix::fs::PermissionsExt;

        let tmp = tempdir().wrap_err("failed to create temp dir")?;
        let withdrawal_address: Address = WITHDRAWAL_ADDRESS
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
            output_dir: tmp.path().join("keys"),
        })?;

        let md = fs::metadata(&outcome.keystore_paths[0])
            .wrap_err("failed to stat generated keystore")?;
        let mode = md.permissions().mode() & 0o777;
        assert_eq!(
            mode, 0o600,
            "keystore should be created with 0o600 permissions"
        );
        Ok(())
    }

    #[test]
    fn default_keystore_kdf_is_scrypt() {
        let kdf = default_kdf(vec![0u8; 32]);
        match kdf {
            Kdf::Scrypt(_) => {}
            other => panic!("expected default KDF to be scrypt, got: {:?}", other),
        }
    }

    #[test]
    fn compounding_withdrawal_credentials_uses_prefix_byte() -> Result<()> {
        let spec = ChainSpec::mainnet();
        let address: Address = WITHDRAWAL_ADDRESS.parse()?;
        let creds = compounding_withdrawal_credentials(address, &spec);
        assert_eq!(creds.as_slice()[0], spec.compounding_withdrawal_prefix_byte);
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

    // Note: We intentionally avoid keystore decryption here to keep the test fast.
    // Pubkey equality combined with deposit data parity exercises the same derivation path
    // and signing behavior without expensive scrypt decrypts.

    #[test]
    fn selects_next_available_deposit_path_with_suffix() -> Result<()> {
        let tmp = tempdir().wrap_err("failed to create temp dir")?;
        let dir = tmp.path();
        let ts = 1_696_969_696u64;
        let base = dir.join(format!("deposit_data-{}.json", ts));
        fs::write(&base, b"{}").wrap_err("failed to create base deposit file")?;

        let (path, suffix) = next_available_deposit_path(dir, ts)?;
        assert_eq!(
            path,
            dir.join(format!("deposit_data-{}-1.json", ts)),
            "should choose -1 when base exists"
        );
        assert_eq!(suffix, Some(1));
        Ok(())
    }

    #[test]
    fn errors_after_1000_deposit_filename_collisions() -> Result<()> {
        let tmp = tempdir().wrap_err("failed to create temp dir")?;
        let dir = tmp.path();
        let ts = 1_700_000_000u64;
        // Create base plus 1..=1000 suffixed files
        fs::write(dir.join(format!("deposit_data-{}.json", ts)), b"{}")
            .wrap_err("failed to create base deposit file")?;
        for i in 1..=1000u32 {
            fs::write(dir.join(format!("deposit_data-{}-{}.json", ts, i)), b"{}")
                .wrap_err("failed to create suffixed deposit file")?;
        }

        let result = next_available_deposit_path(dir, ts);
        assert!(result.is_err(), "should error after 1000 attempts");
        Ok(())
    }

    #[test]
    fn secs_since_unix_epoch_errors_for_pre_epoch_time() {
        let t = UNIX_EPOCH - Duration::from_secs(1);
        assert!(secs_since_unix_epoch(t).is_err());
    }
}
