use crate::domain::config::Config;
use crate::infra::config::ConfigStore;
use eyre::Result;
use tracing::info;

/// Initializes Kittynode, preserving onboarding state
pub fn init_kittynode() -> Result<()> {
    // Load existing config to preserve onboarding_completed
    let existing_config = ConfigStore::load().unwrap_or_default();
    let onboarding_completed = existing_config.onboarding_completed;

    // Create fresh config but preserve onboarding state
    let mut config = Config {
        onboarding_completed,
        ..Default::default()
    };

    ConfigStore::save_normalized(&mut config)?;
    info!(
        "Initialized Kittynode, preserved onboarding_completed: {}",
        onboarding_completed
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::init_kittynode;
    use crate::domain::config::Config;
    use crate::infra::config::ConfigStore;
    use std::env;
    use tempfile::tempdir;

    fn with_temp_home<F, R>(f: F) -> R
    where
        F: FnOnce(&std::path::Path) -> R,
    {
        let _lock = crate::ENV_GUARD.lock().expect("lock poisoned");
        let original_home = env::var_os("HOME");
        let temp = tempdir().expect("tempdir failed");
        // Newer toolchains mark environment mutation unsafe; scope it with a guard.
        unsafe {
            env::set_var("HOME", temp.path());
        }
        let result = f(temp.path());
        match original_home {
            Some(value) => unsafe {
                env::set_var("HOME", value);
            },
            None => unsafe {
                env::remove_var("HOME");
            },
        }
        result
    }

    #[test]
    fn creates_default_config_when_missing() {
        with_temp_home(|_| {
            init_kittynode().expect("init should succeed");
            let config = ConfigStore::load().expect("config should load");
            assert!(!config.onboarding_completed);
            assert_eq!(config.server_url, "");
            assert_eq!(config.last_server_url, "");
            assert!(!config.has_remote_server);
        });
    }

    #[test]
    fn preserves_existing_onboarding_flag() {
        with_temp_home(|_| {
            let mut config = Config {
                onboarding_completed: true,
                server_url: "https://example.com".to_string(),
                last_server_url: "https://example.com".to_string(),
                has_remote_server: true,
                auto_start_docker: true,
                ..Default::default()
            };
            ConfigStore::save_normalized(&mut config).expect("seed config");

            init_kittynode().expect("init should succeed");

            let config = ConfigStore::load().expect("config should load");
            assert!(config.onboarding_completed);
            assert_eq!(config.server_url, "");
            assert_eq!(config.last_server_url, "");
            assert!(!config.has_remote_server);
            assert!(!config.auto_start_docker);
        });
    }
}
