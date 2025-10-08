use eyre::{Context, Result};
use rand::RngCore;
use std::{fs, path::PathBuf};
use tracing::info;

pub fn kittynode_path() -> Result<PathBuf> {
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
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_generate_jwt_secret() {
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
}
