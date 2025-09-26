use crate::application::validator::ports::{CryptoProvider, ValidatorFilesystem};
use crate::domain::validator::{DepositData, ValidatorKey};
use eth2_keystore::keypair_from_secret as lighthouse_keypair_from_secret;
use eyre::{Context, Result, eyre};
use sha2::{Digest, Sha256};
use tree_hash::TreeHash;
use types::{ChainSpec, DepositData as LHDepositData, EthSpec, MainnetEthSpec, PublicKeyBytes, SignatureBytes};
use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;

//

#[derive(Default)]
pub struct SimpleCryptoProvider;

impl CryptoProvider for SimpleCryptoProvider {
    fn generate_key(&self, entropy: &str) -> Result<ValidatorKey> {
        let secret = derive_secret_key(entropy)?;
        let kp = lighthouse_keypair_from_secret(&secret)
            .map_err(|e| eyre!(format!("failed to build keypair: {e:?}")))?;
        let pk_hex = kp.pk.as_hex_string();
        let pk_noprefix = pk_hex.trim_start_matches("0x").to_lowercase();
        Ok(ValidatorKey {
            public_key: pk_noprefix,
            secret_key: encode_hex(&secret),
        })
    }

    fn create_deposit_data(
        &self,
        key: &ValidatorKey,
        withdrawal_credentials: &str,
        amount_gwei: u64,
        fork_version: [u8; 4],
        _genesis_validators_root: &str,
    ) -> Result<DepositData> {
        let secret = decode_hex_fixed::<32>(&key.secret_key)?;
        let kp = lighthouse_keypair_from_secret(&secret)
            .map_err(|e| eyre!(format!("failed to build keypair: {e:?}")))?;

        if !key.public_key.is_empty() {
            let pk_hex = kp.pk.as_hex_string().trim_start_matches("0x").to_lowercase();
            if pk_hex != key.public_key.to_lowercase() {
                return Err(eyre!("validator public key does not match secret key"));
            }
        }

        let mut spec: ChainSpec = MainnetEthSpec::default_spec();
        spec.genesis_fork_version = fork_version;

        let mut deposit = LHDepositData {
            pubkey: PublicKeyBytes::from(kp.pk.clone()),
            withdrawal_credentials: {
                let wc = decode_hex_fixed::<32>(withdrawal_credentials)?;
                types::Hash256::from_slice(&wc)
            },
            amount: amount_gwei,
            signature: SignatureBytes::empty(),
        };
        deposit.signature = deposit.create_signature(&kp.sk, &spec);

        let msg_root = deposit.as_deposit_message().tree_hash_root();
        let data_root = deposit.tree_hash_root();

        let sig_json = serde_json::to_string(&deposit.signature)
            .map_err(|e| eyre!(format!("failed to serialize signature: {e}")))?;
        let sig_hex = sig_json.trim_matches('"').trim_start_matches("0x").to_lowercase();

        Ok(DepositData {
            pubkey: kp.pk.as_hex_string().trim_start_matches("0x").to_lowercase(),
            withdrawal_credentials: hex::encode(decode_hex_fixed::<32>(withdrawal_credentials)?),
            amount: amount_gwei,
            signature: sig_hex,
            deposit_message_root: hex::encode(msg_root.as_slice()),
            deposit_data_root: hex::encode(data_root.as_slice()),
            fork_version: hex::encode(fork_version),
            network_name: None,
        })
    }
}

/// Derives a BLS secret key deterministically from the provided entropy.
///
/// Callers must ensure the entropy is sourced from a high-quality RNG in
/// production contexts—this helper does not add additional randomness.
fn derive_secret_key(entropy: &str) -> Result<[u8; 32]> {
    let mut hasher = Sha256::new();
    hasher.update(entropy.as_bytes());
    Ok(hasher.finalize().into())
}

fn encode_hex(bytes: &[u8]) -> String {
    hex::encode(bytes)
}

