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
