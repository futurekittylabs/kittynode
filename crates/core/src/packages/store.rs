use crate::packages::PackageConfig;
use crate::paths::kittynode_path;
use eyre::Result;
use std::{fs, path::Path, path::PathBuf};

pub struct PackageConfigStore;

impl PackageConfigStore {
    pub fn load(package_name: &str) -> Result<PackageConfig> {
        let base_dir = kittynode_path()?;
        Self::load_from(&base_dir, package_name)
    }

    pub fn save(package_name: &str, config: &PackageConfig) -> Result<()> {
        let base_dir = kittynode_path()?;
        Self::save_to(&base_dir, package_name, config)
    }

    pub fn load_from(base_dir: &Path, package_name: &str) -> Result<PackageConfig> {
        let config_path = Self::config_file_path(base_dir, package_name);
        if !config_path.exists() {
            return Ok(PackageConfig::default());
        }

        let raw = fs::read_to_string(config_path)?;
        let config = toml::from_str(&raw)?;
        Ok(config)
    }

    pub fn save_to(base_dir: &Path, package_name: &str, config: &PackageConfig) -> Result<()> {
        let config_path = Self::config_file_path(base_dir, package_name);
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let raw = toml::to_string_pretty(config)?;
        fs::write(config_path, raw)?;
        Ok(())
    }

    pub(crate) fn package_dir(base_dir: &Path, package_name: &str) -> PathBuf {
        base_dir.join("packages").join(package_name)
    }

    pub fn default_package_dir(package_name: &str) -> Result<PathBuf> {
        Ok(Self::package_dir(&kittynode_path()?, package_name))
    }

    pub(crate) fn config_file_path(base_dir: &Path, package_name: &str) -> PathBuf {
        Self::package_dir(base_dir, package_name).join("config.toml")
    }
}

#[cfg(test)]
mod tests {
    use super::PackageConfigStore;
    use crate::packages::PackageConfig;
    use std::{fs, path::Path};
    use tempfile::tempdir;

    #[test]
    fn load_from_returns_default_when_missing() {
        let temp_dir = tempdir().expect("failed to create temp dir");
        let base_dir = temp_dir.path().join(".config").join("kittynode");
        let config =
            PackageConfigStore::load_from(&base_dir, "test-package").expect("load should succeed");
        assert!(config.values.is_empty());
    }

    #[test]
    fn save_to_persists_configuration_for_round_trip() {
        let temp_dir = tempdir().expect("failed to create temp dir");
        let base_dir = temp_dir.path().join(".config").join("kittynode");
        let mut config = PackageConfig::default();
        config
            .values
            .insert("apiEndpoint".into(), "https://example.invalid".into());

        PackageConfigStore::save_to(&base_dir, "validator", &config)
            .expect("save should create config file");

        let expected_path = base_dir
            .join("packages")
            .join("validator")
            .join("config.toml");
        assert!(expected_path.exists());

        let raw = fs::read_to_string(&expected_path).expect("config should be readable");
        assert!(raw.contains("apiEndpoint"));
        assert!(raw.contains("https://example.invalid"));

        let loaded =
            PackageConfigStore::load_from(&base_dir, "validator").expect("reload should succeed");
        assert_eq!(
            loaded
                .values
                .get("apiEndpoint")
                .map(std::string::String::as_str),
            Some("https://example.invalid")
        );
    }

    #[test]
    fn config_file_path_constructs_correct_path() {
        let base_dir = Path::new("/home/user/.config/kittynode");
        let path = PackageConfigStore::config_file_path(base_dir, "ethereum");
        assert_eq!(
            path,
            Path::new("/home/user/.config/kittynode/packages/ethereum/config.toml")
        );
    }

    #[test]
    fn config_file_path_handles_special_characters() {
        let base_dir = Path::new("/tmp/test");
        let path = PackageConfigStore::config_file_path(base_dir, "my-validator");
        assert_eq!(
            path,
            Path::new("/tmp/test/packages/my-validator/config.toml")
        );
    }
}
