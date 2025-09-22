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
