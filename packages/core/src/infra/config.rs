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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::config::Config;
    use std::env;
    use std::fs;
    use std::path::Path;
    use std::sync::{Mutex, OnceLock};
    use tempfile::tempdir;

    static HOME_GUARD: OnceLock<Mutex<()>> = OnceLock::new();

    fn with_temp_home<F: FnOnce(&Path)>(test: F) {
        let _lock = HOME_GUARD
            .get_or_init(|| Mutex::new(()))
            .lock()
            .expect("lock poisoned");

        let temp_dir = tempdir().expect("failed to create temp dir");
        let home_path = temp_dir.path().to_path_buf();

        let original_home = env::var_os("HOME");
        // SAFETY: We temporarily adjust HOME for the duration of the test while holding a mutex
        unsafe {
            env::set_var("HOME", &home_path);
        }

        #[cfg(windows)]
        let original_userprofile = env::var_os("USERPROFILE");
        #[cfg(windows)]
        unsafe {
            env::set_var("USERPROFILE", &home_path);
        }

        let result =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| test(temp_dir.path())));

        match original_home {
            Some(val) => unsafe {
                env::set_var("HOME", val);
            },
            None => unsafe {
                env::remove_var("HOME");
            },
        }

        #[cfg(windows)]
        match original_userprofile {
            Some(val) => unsafe {
                env::set_var("USERPROFILE", val);
            },
            None => unsafe {
                env::remove_var("USERPROFILE");
            },
        }

        result.unwrap();
    }

    #[test]
    fn load_returns_default_when_missing() {
        with_temp_home(|_| {
            let config = ConfigStore::load().expect("load should succeed");
            assert_eq!(config.server_url, "");
            assert_eq!(config.last_server_url, "");
            assert!(config.capabilities.is_empty());
            assert!(!config.has_remote_server);
        });
    }

    #[test]
    fn save_normalized_creates_config_file_with_trimmed_values() {
        with_temp_home(|home| {
            let mut config = Config {
                server_url: " https://example.com ".to_string(),
                last_server_url: String::new(),
                capabilities: vec!["eth".to_string()],
                ..Default::default()
            };

            ConfigStore::save_normalized(&mut config).expect("save should succeed");

            let config_path = home.join(".kittynode").join("config.toml");
            assert!(config_path.exists(), "config.toml should be created");

            assert_eq!(config.server_url, "https://example.com");
            assert_eq!(config.last_server_url, "https://example.com");
            assert!(config.has_remote_server);

            let stored = fs::read_to_string(config_path).expect("config should be readable");
            let parsed: Config = toml::from_str(&stored).expect("config should parse");
            assert_eq!(parsed.server_url, "https://example.com");
            assert_eq!(parsed.last_server_url, "https://example.com");
            assert!(parsed.has_remote_server);
            assert_eq!(parsed.capabilities, vec!["eth".to_string()]);
        });
    }

    #[test]
    fn load_normalizes_existing_config_file() {
        with_temp_home(|home| {
            let store_dir = home.join(".kittynode");
            fs::create_dir_all(&store_dir).expect("failed to create kittynode dir");
            let config_path = store_dir.join("config.toml");

            fs::write(
                &config_path,
                r#"capabilities = ["eth"]
serverUrl = " https://rpc.example.com "
lastServerUrl = " "
hasRemoteServer = false
"#,
            )
            .expect("failed to seed config");

            let config = ConfigStore::load().expect("load should succeed");
            assert_eq!(config.server_url, "https://rpc.example.com");
            assert_eq!(config.last_server_url, "https://rpc.example.com");
            assert!(config.has_remote_server);
            assert_eq!(config.capabilities, vec!["eth".to_string()]);
        });
    }
}
