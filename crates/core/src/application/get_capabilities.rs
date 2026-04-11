use crate::infra::config::ConfigStore;
use eyre::Result;

pub fn get_capabilities() -> Result<Vec<String>> {
    let config = ConfigStore::load()?;
    Ok(config.capabilities)
}
