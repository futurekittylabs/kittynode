use crate::infra::home::Home;
use eyre::Result;

pub fn get_server_url() -> Result<String> {
    let home = Home::try_default()?;
    home.get_server_url()
}
