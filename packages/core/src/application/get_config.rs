use crate::domain::config::Config;
use crate::infra::config::ConfigStore;
use eyre::Result;

/// Returns the persisted Kittynode configuration
pub fn get_config() -> Result<Config> {
    ConfigStore::load()
}
