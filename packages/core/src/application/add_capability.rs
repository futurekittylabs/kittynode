use crate::infra::config::ConfigStore;
use eyre::Result;

/// Adds a capability to the config if it doesn't already exist.
pub fn add_capability(capability: &str) -> Result<()> {
    let mut config = ConfigStore::load()?;
    add_to_capabilities(&mut config.capabilities, capability);
    ConfigStore::save_normalized(&mut config)?;
    Ok(())
}

/// Helper function to add a capability to the list, ensuring no duplicates.
fn add_to_capabilities(capabilities: &mut Vec<String>, capability: &str) {
    if !capabilities.contains(&capability.to_string()) {
        capabilities.push(capability.to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn doesnt_add_duplicate() {
        let mut capabilities = vec!["cap1".to_string(), "cap2".to_string()];
        add_to_capabilities(&mut capabilities, "cap1");
        assert_eq!(capabilities, vec!["cap1".to_string(), "cap2".to_string()]);
    }

    #[test]
    fn adds_new_capability() {
        let mut capabilities = vec!["cap1".to_string(), "cap2".to_string()];
        add_to_capabilities(&mut capabilities, "cap3");
        assert_eq!(
            capabilities,
            vec!["cap1".to_string(), "cap2".to_string(), "cap3".to_string()]
        );
    }
}
