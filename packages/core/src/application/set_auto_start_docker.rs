use crate::infra::config::ConfigStore;
use eyre::Result;
use tracing::info;

/// Enables or disables automatic Docker startup at application launch
pub fn set_auto_start_docker(enabled: bool) -> Result<()> {
    let mut config = ConfigStore::load()?;
    config.auto_start_docker = enabled;
    ConfigStore::save_normalized(&mut config)?;
    info!("Set auto start docker to: {}", enabled);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::set_auto_start_docker;
    use crate::domain::config::Config;
    use crate::infra::{
        config::ConfigStore,
        file::with_kittynode_path_override,
    };
    use tempfile::tempdir;

    #[test]
    fn enables_auto_start_on_fresh_config() {
        let temp = tempdir().unwrap();

        with_kittynode_path_override(temp.path(), || {
            set_auto_start_docker(true).expect("setting flag should succeed");

            let persisted = ConfigStore::load().expect("config should load");
            assert!(persisted.auto_start_docker);
            assert!(!persisted.has_remote_server);
            assert_eq!(persisted.server_url, "");
        });
    }

    #[test]
    fn updates_only_auto_start_flag() {
        let temp = tempdir().unwrap();

        with_kittynode_path_override(temp.path(), || {
            let mut existing = Config {
                server_url: "https://example.com".into(),
                last_server_url: "https://example.com".into(),
                has_remote_server: true,
                auto_start_docker: true,
                ..Config::default()
            };
            ConfigStore::save_normalized(&mut existing).expect("initial save should succeed");

            set_auto_start_docker(false).expect("setting flag should succeed");

            let persisted = ConfigStore::load().expect("config should load");
            assert!(!persisted.auto_start_docker);
            assert_eq!(persisted.server_url, "https://example.com");
            assert_eq!(persisted.last_server_url, "https://example.com");
            assert!(persisted.has_remote_server);
        });
    }
}
