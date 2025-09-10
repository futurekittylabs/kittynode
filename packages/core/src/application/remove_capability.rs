use crate::infra::home::Home;
use eyre::Result;

pub fn remove_capability(capability: &str) -> Result<()> {
    let home = Home::try_default()?;
    home.remove_capability(capability)?;
    Ok(())
}
