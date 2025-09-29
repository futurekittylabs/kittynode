use crate::domain::config::Config;
use crate::infra::file::kittynode_path;
use eyre::Result;
use std::{
    fs,
    path::{Path, PathBuf},
};

pub struct ConfigStore;

impl ConfigStore {
    /// Loads the configuration from a TOML file and normalizes it before returning.
    pub fn load() -> Result<Config> {
        let config_path = Self::config_file_path()?;
        load_from_path(&config_path)
    }

    /// Saves the configuration to a TOML file after normalizing it in place.
    pub fn save_normalized(config: &mut Config) -> Result<()> {
        let config_path = Self::config_file_path()?;
        save_normalized_to(config, &config_path)
    }

    /// Returns the path to the configuration file.
    fn config_file_path() -> Result<PathBuf> {
        let mut path = kittynode_path()?;
        path.push("config.toml");
        Ok(path)
    }
}

fn load_from_path(config_path: &Path) -> Result<Config> {
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

fn save_normalized_to(config: &mut Config, config_path: &Path) -> Result<()> {
    config.normalize();
    if let Some(parent) = config_path.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent)?;
    }
    let toml_str = toml::to_string(config)?;
    fs::write(config_path, toml_str)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{Config, load_from_path, save_normalized_to};
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn load_returns_normalized_defaults_when_missing() {
        let temp = tempdir().unwrap();
        let config_path = temp.path().join("config.toml");

        let config = load_from_path(&config_path).expect("load should succeed");

        assert!(config.capabilities.is_empty());
        assert_eq!(config.server_url, "");
        assert_eq!(config.last_server_url, "");
        assert!(!config.has_remote_server);
    }

    #[test]
    fn load_trims_and_normalizes_existing_file() {
        let temp = tempdir().unwrap();
        let config_path = temp.path().join("config.toml");
        let toml = r#"
capabilities = ["cap1"]
server_url = " https://node.example.com "
last_server_url = " https://cached.example.com "
has_remote_server = true
auto_start_docker = true
"#;
        fs::write(&config_path, toml).unwrap();

        let config = load_from_path(&config_path).expect("load should succeed");

        assert_eq!(config.server_url, "https://node.example.com");
        assert_eq!(config.last_server_url, "https://cached.example.com");
        assert!(config.has_remote_server);
    }

    #[test]
    fn save_creates_parent_directories_and_normalizes_payload() {
        let temp = tempdir().unwrap();
        let config_path = temp.path().join("nested").join("config.toml");
        let mut config = Config {
            capabilities: vec!["cap1".into()],
            server_url: " https://node.example.com ".into(),
            last_server_url: String::new(),
            has_remote_server: false,
            onboarding_completed: true,
            auto_start_docker: true,
        };

        save_normalized_to(&mut config, &config_path).expect("save should succeed");

        assert!(config_path.exists(), "config file should be created");

        let persisted = load_from_path(&config_path).expect("reload should succeed");
        assert_eq!(persisted.server_url, "https://node.example.com");
        assert_eq!(persisted.last_server_url, "https://node.example.com");
        assert!(persisted.has_remote_server);
    }
}
