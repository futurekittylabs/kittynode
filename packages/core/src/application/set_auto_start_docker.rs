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
    use crate::application::test_support::ConfigSandbox;
    use crate::infra::config::ConfigStore;

    #[test]
    fn set_auto_start_docker_persists_flag() {
        let _sandbox = ConfigSandbox::new();

        set_auto_start_docker(true).expect("enabling auto start should succeed");
        assert!(
            ConfigStore::load()
                .expect("config should load after enabling")
                .auto_start_docker,
            "auto_start_docker flag should be true after enabling"
        );

        set_auto_start_docker(false).expect("disabling auto start should succeed");
        assert!(
            !ConfigStore::load()
                .expect("config should load after disabling")
                .auto_start_docker,
            "auto_start_docker flag should be false after disabling"
        );
    }
}
