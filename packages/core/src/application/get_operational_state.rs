use crate::application::is_docker_running;
use crate::domain::operational_state::{OperationalMode, OperationalState};
use crate::infra::config::ConfigStore;
use eyre::Result;

/// Returns the operational state describing runtime capabilities for the current mode.
pub async fn get_operational_state() -> Result<OperationalState> {
    let config = ConfigStore::load()?;
    let mode = if config.server_url.trim().is_empty() {
        OperationalMode::Local
    } else {
        OperationalMode::Remote
    };

    let (docker_running, can_install, can_manage, diagnostics) = match mode {
        OperationalMode::Local => {
            let running = is_docker_running().await;
            let mut diagnostics = Vec::new();
            if !running {
                diagnostics.push("Docker is not running locally".to_string());
            }
            (running, running, running, diagnostics)
        }
        OperationalMode::Remote => (true, true, true, Vec::new()),
    };

    Ok(OperationalState {
        mode,
        docker_running,
        can_install,
        can_manage,
        diagnostics,
    })
}
