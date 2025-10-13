use crate::application::install_package;
use crate::domain::package::PackageConfig;
use crate::infra::package as infra_package;
use crate::infra::package_config::PackageConfigStore;
use eyre::Result;

pub async fn update_package_config(package_name: &str, config: PackageConfig) -> Result<()> {
    // Capture the current concrete package so we can remove stale containers after the update.
    let pre_update_package = match infra_package::get_package_by_name(package_name) {
        Ok(package) => Some(package),
        Err(err) => {
            let message = err.to_string();
            let unsupported_network = message.contains("Unsupported Ethereum network");
            if unsupported_network {
                None
            } else {
                return Err(err);
            }
        }
    };

    // Overlay incoming keys without clobbering unrelated user-provided values.
    let mut merged = PackageConfigStore::load(package_name)?;
    for (k, v) in config.values.into_iter() {
        merged.values.insert(k, v);
    }

    PackageConfigStore::save(package_name, &merged)?;

    if let Some(package) = &pre_update_package {
        // Best-effort cleanup: tolerate missing Docker resources on first-time installs.
        match infra_package::delete_package(package, false, false).await {
            Ok(_) => {}
            Err(err) => {
                let msg = err.to_string().to_lowercase();
                let missing = msg.contains("no such volume")
                    || (msg.contains("volume") && msg.contains("not found"))
                    || msg.contains("no such network")
                    || (msg.contains("network") && msg.contains("not found"));
                if !missing {
                    return Err(err);
                }
            }
        }
    }

    // Reinstall using the saved configuration so changes take effect.
    install_package(package_name).await?;

    Ok(())
}