fn decode_hex(input: &str) -> Result<Vec<u8>> {
    let trimmed = input.trim();
    let without_prefix = trimmed.strip_prefix("0x").unwrap_or(trimmed);
    if !without_prefix.len().is_multiple_of(2) {
        return Err(eyre!(
            "hex value must have an even length, got {} characters",
            without_prefix.len()
        ));
    }
    hex::decode(without_prefix).map_err(|err| eyre!("invalid hex: {err}"))
}

fn decode_hex_fixed<const N: usize>(input: &str) -> Result<[u8; N]> {
    let bytes = decode_hex(input)?;
    if bytes.len() != N {
        return Err(eyre!("expected {N} bytes, found {}", bytes.len()));
    }
    let mut array = [0u8; N];
    array.copy_from_slice(&bytes);
    Ok(array)
}


#[derive(Default)]
pub struct StdValidatorFilesystem;

impl ValidatorFilesystem for StdValidatorFilesystem {
    fn ensure_secure_directory(&self, path: &Path) -> Result<()> {
        if path.exists() {
            let metadata = fs::metadata(path).wrap_err("failed to inspect directory")?;
            if !metadata.is_dir() {
                eyre::bail!("expected directory at {}", path.display());
            }
            ensure_secure_dir_permissions(path, &metadata)?;
        } else {
            fs::create_dir_all(path).wrap_err("failed to create directory")?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mut permissions = fs::metadata(path)
                    .wrap_err("failed to read directory metadata")?
                    .permissions();
                permissions.set_mode(0o700);
                fs::set_permissions(path, permissions)
                    .wrap_err("failed to set directory permissions")?;
            }
        }
        Ok(())
    }

    fn write_json_secure<T: serde::Serialize + ?Sized>(
        &self,
        path: &Path,
        value: &T,
        overwrite: bool,
    ) -> Result<()> {
        let parent_dir: PathBuf = match path.parent().filter(|p| !p.as_os_str().is_empty()) {
            Some(parent) => parent.to_path_buf(),
            None => env::current_dir()
                .wrap_err("failed to resolve current directory for validator artifact")?,
        };

        self.ensure_secure_directory(&parent_dir)?;

        if path.exists() {
            if !overwrite {
                eyre::bail!("refusing to overwrite existing file: {}", path.display());
            }
            ensure_secure_file_permissions(path)?;
            fs::remove_file(path).wrap_err("failed to remove existing file")?;
        }

        let json = serde_json::to_vec(value).wrap_err("failed to serialize validator artifact")?;
        let mut temp_file =
            NamedTempFile::new_in(&parent_dir).wrap_err("failed to create temp file")?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut permissions = temp_file.as_file().metadata()?.permissions();
            permissions.set_mode(0o600);
            temp_file.as_file().set_permissions(permissions)?;
        }

        temp_file
            .write_all(&json)
            .wrap_err("failed to write validator artifact")?;
        temp_file
            .as_file_mut()
            .sync_all()
            .wrap_err("failed to sync validator artifact")?;
        temp_file.persist(path).map_err(|err| {
            eyre::eyre!(
                "failed to persist validator artifact {}: {}",
                path.display(),
                err
            )
        })?;
        ensure_secure_file_permissions(path)?;
        Ok(())
    }

    fn read_json_secure<T: serde::de::DeserializeOwned>(&self, path: &Path) -> Result<T> {
        if !path.exists() {
            eyre::bail!("validator artifact not found: {}", path.display());
        }
        ensure_secure_file_permissions(path)?;
        let bytes = fs::read(path).wrap_err("failed to read validator artifact")?;
        serde_json::from_slice(&bytes).wrap_err("failed to parse validator artifact")
    }
}

fn ensure_secure_dir_permissions(path: &Path, metadata: &fs::Metadata) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = metadata.permissions().mode();
        if mode & 0o022 != 0 {
            eyre::bail!(
                "directory {} is writable by group or others",
                path.display()
            );
        }
    }
    Ok(())
}

