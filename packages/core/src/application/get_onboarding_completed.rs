use crate::infra::config::ConfigStore;
use eyre::Result;
use tracing::info;

pub fn get_onboarding_completed() -> Result<bool> {
    let config = ConfigStore::load()?;
    info!(
        "Retrieved onboarding completed status: {}",
        config.onboarding_completed
    );
    Ok(config.onboarding_completed)
}
