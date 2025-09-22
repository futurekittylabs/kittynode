use super::ports::{CryptoProvider, ValidatorFilesystem};
use crate::application::validator::input::{
    parse_fork_version_hex, parse_genesis_validators_root_hex,
};
use crate::domain::validator::DepositData;
use crate::infra::validator::{SimpleCryptoProvider, StdValidatorFilesystem};
use eyre::{Context, Result};
use std::path::PathBuf;

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
    pub fn from_hex_inputs(
        key_path: PathBuf,
        output_path: PathBuf,
        withdrawal_credentials: String,
        amount_gwei: u64,
        fork_version_hex: &str,
        genesis_root_hex: &str,
        overwrite: bool,
    ) -> Result<Self> {
        Self::from_hex_inputs_with_metadata(
            key_path,
            output_path,
            withdrawal_credentials,
            amount_gwei,
            fork_version_hex,
            genesis_root_hex,
            overwrite,
            None,
        )
    }

    pub fn from_hex_inputs_with_metadata(
        key_path: PathBuf,
        output_path: PathBuf,
        withdrawal_credentials: String,
        amount_gwei: u64,
        fork_version_hex: &str,
        genesis_root_hex: &str,
        overwrite: bool,
        network_name: Option<String>,
    ) -> Result<Self> {
        let fork_version = parse_fork_version_hex(fork_version_hex)?;
        let genesis_validators_root = parse_genesis_validators_root_hex(genesis_root_hex)?;
        Ok(Self {
            key_path,
            output_path,
            withdrawal_credentials,
            amount_gwei,
            fork_version,
            genesis_validators_root,
            overwrite,
            network_name,
        })
    }
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

    if let Some(parent) = params.output_path.parent() {
        filesystem
            .ensure_secure_directory(parent)
            .with_context(|| {
                format!("invalid validator artifact directory: {}", parent.display())
            })?;
    }

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

    filesystem
        .write_json_secure(
            &params.output_path,
            &vec![deposit.clone()],
            params.overwrite,
        )
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

        fn write_json_secure<T: Serialize>(
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
        let params = CreateDepositDataParams {
            key_path: key_path.clone(),
            output_path: deposit_path.clone(),
            withdrawal_credentials:
                "0x000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f".into(),
            amount_gwei: 32_000_000_000,
            fork_version: [0, 0, 0, 0],
            genesis_validators_root:
                "0x000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f".into(),
            overwrite: true,
            network_name: Some("testnet".into()),
        };

        let deposit = create_deposit_data(params).unwrap();
        let decoded: Vec<DepositData> =
            serde_json::from_slice(&fs::read(&deposit_path).unwrap()).unwrap();
        assert_eq!(decoded.len(), 1);
        assert_eq!(deposit, decoded[0]);
        assert_eq!(deposit.pubkey.len(), 98);
        assert_eq!(deposit.signature.len(), 194);
        assert_eq!(deposit.deposit_data_root.len(), 66);
        assert_eq!(deposit.network_name.as_deref(), Some("testnet"));
    }
}
