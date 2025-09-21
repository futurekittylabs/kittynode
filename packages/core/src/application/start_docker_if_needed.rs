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

    if is_docker_running().await {
        let mut attempted = DOCKER_AUTO_STARTED
            .lock()
            .expect("docker auto-start mutex poisoned");
        *attempted = false;
        return Ok(DockerStartStatus::Running);
    }
    if !config.auto_start_docker {
        return Ok(DockerStartStatus::Disabled);
    }

    {
        let mut attempted = DOCKER_AUTO_STARTED
            .lock()
            .expect("docker auto-start mutex poisoned");
        if *attempted {
            return Ok(DockerStartStatus::AlreadyStarted);
        }
        *attempted = true;
    }

    info!("Starting Docker Desktop via auto-start preference");
    match start_docker().await {
        Ok(()) => Ok(DockerStartStatus::Starting),
        Err(err) => {
            let mut attempted = DOCKER_AUTO_STARTED
                .lock()
                .expect("docker auto-start mutex poisoned");
            *attempted = false;
            Err(err)
        }
    }
}
