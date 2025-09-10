use crate::infra::home::Home;
use eyre::Result;

/// Initializes Kittynode with the default config
pub fn init_kittynode() -> Result<()> {
    let home = Home::try_default()?;
    home.init_kittynode()
}
