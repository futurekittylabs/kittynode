use super::ports::{CryptoProvider, ValidatorFilesystem};
use crate::domain::validator::ValidatorKey;
use crate::infra::validator::{SimpleCryptoProvider, StdValidatorFilesystem};
use eyre::{Context, Result};
use std::path::{Path, PathBuf};

const DEFAULT_KEY_FILENAME: &str = "validator_key.json";

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

pub fn generate_keys(params: GenerateKeysParams) -> Result<ValidatorKey> {
    let crypto = SimpleCryptoProvider;
    let filesystem = StdValidatorFilesystem;
    generate_keys_with(params, &crypto, &filesystem)
}

pub fn generate_keys_with<P, F>(
    params: GenerateKeysParams,
    crypto: &P,
    filesystem: &F,
) -> Result<ValidatorKey>
where
    P: CryptoProvider,
    F: ValidatorFilesystem,
{
    let key_file = params.key_path();
    ensure_parent_secure(&key_file, filesystem)?;

    let key = crypto
        .generate_key(&params.entropy)
        .context("failed to generate validator key")?;

    filesystem
        .write_json_secure(&key_file, &key, params.overwrite)
        .context("failed to write validator key")?;

    Ok(key)
}

fn ensure_parent_secure<F: ValidatorFilesystem>(path: &Path, filesystem: &F) -> Result<()> {
    if let Some(parent) = path.parent() {
        filesystem
            .ensure_secure_directory(parent)
            .with_context(|| format!("invalid validator output directory: {}", parent.display()))?
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::validator::ports::{CryptoProvider, ValidatorFilesystem};
    use crate::domain::validator::ValidatorKey;
    use eyre::Result;
    use serde::{Serialize, de::DeserializeOwned};
    use std::collections::HashMap;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::Mutex;
    use tempfile::tempdir;

    #[derive(Default)]
    struct TestFilesystem {
        files: Mutex<HashMap<PathBuf, String>>,
        insecure_dirs: Mutex<Vec<PathBuf>>,
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
        fn generate_key(&self, entropy: &str) -> Result<ValidatorKey> {
            Ok(ValidatorKey {
                public_key: format!("pub-{entropy}"),
                secret_key: format!("sec-{entropy}"),
            })
        }

        fn create_deposit_data(
            &self,
            _key: &ValidatorKey,
            _withdrawal_credentials: &str,
            _amount_gwei: u64,
            _fork_version: [u8; 4],
            _genesis_validators_root: &str,
        ) -> Result<crate::domain::validator::DepositData> {
            unreachable!("not used in generate_keys tests")
        }
    }

    #[test]
    fn writes_key_using_filesystem() {
        let fs = TestFilesystem::default();
        let crypto = TestCrypto;
        let params = GenerateKeysParams {
            output_dir: PathBuf::from("/validators"),
            file_name: Some("key.json".into()),
            entropy: "seed".into(),
            overwrite: false,
        };

        let key = generate_keys_with(params.clone(), &crypto, &fs).unwrap();
        assert_eq!(
            key,
            ValidatorKey {
                public_key: "pub-seed".into(),
                secret_key: "sec-seed".into(),
            }
        );

        let stored: ValidatorKey = fs
            .read_json_secure(&PathBuf::from("/validators/key.json"))
            .unwrap();
        assert_eq!(stored, key);
    }

    #[test]
    fn rejects_insecure_directory() {
        let fs = TestFilesystem::default();
        fs.insecure_dirs
            .lock()
            .expect("mutex poisoned")
            .push(PathBuf::from("/validators"));
        let crypto = TestCrypto;
        let params = GenerateKeysParams {
            output_dir: PathBuf::from("/validators"),
            file_name: Some("key.json".into()),
            entropy: "seed".into(),
            overwrite: false,
        };

        let result = generate_keys_with(params, &crypto, &fs);
        assert!(result.is_err());
    }

    #[test]
    fn default_flow_creates_real_files() {
        let dir = tempdir().unwrap();
        let output_dir = dir.path().join("validators");
        let params = GenerateKeysParams {
            output_dir: output_dir.clone(),
            file_name: None,
            entropy: "kitty".into(),
            overwrite: true,
        };

        let key = generate_keys(params).unwrap();
        let written = fs::read_to_string(output_dir.join(DEFAULT_KEY_FILENAME)).unwrap();
        let decoded: ValidatorKey = serde_json::from_str(&written).unwrap();
        assert_eq!(decoded, key);
        assert_eq!(key.public_key.len(), 98);
        assert_eq!(key.secret_key.len(), 66);
    }
}
