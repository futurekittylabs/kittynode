use crate::application::validator::ports::{CryptoProvider, ValidatorFilesystem};
use crate::domain::validator::{DepositData, ValidatorKey};
use blst::min_pk::SecretKey;
use eyre::{Context, Result, eyre};
use sha2::{Digest, Sha256};
use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;

const DOMAIN_DEPOSIT: [u8; 4] = [0x03, 0x00, 0x00, 0x00];
const BLS_DST: &[u8] = b"BLS_SIG_BLS12381G2_XMD:SHA-256_SSWU_RO_POP_";
type Root = [u8; 32];
const ZERO_ROOT: Root = [0u8; 32];

#[derive(Default)]
pub struct SimpleCryptoProvider;

impl CryptoProvider for SimpleCryptoProvider {
    fn generate_key(&self, entropy: &str) -> Result<ValidatorKey> {
        let secret = derive_secret_key(entropy)?;
        let public = secret.sk_to_pk();

        Ok(ValidatorKey {
            public_key: encode_hex(&public.to_bytes()),
            secret_key: encode_hex(&secret.to_bytes()),
        })
    }

    fn create_deposit_data(
        &self,
        key: &ValidatorKey,
        withdrawal_credentials: &str,
        amount_gwei: u64,
        fork_version: [u8; 4],
        genesis_validators_root: &str,
    ) -> Result<DepositData> {
        let secret_bytes = decode_hex_fixed::<32>(&key.secret_key)?;
        let secret = SecretKey::from_bytes(&secret_bytes)
            .map_err(|_| eyre!("invalid validator secret key"))?;

        let public_bytes = secret.sk_to_pk().to_bytes();
        let public_hex = encode_hex(&public_bytes);

        if !key.public_key.is_empty() {
            let stored_pub = decode_hex(&key.public_key)?;
            if stored_pub != public_bytes {
                return Err(eyre!("validator public key does not match secret key"));
            }
        }

        let withdrawal_bytes = decode_hex_fixed::<32>(withdrawal_credentials)?;
        let genesis_root_bytes = decode_hex_fixed::<32>(genesis_validators_root)?;
        let domain = compute_deposit_domain(fork_version, &genesis_root_bytes);
        let deposit_message_root =
            compute_deposit_message_root(&public_bytes, &withdrawal_bytes, amount_gwei);
        let signing_root = compute_signing_root(&deposit_message_root, &domain);
        let signature = secret.sign(signing_root.as_ref(), BLS_DST, &[]);
        let signature_bytes = signature.to_bytes();
        let deposit_data_root = compute_deposit_data_root(
            &public_bytes,
            &withdrawal_bytes,
            amount_gwei,
            &signature_bytes,
        );

        Ok(DepositData {
            public_key: public_hex,
            withdrawal_credentials: encode_hex(&withdrawal_bytes),
            amount_gwei,
            signature: encode_hex(&signature_bytes),
            deposit_message_root: encode_hex(&deposit_message_root),
            deposit_data_root: encode_hex(&deposit_data_root),
            fork_version: encode_hex(&fork_version),
        })
    }
}

fn derive_secret_key(entropy: &str) -> Result<SecretKey> {
    let mut hasher = Sha256::new();
    hasher.update(entropy.as_bytes());
    let seed: [u8; 32] = hasher.finalize().into();
    SecretKey::key_gen(&seed, &[]).map_err(|_| eyre!("failed to derive validator secret key"))
}

fn encode_hex(bytes: &[u8]) -> String {
    format!("0x{}", hex::encode(bytes))
}

