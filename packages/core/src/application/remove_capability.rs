use crate::infra::config::ConfigStore;
use eyre::Result;

pub fn remove_capability(capability: &str) -> Result<()> {
    let mut config = ConfigStore::load()?;
    if let Some(pos) = config.capabilities.iter().position(|x| x == capability) {
        config.capabilities.remove(pos);
    }
    ConfigStore::save(&mut config)?;
    Ok(())
}
