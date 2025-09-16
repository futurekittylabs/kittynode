use crate::infra::config::ConfigStore;
use eyre::Result;

pub fn set_onboarding_completed(completed: bool) -> Result<()> {
    let mut config = ConfigStore::load()?;
    config.onboarding_completed = completed;
    ConfigStore::save(&config)?;
    Ok(())
}
