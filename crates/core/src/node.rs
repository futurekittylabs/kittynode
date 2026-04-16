use crate::config::{Config, get_config, write_config};
use crate::docker::{is_docker_running, start_docker};
use eyre::Result;
use std::{
    io::ErrorKind,
    sync::{LazyLock, Mutex},
};
use tracing::info;

#[derive(Copy, Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum OperationalMode {
    Local,
    Remote,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OperationalState {
    pub mode: OperationalMode,
    pub docker_running: bool,
    pub can_install: bool,
    pub can_manage: bool,
    pub diagnostics: Vec<String>,
}

#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DockerStartStatus {
    Running,
    Disabled,
    AlreadyStarted,
    Starting,
}

impl DockerStartStatus {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            DockerStartStatus::Running => "running",
            DockerStartStatus::Disabled => "disabled",
            DockerStartStatus::AlreadyStarted => "already_started",
            DockerStartStatus::Starting => "starting",
        }
    }
}

static DOCKER_AUTO_STARTED: LazyLock<Mutex<bool>> = LazyLock::new(|| Mutex::new(false));

#[derive(Debug, PartialEq, Eq)]
struct AutoStartEvaluation {
    status: DockerStartStatus,
    attempt_state: AttemptState,
    invoke_start: bool,
}

#[derive(Debug, PartialEq, Eq)]
enum AttemptState {
    Unchanged,
    Reset,
    MarkAttempted,
}

pub async fn get_operational_state() -> Result<OperationalState> {
    let config = get_config()?;
    let mode = determine_mode(&config);
    let docker_running = match mode {
        OperationalMode::Local => is_docker_running().await,
        OperationalMode::Remote => true,
    };

    Ok(compose_operational_state(mode, docker_running))
}

pub async fn start_docker_if_needed() -> Result<DockerStartStatus> {
    let config = get_config()?;
    let mode = determine_mode(&config);

    if matches!(mode, OperationalMode::Remote) {
        return Ok(DockerStartStatus::Running);
    }

    let docker_running = is_docker_running().await;
    let evaluation = {
        let mut attempted = DOCKER_AUTO_STARTED
            .lock()
            .expect("docker auto-start mutex poisoned");

        let evaluation =
            evaluate_local_auto_start(config.auto_start_docker, docker_running, *attempted);

        match evaluation.attempt_state {
            AttemptState::Reset => *attempted = false,
            AttemptState::MarkAttempted => *attempted = true,
            AttemptState::Unchanged => {}
        }

        if !evaluation.invoke_start {
            return Ok(evaluation.status);
        }

        drop(attempted);
        evaluation
    };

    info!("Starting Docker via auto-start preference");
    match start_docker().await {
        Ok(()) => Ok(evaluation.status),
        Err(error) => {
            let mut attempted = DOCKER_AUTO_STARTED
                .lock()
                .expect("docker auto-start mutex poisoned");
            *attempted = false;
            Err(error)
        }
    }
}

pub fn init_kittynode() -> Result<()> {
    let existing = get_config().unwrap_or_default();
    let fresh = reset_config_preserving_onboarding(existing);
    let onboarding_completed = fresh.onboarding_completed;
    write_config(fresh)?;

    info!(
        "Initialized Kittynode, preserved onboarding_completed: {}",
        onboarding_completed
    );
    Ok(())
}

fn reset_config_preserving_onboarding(existing: Config) -> Config {
    Config {
        onboarding_completed: existing.onboarding_completed,
        ..Default::default()
    }
}

pub fn delete_kittynode() -> Result<()> {
    if let Err(error) = std::fs::remove_dir_all(crate::paths::kittynode_path()?)
        && error.kind() != ErrorKind::NotFound
    {
        return Err(error.into());
    }

    info!("Successfully deleted Kittynode.");
    Ok(())
}

fn determine_mode(config: &Config) -> OperationalMode {
    if config.server_url.trim().is_empty() {
        OperationalMode::Local
    } else {
        OperationalMode::Remote
    }
}

