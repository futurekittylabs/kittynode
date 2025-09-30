use eyre::{Context, Result};
use rand::RngCore;
use std::{
    fs,
    path::PathBuf,
    sync::{OnceLock, RwLock},
};
#[cfg(test)]
use std::{path::Path, sync::Mutex};
use tracing::info;

fn override_storage() -> &'static RwLock<Option<PathBuf>> {
    static STORAGE: OnceLock<RwLock<Option<PathBuf>>> = OnceLock::new();
    STORAGE.get_or_init(|| RwLock::new(None))
}

#[cfg(test)]
fn override_guard() -> &'static Mutex<()> {
    static GUARD: OnceLock<Mutex<()>> = OnceLock::new();
    GUARD.get_or_init(|| Mutex::new(()))
}

pub(crate) fn kittynode_path() -> Result<PathBuf> {
    let maybe_override = override_storage()
        .read()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    if let Some(path) = maybe_override.clone() {
        return Ok(path);
    }

    home::home_dir()
        .map(|home| home.join(".kittynode"))
        .ok_or_else(|| eyre::eyre!("Failed to determine the .kittynode path"))
}

pub(crate) fn generate_jwt_secret_with_path(path: &PathBuf) -> Result<String> {
    if !path.exists() {
        info!("Creating directory at {:?}", path);
        fs::create_dir_all(path).wrap_err("Failed to create directory")?;
    }

    info!("Generating JWT secret using a random number generator");

    // Generate 32 random bytes
    let mut buf = [0u8; 32];
    rand::rng().fill_bytes(&mut buf);

    // Convert the random bytes to hex
    let secret = hex::encode(buf);

    // Write the secret to the path
    fs::write(path.join("jwt.hex"), &secret).wrap_err("Failed to write JWT secret to file")?;

    info!(
        "JWT secret successfully generated and written to {:?}",
        path.join("jwt.hex")
    );

    Ok(secret)
}

pub(crate) fn generate_jwt_secret() -> Result<String> {
    let path = kittynode_path()?;
    generate_jwt_secret_with_path(&path)
}

#[cfg(test)]
pub(crate) fn with_kittynode_path_override<T>(path: impl AsRef<Path>, f: impl FnOnce() -> T) -> T {
    use std::panic::{self, AssertUnwindSafe};

    let _guard = override_guard()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    {
        let mut storage = override_storage()
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        *storage = Some(path.as_ref().to_path_buf());
    }

    let result = panic::catch_unwind(AssertUnwindSafe(f));

    {
        let mut storage = override_storage()
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        *storage = None;
    }

    match result {
        Ok(value) => value,
        Err(payload) => panic::resume_unwind(payload),
    }
}

#[cfg(test)]
mod tests {
    use std::panic::{self, AssertUnwindSafe};

    use super::*;
    use tempfile::tempdir;

    #[test]
    fn generate_jwt_secret_with_path_writes_hex_secret() {
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_path_buf();

        let result = generate_jwt_secret_with_path(&temp_path);
        assert!(result.is_ok(), "Expected OK, got {result:?}");

        let jwt_file_path = temp_path.join("jwt.hex");
        assert!(jwt_file_path.exists(), "JWT secret file not found");

        let secret = fs::read_to_string(jwt_file_path).unwrap();
        assert_eq!(secret.len(), 64, "Expected 64 hex characters");
        assert!(secret.chars().all(|c| c.is_ascii_hexdigit()));

        assert_eq!(result.unwrap(), secret, "Secrets do not match");
    }

    #[test]
    fn with_override_routes_kittynode_path() {
        let temp_dir = tempdir().unwrap();
        let expected_path = temp_dir.path().join("nested");

        with_kittynode_path_override(&expected_path, || {
            let resolved = kittynode_path().expect("override should provide path");
            assert_eq!(resolved, expected_path);
        });
    }

    #[test]
    fn generate_jwt_secret_uses_overridden_path() {
        let temp_dir = tempdir().unwrap();

        with_kittynode_path_override(temp_dir.path(), || {
            let secret = generate_jwt_secret().expect("secret generation should succeed");
            assert_eq!(secret.len(), 64);

            let jwt_file_path = temp_dir.path().join("jwt.hex");
            let persisted = fs::read_to_string(jwt_file_path).expect("jwt file should exist");
            assert_eq!(persisted, secret);
        });
    }

    #[test]
    fn override_is_cleared_when_closure_panics() {
        let temp_dir = tempdir().unwrap();

        let panic_result = panic::catch_unwind(AssertUnwindSafe(|| {
            with_kittynode_path_override(temp_dir.path(), || {
                panic!("intentional test panic");
            })
        }));

        assert!(panic_result.is_err(), "closure should panic");

        // Ensure the override was cleared by running another closure
        let second_dir = tempdir().unwrap();
        with_kittynode_path_override(second_dir.path(), || {
            let resolved = kittynode_path().expect("override should provide path");
            assert_eq!(resolved, second_dir.path());
        });
    }
}
