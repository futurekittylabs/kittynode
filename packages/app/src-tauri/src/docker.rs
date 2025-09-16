use eyre::{Result, WrapErr, eyre};
use kittynode_core::application;
use serde::Serialize;
use std::{
    env,
    path::PathBuf,
    process::Command,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};
use tokio::time::sleep;
use tracing::{error, info};

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DockerStatus {
    Unknown,
    NotInstalled,
    Starting,
    Running,
    NotRunning,
}

impl Default for DockerStatus {
    fn default() -> Self {
        Self::Unknown
    }
}

#[derive(Clone)]
pub struct DockerManager {
    status: Arc<Mutex<DockerStatus>>,
    initialization_started: Arc<AtomicBool>,
}

impl Default for DockerManager {
    fn default() -> Self {
        Self {
            status: Arc::new(Mutex::new(DockerStatus::Unknown)),
            initialization_started: Arc::new(AtomicBool::new(false)),
        }
    }
}

#[derive(Clone)]
enum DockerDesktopInstall {
    #[cfg(target_os = "macos")]
    Mac,
    #[cfg(target_os = "windows")]
    Windows(PathBuf),
    #[cfg(target_os = "linux")]
    Linux,
}

impl DockerManager {
    pub async fn current_status(&self) -> DockerStatus {
        self.ensure_initialized().await;
        let status = self.get_status();

        match status {
            DockerStatus::Starting => {
                if application::is_docker_running().await {
                    self.set_status(DockerStatus::Running);
                } else if !Self::is_docker_desktop_installed() {
                    self.set_status(DockerStatus::NotInstalled);
                }
            }
            DockerStatus::Running => {
                if !application::is_docker_running().await {
                    self.set_status(DockerStatus::NotRunning);
                }
            }
            DockerStatus::NotRunning => {
                if application::is_docker_running().await {
                    self.set_status(DockerStatus::Running);
                }
            }
            DockerStatus::NotInstalled => {
                if application::is_docker_running().await {
                    self.set_status(DockerStatus::Running);
                }
            }
            DockerStatus::Unknown => {
                // Should not happen after initialization, but handle gracefully
                if application::is_docker_running().await {
                    self.set_status(DockerStatus::Running);
                } else if !Self::is_docker_desktop_installed() {
                    self.set_status(DockerStatus::NotInstalled);
                } else {
                    self.set_status(DockerStatus::NotRunning);
                }
            }
        }

        self.get_status()
    }

    pub async fn ensure_initialized(&self) {
        if self
            .initialization_started
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_err()
        {
            return;
        }

        if application::is_docker_running().await {
            info!("Docker is already running");
            self.set_status(DockerStatus::Running);
            return;
        }

        let installation = match Self::detect_installation() {
            Some(installation) => installation,
            None => {
                info!("Docker Desktop not detected");
                self.set_status(DockerStatus::NotInstalled);
                return;
            }
        };

        info!("Starting Docker Desktop in background");
        self.set_status(DockerStatus::Starting);

        let manager = self.clone();
        tauri::async_runtime::spawn(async move {
            if let Err(err) = manager.start_and_wait(installation).await {
                error!("Failed to start Docker Desktop: {err}");
                manager.set_status(DockerStatus::NotRunning);
            }
        });
    }

    fn get_status(&self) -> DockerStatus {
        *self.status.lock().expect("status mutex poisoned")
    }

    fn set_status(&self, status: DockerStatus) {
        if let Ok(mut guard) = self.status.lock() {
            *guard = status;
        }
    }

    async fn start_and_wait(&self, installation: DockerDesktopInstall) -> Result<()> {
        Self::launch_docker_desktop(&installation)?;

        let mut elapsed = Duration::default();
        let timeout = Duration::from_secs(60);
        let poll_interval = Duration::from_secs(2);

        while elapsed <= timeout {
            if application::is_docker_running().await {
                info!("Docker Desktop is running");
                self.set_status(DockerStatus::Running);
                return Ok(());
            }

            sleep(poll_interval).await;
            elapsed += poll_interval;
        }

        Err(eyre!("Timed out waiting for Docker Desktop to start"))
    }

    fn launch_docker_desktop(installation: &DockerDesktopInstall) -> Result<()> {
        match installation {
            #[cfg(target_os = "macos")]
            DockerDesktopInstall::Mac => {
                Command::new("open")
                    .args(["--background", "-a", "Docker"])
                    .spawn()
                    .wrap_err("Failed to start Docker Desktop on macOS")?;
            }
            #[cfg(target_os = "windows")]
            DockerDesktopInstall::Windows(path) => {
                Command::new(path)
                    .arg("--background")
                    .spawn()
                    .wrap_err("Failed to start Docker Desktop on Windows")?;
            }
            #[cfg(target_os = "linux")]
            DockerDesktopInstall::Linux => {
                Command::new("systemctl")
                    .args(["--user", "start", "docker-desktop"])
                    .spawn()
                    .wrap_err("Failed to start Docker Desktop on Linux")?;
            }
        }

        Ok(())
    }

    fn detect_installation() -> Option<DockerDesktopInstall> {
        #[cfg(target_os = "macos")]
        {
            let mut paths = vec![PathBuf::from("/Applications/Docker.app")];
            if let Some(home) = home_dir() {
                paths.push(home.join("Applications").join("Docker.app"));
            }

            if paths.iter().any(|path| path.exists()) {
                return Some(DockerDesktopInstall::Mac);
            }

            None
        }

        #[cfg(target_os = "windows")]
        {
            let candidates = [
                PathBuf::from(r"C:\\Program Files\\Docker\\Docker\\Docker Desktop.exe"),
                PathBuf::from(r"C:\\Program Files (x86)\\Docker\\Docker\\Docker Desktop.exe"),
            ];

            for path in candidates {
                if path.exists() {
                    return Some(DockerDesktopInstall::Windows(path));
                }
            }

            None
        }

        #[cfg(target_os = "linux")]
        {
            let mut candidates = vec![
                PathBuf::from("/usr/lib/systemd/user/docker-desktop.service"),
                PathBuf::from("/usr/lib/systemd/user/docker-desktop.socket"),
                PathBuf::from("/usr/bin/docker-desktop"),
            ];

            if let Some(home) = home_dir() {
                candidates.push(home.join(".config/systemd/user/docker-desktop.service"));
                candidates.push(home.join(".local/share/applications/docker-desktop.desktop"));
            }

            if candidates.iter().any(|path| path.exists()) {
                return Some(DockerDesktopInstall::Linux);
            }

            None
        }

        #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
        {
            None
        }
    }

    fn is_docker_desktop_installed() -> bool {
        Self::detect_installation().is_some()
    }
}

fn home_dir() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        env::var("USERPROFILE").ok().map(PathBuf::from)
    }

    #[cfg(not(target_os = "windows"))]
    {
        env::var("HOME").ok().map(PathBuf::from)
    }
}
