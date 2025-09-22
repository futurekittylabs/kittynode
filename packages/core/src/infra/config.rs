use crate::domain::config::Config;
use crate::infra::file::kittynode_path;
use eyre::Result;
use std::{fs, path::PathBuf};

pub struct ConfigStore;

impl ConfigStore {
    /// Loads the configuration from a TOML file.
    pub fn load() -> Result<Config> {
        let config_path = Self::config_file_path()?;
        if !config_path.exists() {
            return Ok(Config::default().normalize());
        }
        let toml_str = fs::read_to_string(config_path)?;
        let config: Config = toml::from_str(&toml_str)?;
        Ok(config.normalize())
    }

    /// Saves the configuration to a TOML file.
    pub fn save(config: &Config) -> Result<()> {
        let config_path = Self::config_file_path()?;
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let normalized = config.clone().normalize();
        let toml_str = toml::to_string(&normalized)?;
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