fn ensure_secure_file_permissions(path: &Path) -> Result<()> {
    let metadata = fs::metadata(path).wrap_err("failed to read file metadata")?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = metadata.permissions().mode();
        if mode & 0o077 != 0 {
            eyre::bail!("file {} is accessible to other users", path.display());
        }
    }
    #[cfg(not(unix))]
    {
        let _ = path;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn simple_crypto_is_deterministic() {
        let provider = SimpleCryptoProvider;
        let key = provider.generate_key("entropy").unwrap();
        let withdrawal = "0x000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f";
        let genesis_root = "0x000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f";
        let deposit = provider
            .create_deposit_data(&key, withdrawal, 32_000_000_000, [0, 0, 0, 0], genesis_root)
            .unwrap();

        let expected_secret = derive_secret_key("entropy").unwrap();
        let expected_public = expected_secret.sk_to_pk();
        let public_bytes = expected_public.to_bytes();
        let expected_secret_hex = encode_hex(&expected_secret.to_bytes());
        let expected_public_hex = encode_hex(&public_bytes);
        assert_eq!(key.secret_key, expected_secret_hex);
        assert_eq!(key.public_key, expected_public_hex);

        let withdrawal_bytes = decode_hex_fixed::<32>(withdrawal).unwrap();
        let message_root =
            compute_deposit_message_root(&public_bytes, &withdrawal_bytes, 32_000_000_000);
        let genesis_root_bytes = decode_hex_fixed::<32>(genesis_root).unwrap();
        let domain = compute_deposit_domain([0, 0, 0, 0], &genesis_root_bytes);
        let signing_root = compute_signing_root(&message_root, &domain);
        let signature = expected_secret.sign(signing_root.as_ref(), BLS_DST, &[]);
        let signature_bytes = signature.to_bytes();
        let data_root = compute_deposit_data_root(
            &public_bytes,
            &withdrawal_bytes,
            32_000_000_000,
            &signature_bytes,
        );

        assert_eq!(deposit.pubkey, encode_hex(&public_bytes));
        assert_eq!(deposit.deposit_message_root, encode_hex(&message_root));
        assert_eq!(deposit.signature, encode_hex(&signature_bytes));
        assert_eq!(deposit.deposit_data_root, encode_hex(&data_root));
        assert_eq!(
            deposit.withdrawal_credentials,
            encode_hex(&withdrawal_bytes)
        );
        assert_eq!(deposit.fork_version, "00000000");
    }

    #[test]
    fn filesystem_enforces_permissions() {
        let fs_impl = StdValidatorFilesystem;
        let dir = tempdir().unwrap();
        let secure_dir = dir.path().join("secure");
        fs_impl.ensure_secure_directory(&secure_dir).unwrap();

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&secure_dir).unwrap().permissions();
            perms.set_mode(0o775);
            fs::set_permissions(&secure_dir, perms).unwrap();
            let result = fs_impl.ensure_secure_directory(&secure_dir);
            assert!(result.is_err());
        }
    }

    #[test]
    fn filesystem_allows_current_directory_targets() {
        use std::path::Path;

        let fs_impl = StdValidatorFilesystem;
        let dir = tempdir().unwrap();

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(dir.path()).unwrap().permissions();
            perms.set_mode(0o700);
            fs::set_permissions(dir.path(), perms).unwrap();
        }

        let original_dir = env::current_dir().unwrap();
        env::set_current_dir(dir.path()).unwrap();

        let key = ValidatorKey {
            public_key: "pk".into(),
            secret_key: "sk".into(),
        };

        let result = fs_impl.write_json_secure(Path::new("artifact.json"), &key, true);
        env::set_current_dir(&original_dir).unwrap();

        result.unwrap();
        assert!(dir.path().join("artifact.json").exists());
    }
}
