use crate::infra::home::Home;
use eyre::Result;

pub fn get_capabilities() -> Result<Vec<String>> {
    let home = Home::try_default()?;
    home.get_capabilities()
}
