use crate::paths::kittynode_path;
use eyre::{Result, eyre};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};
use tracing::info;
use url::Url;

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    #[serde(default)]
    pub capabilities: Vec<String>,
    #[serde(default, alias = "server_url")]
    pub server_url: String,
    #[serde(default, alias = "last_server_url")]
    pub last_server_url: String,
    #[serde(default, alias = "remote_connected")]
    pub has_remote_server: bool,
    #[serde(default, alias = "onboarding_completed")]
    pub onboarding_completed: bool,
    #[serde(default, alias = "auto_start_docker")]
    pub auto_start_docker: bool,
    #[serde(default = "default_show_tray_icon", alias = "show_tray_icon")]
    pub show_tray_icon: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            capabilities: Vec::new(),
            server_url: String::new(),
            last_server_url: String::new(),
            has_remote_server: false,
            onboarding_completed: false,
            auto_start_docker: false,
            show_tray_icon: true,
        }
    }
}

fn default_show_tray_icon() -> bool {
    true
}

impl Config {
    pub fn normalize(&mut self) {
        self.server_url = self.server_url.trim().to_string();
        let has_server_url = !self.server_url.is_empty();

        if self.last_server_url.trim().is_empty() && has_server_url {
            self.last_server_url = self.server_url.clone();
        } else {
            self.last_server_url = self.last_server_url.trim().to_string();
        }

        self.has_remote_server = has_server_url;
    }
}

struct ConfigStore;

impl ConfigStore {
    fn load() -> Result<Config> {
        let config_path = Self::config_path()?;
        load_from_path(&config_path)
    }

    fn save(config: &mut Config) -> Result<()> {
        let config_path = Self::config_path()?;
        save_to_path(config, &config_path)
    }

    fn config_path() -> Result<PathBuf> {
        Ok(kittynode_path()?.join("config.toml"))
    }
}

pub fn get_config() -> Result<Config> {
    ConfigStore::load()
}

pub fn get_server_url() -> Result<String> {
    Ok(ConfigStore::load()?.server_url)
}

pub fn get_onboarding_completed() -> Result<bool> {
    let config = ConfigStore::load()?;
    info!(
        "Retrieved onboarding completed status: {}",
        config.onboarding_completed
    );
    Ok(config.onboarding_completed)
}

pub fn get_capabilities() -> Result<Vec<String>> {
    Ok(ConfigStore::load()?.capabilities)
}

pub(crate) fn write_config(mut config: Config) -> Result<()> {
    ConfigStore::save(&mut config)
}

pub fn set_onboarding_completed(completed: bool) -> Result<()> {
    let mut config = ConfigStore::load()?;
    config.onboarding_completed = completed;
    ConfigStore::save(&mut config)?;
    info!("Set onboarding completed to: {}", completed);
    Ok(())
}

pub fn set_auto_start_docker(enabled: bool) -> Result<()> {
    let mut config = ConfigStore::load()?;
    config.auto_start_docker = enabled;
    ConfigStore::save(&mut config)?;
    info!("Set auto start docker to: {}", enabled);
    Ok(())
}

pub fn set_show_tray_icon(enabled: bool) -> Result<()> {
    let mut config = ConfigStore::load()?;
    config.show_tray_icon = enabled;
    ConfigStore::save(&mut config)?;
    info!("Set show tray icon to: {}", enabled);
    Ok(())
}

pub fn set_server_url(endpoint: String) -> Result<()> {
    let mut config = ConfigStore::load()?;
    apply_server_url(&mut config, &endpoint)?;
    ConfigStore::save(&mut config)
}

pub fn add_capability(capability: &str) -> Result<()> {
    let mut config = ConfigStore::load()?;
    add_to_capabilities(&mut config.capabilities, capability);
    ConfigStore::save(&mut config)
}

pub fn remove_capability(capability: &str) -> Result<()> {
    let mut config = ConfigStore::load()?;
    if let Some(position) = config
        .capabilities
        .iter()
        .position(|value| value == capability)
    {
        config.capabilities.remove(position);
    }
    ConfigStore::save(&mut config)
}

fn load_from_path(config_path: &Path) -> Result<Config> {
    if !config_path.exists() {
        let mut config = Config::default();
        config.normalize();
        return Ok(config);
    }

    let raw = fs::read_to_string(config_path)?;
    let mut config: Config = toml::from_str(&raw)?;
    config.normalize();
    Ok(config)
}

fn save_to_path(config: &mut Config, config_path: &Path) -> Result<()> {
    config.normalize();
    if let Some(parent) = config_path.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent)?;
    }

    let raw = toml::to_string(config)?;
    fs::write(config_path, raw)?;
    Ok(())
}

