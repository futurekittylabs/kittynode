use crate::domain::config::Config;
use crate::infra::config::ConfigStore;
use eyre::{Result, eyre};
use tracing::info;
use url::Url;

// ── Getters ──────────────────────────────────────────────────────────

/// Returns the persisted Kittynode configuration
pub fn get_config() -> Result<Config> {
    ConfigStore::load()
}

pub fn get_server_url() -> Result<String> {
    let config = ConfigStore::load()?;
    Ok(config.server_url)
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
    let config = ConfigStore::load()?;
    Ok(config.capabilities)
}

// ── Setters ──────────────────────────────────────────────────────────

pub fn set_onboarding_completed(completed: bool) -> Result<()> {
    let mut config = ConfigStore::load()?;
    config.onboarding_completed = completed;
    ConfigStore::save_normalized(&mut config)?;
    info!("Set onboarding completed to: {}", completed);
    Ok(())
}

/// Enables or disables automatic Docker startup at application launch
pub fn set_auto_start_docker(enabled: bool) -> Result<()> {
    let mut config = ConfigStore::load()?;
    config.auto_start_docker = enabled;
    ConfigStore::save_normalized(&mut config)?;
    info!("Set auto start docker to: {}", enabled);
    Ok(())
}

/// Enables or disables the system tray icon
pub fn set_show_tray_icon(enabled: bool) -> Result<()> {
    let mut config = ConfigStore::load()?;
    config.show_tray_icon = enabled;
    ConfigStore::save_normalized(&mut config)?;
    info!("Set show tray icon to: {}", enabled);
    Ok(())
}

pub fn set_server_url(endpoint: String) -> Result<()> {
    let mut config = ConfigStore::load()?;
    apply_server_url(&mut config, &endpoint)?;
    ConfigStore::save_normalized(&mut config)?;
    Ok(())
}

// ── Capabilities ─────────────────────────────────────────────────────

/// Adds a capability to the config if it doesn't already exist.
pub fn add_capability(capability: &str) -> Result<()> {
    let mut config = ConfigStore::load()?;
    add_to_capabilities(&mut config.capabilities, capability);
    ConfigStore::save_normalized(&mut config)?;
    Ok(())
}

pub fn remove_capability(capability: &str) -> Result<()> {
    let mut config = ConfigStore::load()?;
    if let Some(pos) = config.capabilities.iter().position(|x| x == capability) {
        config.capabilities.remove(pos);
    }
    ConfigStore::save_normalized(&mut config)?;
    Ok(())
}

// ── Internal helpers ─────────────────────────────────────────────────

fn validate_server_url(endpoint: &str) -> Result<()> {
    if endpoint.is_empty() {
        return Ok(());
    }

    let parsed = Url::parse(endpoint).map_err(|e| eyre!("invalid server URL '{endpoint}': {e}"))?;

    match parsed.scheme() {
        "http" | "https" => {}
        other => {
            return Err(eyre!(
                "invalid server URL '{endpoint}': unsupported scheme '{other}' (expected http or https)"
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
    if !capabilities.contains(&capability.to_string()) {
        capabilities.push(capability.to_string());
    }
}

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::config::Config;

    // server url validation

    #[test]
    fn validate_allows_empty_endpoint() {
        assert!(validate_server_url("").is_ok());
    }

    #[test]
    fn validate_rejects_invalid_scheme() {
        let err =
            validate_server_url("ftp://example.com").expect_err("expected validation failure");
        assert!(err.to_string().contains("unsupported scheme"));
    }

    // apply server url

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
        let _err =
            apply_server_url(&mut config, "not a url").expect_err("expected validation failure");

        assert_eq!(config.server_url, "");
        assert_eq!(config.last_server_url, "");
        assert!(!config.has_remote_server);
    }

    // capabilities

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
