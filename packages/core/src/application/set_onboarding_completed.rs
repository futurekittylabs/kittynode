use crate::infra::config::ConfigStore;
use eyre::Result;
use tracing::info;

pub fn set_onboarding_completed(completed: bool) -> Result<()> {
    let mut config = ConfigStore::load()?;
    config.onboarding_completed = completed;
    ConfigStore::save_normalized(&mut config)?;
    info!("Set onboarding completed to: {}", completed);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::set_onboarding_completed;
    use crate::domain::config::Config;
    use crate::infra::{config::ConfigStore, file::with_kittynode_path_override};
    use tempfile::tempdir;

    #[test]
    fn marks_onboarding_complete_in_fresh_config() {
        let temp = tempdir().unwrap();

        with_kittynode_path_override(temp.path(), || {
            set_onboarding_completed(true).expect("setting flag should succeed");

            let persisted = ConfigStore::load().expect("config should load");
            assert!(persisted.onboarding_completed);
        });
    }

    #[test]
    fn preserves_existing_configuration_values() {
        let temp = tempdir().unwrap();

        with_kittynode_path_override(temp.path(), || {
            let mut existing = Config {
                server_url: "https://example.com".into(),
                last_server_url: "https://example.com".into(),
                has_remote_server: true,
                onboarding_completed: true,
                ..Config::default()
            };
            ConfigStore::save_normalized(&mut existing).expect("initial save should succeed");

            set_onboarding_completed(false).expect("setting flag should succeed");

            let persisted = ConfigStore::load().expect("config should load");
            assert!(!persisted.onboarding_completed);
            assert_eq!(persisted.server_url, "https://example.com");
            assert!(persisted.has_remote_server);
        });
    }
}