fn decode_hex(input: &str) -> Result<Vec<u8>> {
    let trimmed = input.trim();
    let without_prefix = trimmed.strip_prefix("0x").unwrap_or(trimmed);
    if without_prefix.len() % 2 != 0 {
        return Err(eyre!("hex value must have an even length"));
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

fn compute_deposit_domain(fork_version: [u8; 4], genesis_validators_root: &[u8; 32]) -> Root {
    let mut current_version_chunk = [0u8; 32];
    current_version_chunk[..4].copy_from_slice(&fork_version);
    let fork_data_root = merkleize(&[current_version_chunk, *genesis_validators_root]);

    let mut domain = [0u8; 32];
    domain[..4].copy_from_slice(&DOMAIN_DEPOSIT);
    domain[4..].copy_from_slice(&fork_data_root[..28]);
    domain
}

fn compute_deposit_message_root(
    pubkey: &[u8; 48],
    withdrawal_credentials: &[u8; 32],
    amount_gwei: u64,
) -> Root {
    let pubkey_root = merkleize(&pack_bytes(pubkey));
    let withdrawal_root = chunk_bytes32(withdrawal_credentials);
    let amount_root = uint64_chunk(amount_gwei);
    merkleize(&[pubkey_root, withdrawal_root, amount_root])
}

fn compute_deposit_data_root(
    pubkey: &[u8; 48],
    withdrawal_credentials: &[u8; 32],
    amount_gwei: u64,
    signature: &[u8; 96],
) -> Root {
    let pubkey_root = merkleize(&pack_bytes(pubkey));
    let withdrawal_root = chunk_bytes32(withdrawal_credentials);
    let amount_root = uint64_chunk(amount_gwei);
    let signature_root = merkleize(&pack_bytes(signature));
    merkleize(&[pubkey_root, withdrawal_root, amount_root, signature_root])
}

fn compute_signing_root(object_root: &Root, domain: &Root) -> Root {
    merkleize(&[*object_root, *domain])
}

fn merkleize(chunks: &[Root]) -> Root {
    if chunks.is_empty() {
        return ZERO_ROOT;
    }

    let mut layer: Vec<Root> = chunks.to_vec();
    let target_len = next_power_of_two(layer.len());
    layer.resize(target_len, ZERO_ROOT);

    while layer.len() > 1 {
        let mut next_layer = Vec::with_capacity(layer.len() / 2);
        for pair in layer.chunks(2) {
            let left = pair[0];
            let right = pair[1];
            next_layer.push(hash_pair(&left, &right));
        }
        layer = next_layer;
    }

    layer[0]
}

fn hash_pair(left: &Root, right: &Root) -> Root {
    let mut hasher = Sha256::new();
    hasher.update(left);
    hasher.update(right);
    hasher.finalize().into()
}

fn pack_bytes(bytes: &[u8]) -> Vec<Root> {
    if bytes.is_empty() {
        return vec![ZERO_ROOT];
    }

    let mut chunks = Vec::new();
    for chunk in bytes.chunks(32) {
        let mut root = [0u8; 32];
        root[..chunk.len()].copy_from_slice(chunk);
        chunks.push(root);
    }
    chunks
}

fn chunk_bytes32(bytes: &[u8; 32]) -> Root {
    *bytes
}

fn uint64_chunk(value: u64) -> Root {
    let mut chunk = [0u8; 32];
    chunk[..8].copy_from_slice(&value.to_le_bytes());
    chunk
}

fn next_power_of_two(value: usize) -> usize {
    if value == 0 {
        1
    } else {
        value.next_power_of_two()
    }
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

    fn write_json_secure<T: serde::Serialize>(
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

        let json =
            serde_json::to_vec_pretty(value).wrap_err("failed to serialize validator artifact")?;
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
        if mode & 0o077 != 0 {
            eyre::bail!("directory {} is accessible to other users", path.display());
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
        let provider = SimpleCryptoProvider::default();
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

        assert_eq!(deposit.deposit_message_root, encode_hex(&message_root));
        assert_eq!(deposit.signature, encode_hex(&signature_bytes));
        assert_eq!(deposit.deposit_data_root, encode_hex(&data_root));
        assert_eq!(
            deposit.withdrawal_credentials,
            encode_hex(&withdrawal_bytes)
        );
        assert_eq!(deposit.fork_version, "0x00000000");
    }

    #[test]
    fn filesystem_enforces_permissions() {
        let fs_impl = StdValidatorFilesystem::default();
        let dir = tempdir().unwrap();
        let secure_dir = dir.path().join("secure");
        fs_impl.ensure_secure_directory(&secure_dir).unwrap();

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&secure_dir).unwrap().permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&secure_dir, perms).unwrap();
            let result = fs_impl.ensure_secure_directory(&secure_dir);
            assert!(result.is_err());
        }
    }

    #[test]
    fn filesystem_allows_current_directory_targets() {
        use std::path::Path;

        let fs_impl = StdValidatorFilesystem::default();
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
