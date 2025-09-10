use crate::infra::home::Home;
use eyre::Result;

/// Deletes the Kittynode config directory
pub fn delete_kittynode() -> Result<()> {
    let home = Home::try_default()?;
    home.delete_kittynode()
}
