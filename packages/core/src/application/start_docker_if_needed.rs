use crate::application::is_docker_running;
use crate::application::start_docker;
use crate::domain::operational_state::OperationalMode;
use crate::infra::config::ConfigStore;
use eyre::Result;
use std::sync::{LazyLock, Mutex};
use tracing::info;

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

// Encapsulates the branching logic for starting Docker while keeping the outer async function slim and testable.
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

/// Attempts to start Docker if the configuration allows auto start and Docker is not already running.
pub async fn start_docker_if_needed() -> Result<DockerStartStatus> {
    let config = ConfigStore::load()?;
    let mode = if config.server_url.trim().is_empty() {
        OperationalMode::Local
    } else {
        OperationalMode::Remote
    };

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

    info!("Starting Docker Desktop via auto-start preference");
    match start_docker().await {
        Ok(()) => Ok(evaluation.status),
        Err(err) => {
            let mut attempted = DOCKER_AUTO_STARTED
                .lock()
                .expect("docker auto-start mutex poisoned");
            *attempted = false;
            Err(err)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::evaluate_local_auto_start;
    use super::{AttemptState, AutoStartEvaluation, DockerStartStatus};

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
}
