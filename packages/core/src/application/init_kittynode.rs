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
    let config = Config {
        onboarding_completed,
        ..Default::default()
    };

    ConfigStore::save(&config)?;
    info!(
        "Initialized Kittynode, preserved onboarding_completed: {}",
        onboarding_completed
    );
    Ok(())
}
