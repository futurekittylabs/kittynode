use crate::infra::config::ConfigStore;
use eyre::Result;

pub fn set_server_url(endpoint: String) -> Result<()> {
    let mut config = ConfigStore::load()?;
    let trimmed = endpoint.trim();

    if trimmed.is_empty() {
        config.server_url.clear();
        config.remote_connected = false;
    } else {
        let normalized = trimmed.to_string();
        config.server_url = normalized.clone();
        config.last_server_url = normalized;
        config.remote_connected = true;
    }

    ConfigStore::save(&config)?;
    Ok(())
}
