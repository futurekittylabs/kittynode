use crate::application::is_docker_running;
use crate::domain::operational_state::{OperationalMode, OperationalState};
use crate::infra::config::ConfigStore;
use eyre::Result;

/// Returns the operational state describing runtime capabilities for the current mode.
pub async fn get_operational_state() -> Result<OperationalState> {
    let config = ConfigStore::load()?;
    let mode = determine_mode(&config.server_url);
    let docker_running = match mode {
        OperationalMode::Local => is_docker_running().await,
        OperationalMode::Remote => true,
    };

    Ok(compose_operational_state(mode, docker_running))
}

fn determine_mode(server_url: &str) -> OperationalMode {
    if server_url.trim().is_empty() {
        OperationalMode::Local
    } else {
        OperationalMode::Remote
    }
}

fn compose_operational_state(mode: OperationalMode, docker_running: bool) -> OperationalState {
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
    fn determine_mode_discriminates_on_trimmed_server_url() {
        assert!(matches!(determine_mode(""), OperationalMode::Local));
        assert!(matches!(determine_mode("   "), OperationalMode::Local));
        assert!(matches!(
            determine_mode("https://example.com"),
            OperationalMode::Remote
        ));
    }

    #[test]
    fn compose_local_state_reflects_docker_availability() {
        let running = compose_operational_state(OperationalMode::Local, true);
        assert!(running.docker_running, "docker_running should passthrough");
        assert!(
            running.can_install,
            "install should be allowed when running"
        );
        assert!(running.can_manage, "manage should be allowed when running");
        assert!(
            running.diagnostics.is_empty(),
            "no diagnostics when running"
        );

        let stopped = compose_operational_state(OperationalMode::Local, false);
        assert!(!stopped.docker_running, "docker_running should passthrough");
        assert!(
            !stopped.can_install,
            "install should be blocked when stopped"
        );
        assert!(!stopped.can_manage, "manage should be blocked when stopped");
        assert_eq!(
            stopped.diagnostics,
            vec!["Docker is not running locally".to_string()],
            "diagnostics should explain the failure"
        );
    }

    #[test]
    fn compose_remote_state_always_enables_capabilities() {
        let state = compose_operational_state(OperationalMode::Remote, false);
        assert!(
            state.docker_running,
            "remote mode always reports docker running"
        );
        assert!(state.can_install, "remote mode can install");
        assert!(state.can_manage, "remote mode can manage");
        assert!(
            state.diagnostics.is_empty(),
            "remote mode should not emit diagnostics"
        );
    }
}
