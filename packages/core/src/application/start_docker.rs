use eyre::Result;
use std::process::Command;
use tracing::info;

pub async fn start_docker() -> Result<()> {
    info!("Attempting to start Docker Desktop");

    #[cfg(target_os = "macos")]
    {
        Command::new("open").arg("-a").arg("Docker").spawn()?;
    }

    #[cfg(target_os = "linux")]
    {
        // Try systemctl first (for systemd-based distros)
        if Command::new("systemctl")
            .arg("--user")
            .arg("start")
            .arg("docker-desktop")
            .spawn()
            .is_err()
        {
            // Fallback to direct launch
            Command::new("docker-desktop").spawn()?;
        }
    }

    #[cfg(target_os = "windows")]
    {
        Command::new("cmd")
            .args(&["/C", "start", "", "Docker Desktop.exe"])
            .spawn()?;
    }

    Ok(())
}
