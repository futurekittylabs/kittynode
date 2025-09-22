use crate::domain::config::Config;
use crate::infra::file::kittynode_path;
use eyre::Result;
use std::{fs, path::PathBuf};

pub struct ConfigStore;

impl ConfigStore {
    /// Loads the configuration from a TOML file and normalizes it before returning.
    pub fn load() -> Result<Config> {
        let config_path = Self::config_file_path()?;
        if !config_path.exists() {
            let mut config = Config::default();
            config.normalize();
            return Ok(config);
        }
        let toml_str = fs::read_to_string(config_path)?;
        let mut config: Config = toml::from_str(&toml_str)?;
        config.normalize();
        Ok(config)
    }

    /// Saves the configuration to a TOML file after normalizing it in place.
    pub fn save_normalized(config: &mut Config) -> Result<()> {
        config.normalize();
        let config_path = Self::config_file_path()?;
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let toml_str = toml::to_string(config)?;
        fs::write(config_path, toml_str)?;
        Ok(())
    }

    /// Returns the path to the configuration file.
    fn config_file_path() -> Result<PathBuf> {
        let mut path = kittynode_path()?;
        path.push("config.toml");
        Ok(path)
    }
}
