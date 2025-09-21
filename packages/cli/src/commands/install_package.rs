use eyre::Result;
use kittynode_core::api::install_package;

pub async fn install_package_cmd(name: String) -> Result<()> {
    install_package(&name).await
}
