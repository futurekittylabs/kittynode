use crate::infra::config::ConfigStore;
use eyre::Result;

pub fn get_onboarding_completed() -> Result<bool> {
    let config = ConfigStore::load()?;
    Ok(config.onboarding_completed)
}
