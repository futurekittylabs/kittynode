use eyre::{Result, eyre};
use std::process::Command;
use tracing::{error, info};

#[cfg(target_os = "windows")]
use std::path::Path;

pub async fn start_docker() -> Result<()> {
    info!("Attempting to start Docker Desktop");

    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg("-a")
            .arg("Docker")
            .spawn()
            .map_err(|e| {
                error!("Failed to start Docker Desktop on macOS: {}", e);
                eyre!("Failed to start Docker Desktop: {}. Please ensure Docker Desktop is installed.", e)
            })?;
        info!("Docker Desktop start command sent on macOS");
    }

    #[cfg(target_os = "linux")]
    {
        let mut started = false;

        // Try different installation methods
        let attempts = [
            // System service
            ("systemctl", vec!["start", "docker-desktop"]),
            // User service
            ("systemctl", vec!["--user", "start", "docker-desktop"]),
            // Flatpak
            ("flatpak", vec!["run", "com.docker.DockerDesktop"]),
            // Direct binary in common locations
            ("docker-desktop", vec![]),
            ("/usr/bin/docker-desktop", vec![]),
            ("/opt/docker-desktop/docker-desktop", vec![]),
        ];

        for (cmd, args) in attempts {
            if Command::new(cmd).args(&args).spawn().is_ok() {
                info!("Started Docker Desktop using: {} {:?}", cmd, args);
                started = true;
                break;
            }
        }

        if !started {
            error!("Failed to start Docker Desktop on Linux after trying all methods");
            return Err(eyre!(
                "Failed to start Docker Desktop. Please ensure Docker Desktop is installed and try starting it manually."
            ));
        }
    }

    #[cfg(target_os = "windows")]
    {
        let mut started = false;

        // Try common installation paths first (more reliable than PATH check)
        let common_paths = [
            "C:\\Program Files\\Docker\\Docker\\Docker Desktop.exe",
            "C:\\Program Files (x86)\\Docker\\Docker\\Docker Desktop.exe",
            "%LOCALAPPDATA%\\Docker\\Docker Desktop.exe",
        ];

        for path in common_paths {
            let expanded_path = if path.contains('%') {
                // Expand environment variables
                std::env::var("LOCALAPPDATA")
                    .ok()
                    .map(|appdata| path.replace("%LOCALAPPDATA%", &appdata))
                    .unwrap_or_else(|| path.to_string())
            } else {
                path.to_string()
            };

            if Path::new(&expanded_path).exists() {
                // Start Docker Desktop silently without opening a command window
                if Command::new(&expanded_path).spawn().is_ok() {
                    started = true;
                    info!("Started Docker Desktop from: {}", expanded_path);
                    break;
                }
            }
        }

        // Only try PATH as a last resort with a check for existence
        if !started {
            // Use 'where' command to check if Docker Desktop.exe is in PATH
            if let Ok(output) = Command::new("where").arg("Docker Desktop.exe").output() {
                if output.status.success() {
                    // Start directly without cmd window
                    if Command::new("Docker Desktop.exe").spawn().is_ok() {
                        started = true;
                        info!("Started Docker Desktop using PATH");
                    }
                }
            }
        }

        if !started {
            error!("Failed to start Docker Desktop on Windows");
            return Err(eyre!(
                "Failed to start Docker Desktop. Please ensure Docker Desktop is installed and try starting it manually."
            ));
        }
    }

    Ok(())
}
