use crate::application::install_package;
use crate::domain::package::PackageConfig;
use crate::infra::package as infra_package;
use crate::infra::package_config::PackageConfigStore;
use eyre::Result;

pub async fn update_package_config(package_name: &str, config: PackageConfig) -> Result<()> {
    // Take a snapshot of the pre-update package definition so we can remove
    // containers that may be dropped by the new config (e.g., validator).
    let pre_update_package = infra_package::get_package_by_name(package_name)?;

    // Persist the new configuration first so that any fallback install paths
    // (e.g., when deletion hits missing resources) can read the intended config.
    PackageConfigStore::save(package_name, &config)?;

    // Best-effort removal using the pre-update snapshot; tolerate missing Docker
    // resources to keep reconfiguration robust on first-time installs.
    if let Err(err) = infra_package::delete_package(&pre_update_package, false, false).await {
        let msg = err.to_string().to_lowercase();
        let missing = msg.contains("no such volume")
            || (msg.contains("volume") && msg.contains("not found"))
            || msg.contains("no such network")
            || (msg.contains("network") && msg.contains("not found"));
        if !missing {
            return Err(err);
        }
    }

    // Install using the newly-saved configuration
    install_package(package_name).await?;

    Ok(())
}
