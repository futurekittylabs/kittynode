use super::filesystem::ensure_parent_secure;
use super::ports::{CryptoProvider, ValidatorFilesystem};
use crate::domain::validator::DepositData;
use crate::infra::validator::{SimpleCryptoProvider, StdValidatorFilesystem};
use eyre::{Context, Result, eyre};
use std::path::PathBuf;
use tracing::info;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

pub fn create_deposit_data(params: CreateDepositDataParams) -> Result<DepositData> {
    let crypto = SimpleCryptoProvider;
    let filesystem = StdValidatorFilesystem;
    create_deposit_data_with(params, &crypto, &filesystem)
}

pub fn create_deposit_data_with<P, F>(
    params: CreateDepositDataParams,
    crypto: &P,
    filesystem: &F,
) -> Result<DepositData>
where
    P: CryptoProvider,
    F: ValidatorFilesystem,
{
    let key = filesystem
        .read_json_secure(&params.key_path)
        .with_context(|| {
            format!(
                "failed to read validator key from {}",
                params.key_path.display()
            )
        })?;

    ensure_parent_secure(
        &params.output_path,
        filesystem,
        "invalid validator artifact directory",
    )?;

    let mut deposit = crypto
        .create_deposit_data(
            &key,
            &params.withdrawal_credentials,
            params.amount_gwei,
            params.fork_version,
            &params.genesis_validators_root,
        )
        .context("failed to build deposit data")?;

    if deposit.network_name.is_none() {
        deposit.network_name = params.network_name.clone();
    }

    if let Some(ref network) = deposit.network_name {
        info!("Creating deposit data for network {network}");
    }

    let deposit_slice = std::slice::from_ref(&deposit);
    filesystem
        .write_json_secure(&params.output_path, deposit_slice, params.overwrite)
        .context("failed to write deposit data")?;

    Ok(deposit)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::validator::ports::{CryptoProvider, ValidatorFilesystem};
    use crate::application::validator::{GenerateKeysParams, generate_keys};
    use crate::domain::validator::{DepositData, ValidatorKey};
    use eyre::Result;
    use serde::{Serialize, de::DeserializeOwned};
    use std::collections::HashMap;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::Mutex;
    use tempfile::tempdir;

    #[derive(Default)]
    struct TestFilesystem {
        files: Mutex<HashMap<PathBuf, String>>,
        insecure_dirs: Mutex<Vec<PathBuf>>,
        insecure_files: Mutex<Vec<PathBuf>>,
    }

    impl ValidatorFilesystem for TestFilesystem {
        fn ensure_secure_directory(&self, path: &Path) -> Result<()> {
            if self
                .insecure_dirs
                .lock()
                .expect("mutex poisoned")
                .iter()
                .any(|p| p == path)
            {
                eyre::bail!("directory has insecure permissions: {}", path.display());
            }
            Ok(())
        }

        fn write_json_secure<T: Serialize + ?Sized>(
            &self,
            path: &Path,
            value: &T,
            _overwrite: bool,
        ) -> Result<()> {
            let json = serde_json::to_string(value)?;
            self.files
                .lock()
                .expect("mutex poisoned")
                .insert(path.to_path_buf(), json);
            Ok(())
        }

        fn read_json_secure<T: DeserializeOwned>(&self, path: &Path) -> Result<T> {
            if self
                .insecure_files
                .lock()
                .expect("mutex poisoned")
                .iter()
                .any(|p| p == path)
            {
                eyre::bail!("insecure file permissions: {}", path.display());
            }
            let files = self.files.lock().expect("mutex poisoned");
            let contents = files
                .get(path)
                .ok_or_else(|| eyre::eyre!("missing file: {}", path.display()))?;
            Ok(serde_json::from_str(contents)?)
        }
    }

    #[derive(Default)]
    struct TestCrypto;

    impl CryptoProvider for TestCrypto {
        fn generate_key(&self, _entropy: &str) -> Result<ValidatorKey> {
            unreachable!("not used in create_deposit_data tests")
        }

        fn create_deposit_data(
            &self,
            key: &ValidatorKey,
            withdrawal_credentials: &str,
            amount_gwei: u64,
            _fork_version: [u8; 4],
            _genesis_validators_root: &str,
        ) -> Result<DepositData> {
            Ok(DepositData {
                pubkey: key.public_key.clone(),
                withdrawal_credentials: withdrawal_credentials.to_string(),
                amount: amount_gwei,
                signature: "sig".into(),
                deposit_message_root: "msg_root".into(),
                deposit_data_root: "data_root".into(),
                fork_version: "00000000".into(),
                network_name: None,
            })
        }
    }

    #[test]
    fn builds_deposit_and_writes_file() {
        let fs = TestFilesystem::default();
        let crypto = TestCrypto;
        let key_path = PathBuf::from("/validators/key.json");
        let deposit_path = PathBuf::from("/validators/deposit.json");

        let key = ValidatorKey {
            public_key: "pub".into(),
            secret_key: "sec".into(),
        };
        fs.write_json_secure(&key_path, &key, true).unwrap();

        let params = CreateDepositDataParams {
            key_path: key_path.clone(),
            output_path: deposit_path.clone(),
            withdrawal_credentials: "cred".into(),
            amount_gwei: 32_000_000_000,
            fork_version: [0, 0, 0, 0],
            genesis_validators_root:
                "0x000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f".into(),
            overwrite: false,
            network_name: None,
        };

        let deposit = create_deposit_data_with(params, &crypto, &fs).unwrap();
        assert_eq!(deposit.pubkey, "pub");

        let stored: Vec<DepositData> = fs.read_json_secure(&deposit_path).unwrap();
        assert_eq!(stored.len(), 1);
        assert_eq!(stored[0].signature, "sig");
    }

    #[test]
    fn rejects_insecure_key_file() {
        let fs = TestFilesystem::default();
        let crypto = TestCrypto;
        let key_path = PathBuf::from("/validators/key.json");
        fs.insecure_files
            .lock()
            .expect("mutex poisoned")
            .push(key_path.clone());

        let params = CreateDepositDataParams {
            key_path: key_path.clone(),
            output_path: PathBuf::from("/validators/deposit.json"),
            withdrawal_credentials: "cred".into(),
            amount_gwei: 32_000_000_000,
            fork_version: [0, 0, 0, 0],
            genesis_validators_root:
                "0x000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f".into(),
            overwrite: false,
            network_name: None,
        };

        let result = create_deposit_data_with(params, &crypto, &fs);
        assert!(result.is_err());
    }

    #[test]
    fn default_flow_writes_to_disk() {
        let dir = tempdir().unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(dir.path()).unwrap().permissions();
            perms.set_mode(0o700);
            fs::set_permissions(dir.path(), perms).unwrap();
        }
        let key_params = GenerateKeysParams {
            output_dir: dir.path().to_path_buf(),
            file_name: Some("validator_key.json".into()),
            entropy: "kitty".into(),
            overwrite: true,
        };
        let key_path = key_params.key_path();
        generate_keys(key_params).unwrap();

        let deposit_path = dir.path().join("deposit_data.json");
        let params = CreateDepositDataParams::for_network(
            key_path.clone(),
            deposit_path.clone(),
            "0x000102030405060708090a0b0c0d0e0f10111213",
            32_000_000_000,
            "hoodi",
            true,
        )
        .unwrap();

        let deposit = create_deposit_data(params).unwrap();
        let decoded: Vec<DepositData> =
            serde_json::from_slice(&fs::read(&deposit_path).unwrap()).unwrap();
        assert_eq!(decoded.len(), 1);
        assert_eq!(deposit, decoded[0]);
        assert_eq!(deposit.pubkey.len(), 96);
        assert_eq!(deposit.signature.len(), 192);
        assert_eq!(deposit.deposit_data_root.len(), 64);
        assert_eq!(deposit.network_name.as_deref(), Some("hoodi"));
    }

    #[test]
    fn for_network_normalizes_and_validates() {
        let params = CreateDepositDataParams::for_network(
            PathBuf::from("/validators/key.json"),
            PathBuf::from("/validators/deposit.json"),
            "0X9cAD4b41470bD949e1c4248ebA867E53aB3ceFEF",
            32_000_000_000,
            " Hoodi ",
            false,
        )
        .unwrap();

        assert_eq!(params.network_name.as_deref(), Some("hoodi"));
        assert_eq!(params.fork_version, [0x10, 0x00, 0x09, 0x10]);
        assert_eq!(
            params.withdrawal_credentials,
            "0200000000000000000000009cad4b41470bd949e1c4248eba867e53ab3cefef"
        );

        let err = CreateDepositDataParams::for_network(
            PathBuf::from("/validators/key.json"),
            PathBuf::from("/validators/deposit.json"),
            "0x1111",
            32_000_000_000,
            "hoodi",
            false,
        )
        .unwrap_err();
        assert!(
            err.to_string()
                .contains("withdrawal address must be 20 bytes")
        );

        let network_err = CreateDepositDataParams::for_network(
            PathBuf::from("/validators/key.json"),
            PathBuf::from("/validators/deposit.json"),
            "0x0000000000000000000000000000000000000000",
            32_000_000_000,
            "unknown",
            false,
        )
        .unwrap_err();
        assert!(network_err.to_string().contains("unsupported network name"));
    }

    #[test]
    fn writes_array_wrapper_json() {
        let fs = TestFilesystem::default();
        let crypto = TestCrypto;
        let key_path = PathBuf::from("/validators/key.json");
        let deposit_path = PathBuf::from("/validators/deposit.json");

        let key = ValidatorKey {
            public_key: "pub".into(),
            secret_key: "sec".into(),
        };
        fs.write_json_secure(&key_path, &key, true).unwrap();

        let params = CreateDepositDataParams {
            key_path: key_path.clone(),
            output_path: deposit_path.clone(),
            withdrawal_credentials: "cred".into(),
            amount_gwei: 32_000_000_000,
            fork_version: [0, 0, 0, 0],
            genesis_validators_root:
                "0x000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f".into(),
            overwrite: false,
            network_name: None,
        };

        create_deposit_data_with(params, &crypto, &fs).unwrap();

        let stored_json = fs
            .files
            .lock()
            .expect("mutex poisoned")
            .get(&deposit_path)
            .expect("missing deposit file")
            .clone();

        let trimmed = stored_json.trim();
        assert!(
            trimmed.starts_with('['),
            "expected array start, got {trimmed}"
        );
        assert!(trimmed.ends_with(']'), "expected array end, got {trimmed}");
    }
}
