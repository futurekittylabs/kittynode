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

    let docker_running = if matches!(mode, OperationalMode::Local) {
        is_docker_running().await
    } else {
        true
    };

    Ok(build_operational_state(mode, docker_running))
}

fn build_operational_state(mode: OperationalMode, docker_running: bool) -> OperationalState {
    match mode {
        OperationalMode::Local => {
            let mut diagnostics = Vec::new();
            if !docker_running {
                diagnostics.push("Docker is not running locally".to_string());
            }
            OperationalState {
                mode,
                docker_running,
                can_install: docker_running,
                can_manage: docker_running,
                diagnostics,
            }
        }
        OperationalMode::Remote => OperationalState {
            mode,
            docker_running: true,
            can_install: true,
            can_manage: true,
            diagnostics: Vec::new(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_mode_reports_when_docker_is_stopped() {
        let state = build_operational_state(OperationalMode::Local, false);

        assert_eq!(state.mode, OperationalMode::Local);
        assert!(!state.docker_running);
        assert!(!state.can_install);
        assert!(!state.can_manage);
        assert_eq!(
            state.diagnostics,
            vec!["Docker is not running locally".to_string()]
        );
    }

    #[test]
    fn local_mode_with_running_docker_enables_management() {
        let state = build_operational_state(OperationalMode::Local, true);

        assert!(state.diagnostics.is_empty());
        assert!(state.docker_running);
        assert!(state.can_install);
        assert!(state.can_manage);
    }

    #[test]
    fn remote_mode_allows_management_without_local_docker() {
        let state = build_operational_state(OperationalMode::Remote, false);

        assert_eq!(state.mode, OperationalMode::Remote);
        assert!(state.docker_running);
        assert!(state.can_install);
        assert!(state.can_manage);
        assert!(state.diagnostics.is_empty());
    }
}
