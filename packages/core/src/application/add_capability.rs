use crate::infra::home::Home;
use eyre::Result;

/// Adds a capability to the config if it doesn't already exist.
pub fn add_capability(capability: &str) -> Result<()> {
    let home = Home::try_default()?;
    home.add_capability(capability)?;
    Ok(())
}
