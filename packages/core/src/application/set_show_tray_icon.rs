use crate::infra::config::ConfigStore;
use eyre::Result;
use tracing::info;

/// Enables or disables the system tray icon
pub fn set_show_tray_icon(enabled: bool) -> Result<()> {
    let mut config = ConfigStore::load()?;
    config.show_tray_icon = enabled;
    ConfigStore::save_normalized(&mut config)?;
    info!("Set show tray icon to: {}", enabled);
    Ok(())
}