fn validate_server_url(endpoint: &str) -> Result<()> {
    if endpoint.is_empty() {
        return Ok(());
    }

    let parsed =
        Url::parse(endpoint).map_err(|error| eyre!("invalid server URL '{endpoint}': {error}"))?;

    match parsed.scheme() {
        "http" | "https" => {}
        scheme => {
            return Err(eyre!(
                "invalid server URL '{endpoint}': unsupported scheme '{scheme}' (expected http or https)"
            ));
        }
    }

    if parsed.host_str().is_none() {
        return Err(eyre!("invalid server URL '{endpoint}': missing host"));
    }

    if !parsed.username().is_empty() || parsed.password().is_some() {
        return Err(eyre!(
            "invalid server URL '{endpoint}': credentials are not supported"
        ));
    }

    Ok(())
}

fn apply_server_url(config: &mut Config, endpoint: &str) -> Result<()> {
    let trimmed = endpoint.trim();
    validate_server_url(trimmed)?;

    if trimmed.is_empty() {
        config.server_url.clear();
        config.has_remote_server = false;
    } else {
        let normalized = trimmed.to_string();
        config.server_url = normalized.clone();
        config.last_server_url = normalized;
        config.has_remote_server = true;
    }

    Ok(())
}

fn add_to_capabilities(capabilities: &mut Vec<String>, capability: &str) {
    let already_present = capabilities.iter().any(|value| value == capability);
    if !already_present {
        capabilities.push(capability.to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::{
        Config, add_to_capabilities, apply_server_url, load_from_path, save_to_path,
        validate_server_url,
    };
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn default_enables_tray_icon() {
        let config = Config::default();
        assert!(config.show_tray_icon);
    }

    #[test]
    fn normalizes_server_and_last_urls() {
        let mut config = Config {
            server_url: " https://example.com ".to_string(),
            last_server_url: String::new(),
            has_remote_server: false,
            ..Default::default()
        };

        config.normalize();

        assert_eq!(config.server_url, "https://example.com");
        assert_eq!(config.last_server_url, "https://example.com");
        assert!(config.has_remote_server);
    }

    #[test]
    fn normalize_trims_last_url_and_updates_flag() {
        let mut config = Config {
            server_url: String::new(),
            last_server_url: " https://cached.example.com ".to_string(),
            has_remote_server: true,
            ..Default::default()
        };

        config.normalize();

        assert_eq!(config.server_url, "");
        assert_eq!(config.last_server_url, "https://cached.example.com");
        assert!(!config.has_remote_server);
    }

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
            show_tray_icon: true,
        };

        save_to_path(&mut config, &config_path).expect("save should succeed");

        assert!(config_path.exists(), "config file should be created");

        let persisted = load_from_path(&config_path).expect("reload should succeed");
        assert_eq!(persisted.server_url, "https://node.example.com");
        assert_eq!(persisted.last_server_url, "https://node.example.com");
        assert!(persisted.has_remote_server);
    }

    #[test]
    fn validate_allows_empty_endpoint() {
        assert!(validate_server_url("").is_ok());
    }

    #[test]
    fn validate_rejects_invalid_scheme() {
        let error =
            validate_server_url("ftp://example.com").expect_err("expected validation failure");
        assert!(error.to_string().contains("unsupported scheme"));
    }

    #[test]
    fn apply_sets_server_url_and_last() {
        let mut config = Config::default();
        apply_server_url(&mut config, " http://example.com ").expect("apply should succeed");

        assert_eq!(config.server_url, "http://example.com");
        assert_eq!(config.last_server_url, "http://example.com");
        assert!(config.has_remote_server);
    }

    #[test]
    fn apply_clears_server_but_preserves_last() {
        let mut config = Config::default();
        apply_server_url(&mut config, "http://example.com").expect("initial apply should succeed");
        apply_server_url(&mut config, "").expect("clearing should succeed");

        assert_eq!(config.server_url, "");
        assert_eq!(config.last_server_url, "http://example.com");
        assert!(!config.has_remote_server);
    }

    #[test]
    fn apply_preserves_trailing_slash() {
        let mut config = Config::default();
        apply_server_url(&mut config, "https://example.com/ ").expect("apply should succeed");

        assert_eq!(config.server_url, "https://example.com/");
        assert_eq!(config.last_server_url, "https://example.com/");
        assert!(config.has_remote_server);
    }

    #[test]
    fn apply_does_not_mutate_on_validation_error() {
        let mut config = Config::default();
        let _error =
            apply_server_url(&mut config, "not a url").expect_err("expected validation failure");

        assert_eq!(config.server_url, "");
        assert_eq!(config.last_server_url, "");
        assert!(!config.has_remote_server);
    }

    #[test]
    fn doesnt_add_duplicate() {
        let mut capabilities = vec!["cap1".to_string(), "cap2".to_string()];
        add_to_capabilities(&mut capabilities, "cap1");
        assert_eq!(capabilities, vec!["cap1".to_string(), "cap2".to_string()]);
    }

    #[test]
    fn adds_new_capability() {
        let mut capabilities = vec!["cap1".to_string(), "cap2".to_string()];
        add_to_capabilities(&mut capabilities, "cap3");
        assert_eq!(
            capabilities,
            vec!["cap1".to_string(), "cap2".to_string(), "cap3".to_string()]
        );
    }
}
