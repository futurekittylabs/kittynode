use eyre::Result;
use kittynode_core::api::get_packages;

pub async fn get_packages_cmd() -> Result<()> {
    let packages = get_packages()?;
    for package in packages.values() {
        println!("{}", package);
    }
    Ok(())
}
