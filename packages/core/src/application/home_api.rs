use crate::infra::{config::ConfigStore, home::Home};
use eyre::Result;
use std::{fs, io::ErrorKind};
use tracing::info;

impl Home {
    pub fn init_kittynode(&self) -> Result<()> {
        let config = crate::domain::config::Config::default();
        ConfigStore::save_to(self, &config)?;
        Ok(())
    }

    pub fn get_server_url(&self) -> Result<String> {
        let config = ConfigStore::load_from(self)?;
        Ok(config.server_url)
    }

    pub fn set_server_url(&self, endpoint: String) -> Result<()> {
        let mut config = ConfigStore::load_from(self)?;
        config.server_url = endpoint;
        ConfigStore::save_to(self, &config)?;
        Ok(())
    }

    pub fn get_capabilities(&self) -> Result<Vec<String>> {
        let config = ConfigStore::load_from(self)?;
        Ok(config.capabilities)
    }

    pub fn add_capability(&self, capability: &str) -> Result<()> {
        let mut config = ConfigStore::load_from(self)?;
        if !config.capabilities.iter().any(|c| c == capability) {
            config.capabilities.push(capability.to_string());
        }
        ConfigStore::save_to(self, &config)?;
        Ok(())
    }

    pub fn remove_capability(&self, capability: &str) -> Result<()> {
        let mut config = ConfigStore::load_from(self)?;
        if let Some(pos) = config.capabilities.iter().position(|x| x == capability) {
            config.capabilities.remove(pos);
        }
        ConfigStore::save_to(self, &config)?;
        Ok(())
    }

    pub fn delete_kittynode(&self) -> Result<()> {
        if let Err(e) = fs::remove_dir_all(self.base())
            && e.kind() != ErrorKind::NotFound
        {
            return Err(e.into());
        }
        info!("Successfully deleted Kittynode");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn add_capability_no_duplicates() {
        let dir = tempdir().unwrap();
        let home = Home::from_base(dir.path().join(".kittynode"));

        home.add_capability("cap1").unwrap();
        home.add_capability("cap1").unwrap();
        let caps = home.get_capabilities().unwrap();
        assert_eq!(caps, vec!["cap1".to_string()]);
    }
}
