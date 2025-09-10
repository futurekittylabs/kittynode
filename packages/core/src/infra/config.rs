use crate::domain::config::Config;
use crate::infra::home::Home;
use eyre::Result;
use std::{
    fs,
    path::{Path, PathBuf},
};

pub struct ConfigStore;

impl ConfigStore {
    // Default load/save removed in favor of explicit context to avoid global state in tests.

    /// Returns the path to the configuration file using a provided base directory.
    fn config_file_path_with_base(base: &Path) -> PathBuf {
        let mut path = PathBuf::from(base);
        path.push("config.toml");
        path
    }

    /// Loads the configuration from a TOML file using the provided home.
    pub fn load_from(home: &Home) -> Result<Config> {
        let config_path = Self::config_file_path_with_base(home.base());
        if !config_path.exists() {
            return Ok(Config::default());
        }
        let toml_str = fs::read_to_string(config_path)?;
        let config = toml::from_str(&toml_str)?;
        Ok(config)
    }

    /// Saves the configuration to a TOML file using the provided home.
    pub fn save_to(home: &Home, config: &Config) -> Result<()> {
        let config_path = Self::config_file_path_with_base(home.base());
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let toml_str = toml::to_string(config)?;
        fs::write(config_path, toml_str)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn load_default_when_missing_and_save_roundtrip() {
        let dir = tempdir().unwrap();
        let home = Home::from_base(dir.path().join(".kittynode"));

        // Loading before save yields default
        let cfg = ConfigStore::load_from(&home).expect("load default");
        assert!(cfg.capabilities.is_empty());
        assert_eq!(cfg.server_url, "");

        // Save a config and load it back
        let mut to_save = cfg;
        to_save.capabilities = vec!["eth".into(), "solana".into()];
        to_save.server_url = "http://localhost:3000".into();
        ConfigStore::save_to(&home, &to_save).expect("save ok");

        let loaded = ConfigStore::load_from(&home).expect("load saved");
        assert_eq!(loaded.capabilities, to_save.capabilities);
        assert_eq!(loaded.server_url, to_save.server_url);
    }
}
