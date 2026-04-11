use eyre::Result;
use kittynode_core::api::install_package_with_network;

pub async fn install_package_cmd(name: String, network: Option<&str>) -> Result<()> {
    install_package_with_network(&name, network).await
}