fn compose_operational_state(mode: OperationalMode, docker_running: bool) -> OperationalState {
    match mode {
        OperationalMode::Local => {
            let diagnostics = if docker_running {
                Vec::new()
            } else {
                vec!["Docker is not running locally".to_string()]
            };

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

fn evaluate_local_auto_start(
    auto_start_enabled: bool,
    docker_running: bool,
    auto_start_attempted: bool,
) -> AutoStartEvaluation {
    if docker_running {
        return AutoStartEvaluation {
            status: DockerStartStatus::Running,
            attempt_state: AttemptState::Reset,
            invoke_start: false,
        };
    }

    if !auto_start_enabled {
        return AutoStartEvaluation {
            status: DockerStartStatus::Disabled,
            attempt_state: AttemptState::Unchanged,
            invoke_start: false,
        };
    }

    if auto_start_attempted {
        return AutoStartEvaluation {
            status: DockerStartStatus::AlreadyStarted,
            attempt_state: AttemptState::Unchanged,
            invoke_start: false,
        };
    }

    AutoStartEvaluation {
        status: DockerStartStatus::Starting,
        attempt_state: AttemptState::MarkAttempted,
        invoke_start: true,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        AttemptState, AutoStartEvaluation, DockerStartStatus, OperationalMode,
        compose_operational_state, determine_mode, evaluate_local_auto_start,
        reset_config_preserving_onboarding,
    };
    use crate::config::Config;

    #[test]
    fn determine_mode_discriminates_on_trimmed_server_url() {
        let local = Config {
            server_url: String::new(),
            ..Default::default()
        };
        let spaced = Config {
            server_url: "   ".to_string(),
            ..Default::default()
        };
        let remote = Config {
            server_url: "https://example.com".to_string(),
            ..Default::default()
        };

        assert!(matches!(determine_mode(&local), OperationalMode::Local));
        assert!(matches!(determine_mode(&spaced), OperationalMode::Local));
        assert!(matches!(determine_mode(&remote), OperationalMode::Remote));
    }

    #[test]
    fn compose_local_state_reflects_docker_availability() {
        let running = compose_operational_state(OperationalMode::Local, true);
        assert!(running.docker_running);
        assert!(running.can_install);
        assert!(running.can_manage);
        assert!(running.diagnostics.is_empty());

        let stopped = compose_operational_state(OperationalMode::Local, false);
        assert!(!stopped.docker_running);
        assert!(!stopped.can_install);
        assert!(!stopped.can_manage);
        assert_eq!(
            stopped.diagnostics,
            vec!["Docker is not running locally".to_string()]
        );
    }

    #[test]
    fn compose_remote_state_always_enables_capabilities() {
        let state = compose_operational_state(OperationalMode::Remote, false);
        assert!(state.docker_running);
        assert!(state.can_install);
        assert!(state.can_manage);
        assert!(state.diagnostics.is_empty());
    }

    #[test]
    fn running_docker_is_reported_and_resets_attempt() {
        let evaluation = evaluate_local_auto_start(true, true, true);

        assert_eq!(
            evaluation,
            AutoStartEvaluation {
                status: DockerStartStatus::Running,
                attempt_state: AttemptState::Reset,
                invoke_start: false,
            }
        );
    }

    #[test]
    fn disabled_auto_start_reflects_disabled_status() {
        let evaluation = evaluate_local_auto_start(false, false, false);

        assert_eq!(
            evaluation,
            AutoStartEvaluation {
                status: DockerStartStatus::Disabled,
                attempt_state: AttemptState::Unchanged,
                invoke_start: false,
            }
        );
    }

    #[test]
    fn repeated_attempt_returns_already_started_without_changes() {
        let evaluation = evaluate_local_auto_start(true, false, true);

        assert_eq!(
            evaluation,
            AutoStartEvaluation {
                status: DockerStartStatus::AlreadyStarted,
                attempt_state: AttemptState::Unchanged,
                invoke_start: false,
            }
        );
    }

    #[test]
    fn first_attempt_requests_docker_start_and_marks_attempt() {
        let evaluation = evaluate_local_auto_start(true, false, false);

        assert_eq!(
            evaluation,
            AutoStartEvaluation {
                status: DockerStartStatus::Starting,
                attempt_state: AttemptState::MarkAttempted,
                invoke_start: true,
            }
        );
    }

    #[test]
    fn docker_start_status_strings_match_expected_variants() {
        assert_eq!(DockerStartStatus::Running.as_str(), "running");
        assert_eq!(DockerStartStatus::Disabled.as_str(), "disabled");
        assert_eq!(
            DockerStartStatus::AlreadyStarted.as_str(),
            "already_started"
        );
        assert_eq!(DockerStartStatus::Starting.as_str(), "starting");
    }

    #[test]
    fn reset_config_preserves_only_onboarding() {
        let existing = Config {
            capabilities: vec!["ethereum".to_string()],
            server_url: "https://example.com".to_string(),
            last_server_url: "https://cached.example.com".to_string(),
            has_remote_server: true,
            onboarding_completed: true,
            auto_start_docker: true,
            show_tray_icon: false,
        };

        let reset = reset_config_preserving_onboarding(existing);

        assert!(reset.onboarding_completed);
        assert!(reset.capabilities.is_empty());
        assert_eq!(reset.server_url, "");
        assert_eq!(reset.last_server_url, "");
        assert!(!reset.has_remote_server);
        assert!(!reset.auto_start_docker);
        assert!(reset.show_tray_icon);
    }
}
