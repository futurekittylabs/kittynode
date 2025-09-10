use crate::infra::home::Home;
use eyre::Result;

pub fn set_server_url(endpoint: String) -> Result<()> {
    let home = Home::try_default()?;
    home.set_server_url(endpoint)?;
    Ok(())
}
