use crate::application::install_package;
use crate::domain::package::PackageConfig;
use crate::infra::package as infra_package;
use crate::infra::package_config::PackageConfigStore;
use eyre::Result;

pub async fn update_package_config(package_name: &str, config: PackageConfig) -> Result<()> {
    // Take a snapshot of the pre-update package definition so we can remove
    // containers that may be dropped by the new config (e.g., validator).
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

    // Load existing config and perform a shallow overlay with the incoming keys
    // to avoid clobbering user-provided settings. This is deterministic and
    // idempotent: repeated calls with the same inputs yield the same result.
    let mut merged = PackageConfigStore::load(package_name)?;
    for (k, v) in config.values.into_iter() {
        merged.values.insert(k, v);
    }

    // Persist the merged configuration so any fallback install paths can read it
    PackageConfigStore::save(package_name, &merged)?;

    // Best-effort removal using the pre-update snapshot; tolerate missing Docker
    // resources to keep reconfiguration robust on first-time installs.
    if let Some(package) = &pre_update_package {
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

    // Install using the newly-saved configuration
    install_package(package_name).await?;

    Ok(())
}
