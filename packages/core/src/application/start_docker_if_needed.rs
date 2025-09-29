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
struct AutoStartDecision {
    status: DockerStartStatus,
    attempt_change: AttemptChange,
    should_start: bool,
}

#[derive(Debug, PartialEq, Eq)]
enum AttemptChange {
    Reset,
    Record,
    None,
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

    let (status, should_start) = {
        let mut attempted = DOCKER_AUTO_STARTED
            .lock()
            .expect("docker auto-start mutex poisoned");

        let decision =
            decide_local_auto_start(config.auto_start_docker, docker_running, *attempted);

        match decision.attempt_change {
            AttemptChange::Reset => *attempted = false,
            AttemptChange::Record => *attempted = true,
            AttemptChange::None => {}
        }

        (decision.status, decision.should_start)
    };

    if !should_start {
        return Ok(status);
    }

    info!("Starting Docker Desktop via auto-start preference");
    match start_docker().await {
        Ok(()) => Ok(status),
        Err(err) => {
            let mut attempted = DOCKER_AUTO_STARTED
                .lock()
                .expect("docker auto-start mutex poisoned");
            *attempted = false;
            Err(err)
        }
    }
}

fn decide_local_auto_start(
    auto_start_enabled: bool,
    docker_running: bool,
    already_attempted: bool,
) -> AutoStartDecision {
    if docker_running {
        return AutoStartDecision {
            status: DockerStartStatus::Running,
            attempt_change: AttemptChange::Reset,
            should_start: false,
        };
    }

    if !auto_start_enabled {
        return AutoStartDecision {
            status: DockerStartStatus::Disabled,
            attempt_change: AttemptChange::None,
            should_start: false,
        };
    }

    if already_attempted {
        return AutoStartDecision {
            status: DockerStartStatus::AlreadyStarted,
            attempt_change: AttemptChange::None,
            should_start: false,
        };
    }

    AutoStartDecision {
        status: DockerStartStatus::Starting,
        attempt_change: AttemptChange::Record,
        should_start: true,
    }
}

#[cfg(test)]
mod tests {
    use super::{AttemptChange, DockerStartStatus, decide_local_auto_start};

    #[test]
    fn running_resets_attempt_without_starting() {
        let decision = decide_local_auto_start(true, true, true);
        assert_eq!(decision.status, DockerStartStatus::Running);
        assert_eq!(decision.attempt_change, AttemptChange::Reset);
        assert!(!decision.should_start);
    }

    #[test]
    fn disabled_auto_start_skips_starting() {
        let decision = decide_local_auto_start(false, false, false);
        assert_eq!(decision.status, DockerStartStatus::Disabled);
        assert_eq!(decision.attempt_change, AttemptChange::None);
        assert!(!decision.should_start);
    }

    #[test]
    fn repeated_attempt_reports_already_started() {
        let decision = decide_local_auto_start(true, false, true);
        assert_eq!(decision.status, DockerStartStatus::AlreadyStarted);
        assert_eq!(decision.attempt_change, AttemptChange::None);
        assert!(!decision.should_start);
    }

    #[test]
    fn first_attempt_requests_start_and_records_it() {
        let decision = decide_local_auto_start(true, false, false);
        assert_eq!(decision.status, DockerStartStatus::Starting);
        assert_eq!(decision.attempt_change, AttemptChange::Record);
        assert!(decision.should_start);
    }
}
