use crate::infra::config::ConfigStore;
use eyre::Result;

pub fn get_server_url() -> Result<String> {
    let config = ConfigStore::load()?;
    Ok(config.server_url)
}
