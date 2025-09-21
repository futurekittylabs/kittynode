use eyre::Result;
use kittynode_core::api::types::{Config, Package, PackageConfig, SystemInfo};
use std::collections::HashMap;
use std::sync::{Arc, LazyLock, Mutex};
use tauri::Manager;
use tauri_plugin_http::reqwest;
use tracing::info;

pub static HTTP_CLIENT: LazyLock<reqwest::Client> = LazyLock::new(reqwest::Client::new);
// Tracks whether we've already attempted to start Docker automatically this session
pub static DOCKER_AUTO_STARTED: LazyLock<Arc<Mutex<bool>>> =
    LazyLock::new(|| Arc::new(Mutex::new(false)));

#[tauri::command]
async fn add_capability(name: String, server_url: String) -> Result<(), String> {
    info!("Adding capability: {}", name);

    if !server_url.is_empty() {
        let url = format!("{}/add_capability/{}", server_url, name);
        let res = HTTP_CLIENT
            .post(&url)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        if !res.status().is_success() {
            return Err(format!("Failed to add capability: {}", res.status()));
        }
        Ok(())
    } else {
        kittynode_core::api::add_capability(&name).map_err(|e| e.to_string())
    }
}

#[tauri::command]
async fn remove_capability(name: String, server_url: String) -> Result<(), String> {
    info!("Removing capability: {}", name);

    if !server_url.is_empty() {
        let url = format!("{}/remove_capability/{}", server_url, name);
        let res = HTTP_CLIENT
            .post(&url)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        if !res.status().is_success() {
            return Err(format!("Failed to remove capability: {}", res.status()));
        }
        Ok(())
    } else {
        kittynode_core::api::remove_capability(&name).map_err(|e| e.to_string())
    }
}

#[tauri::command]
async fn get_capabilities(server_url: String) -> Result<Vec<String>, String> {
    info!("Getting capabilities");

    if !server_url.is_empty() {
        let url = format!("{}/get_capabilities", server_url);
        let res = HTTP_CLIENT
            .get(&url)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !res.status().is_success() {
            let status = res.status();
            let error_text = res.text().await.unwrap_or_default();
            return Err(format!(
                "Failed to get capabilities: {} - {}",
                status, error_text
            ));
        }

        res.json::<Vec<String>>().await.map_err(|e| e.to_string())
    } else {
        kittynode_core::api::get_capabilities().map_err(|e| e.to_string())
    }
}

#[tauri::command]
fn get_packages() -> Result<HashMap<String, Package>, String> {
    info!("Getting packages");
    kittynode_core::api::get_packages()
        .map(|packages| {
            packages
                .into_iter()
                .map(|(name, package)| (name.to_string(), package))
                .collect()
        })
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_installed_packages(server_url: String) -> Result<Vec<Package>, String> {
    info!("Getting installed packages");

    if !server_url.is_empty() {
        let url = format!("{}/get_installed_packages", server_url);
        let res = HTTP_CLIENT
            .get(&url)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        let status = res.status();
        if !status.is_success() {
            let error_text = res.text().await.unwrap_or_default();
            return Err(format!(
                "Failed to get installed packages: {} - {}",
                status, error_text
            ));
        }

        res.json::<Vec<Package>>().await.map_err(|e| e.to_string())
    } else {
        kittynode_core::api::get_installed_packages()
            .await
            .map_err(|e| e.to_string())
    }
}

#[tauri::command]
async fn is_docker_running() -> bool {
    info!("Checking if Docker is running");
    kittynode_core::api::is_docker_running().await
}

#[tauri::command]
async fn start_docker_if_needed() -> Result<String, String> {
    info!("Checking if Docker needs to be started");

    let already_attempted = {
        let auto_started = DOCKER_AUTO_STARTED.lock().unwrap();
        *auto_started
    };

    if already_attempted {
        if kittynode_core::api::is_docker_running().await {
            return Ok("running".to_string());
        }

        return Ok("already_started".to_string());
    }

    if kittynode_core::api::is_docker_running().await {
        let mut auto_started = DOCKER_AUTO_STARTED.lock().unwrap();
        *auto_started = true;
        return Ok("running".to_string());
    }

    let config = kittynode_core::api::get_config().map_err(|e| e.to_string())?;
    if !config.auto_start_docker {
        info!("Skipping Docker auto-start due to user preference");
        return Ok("disabled".to_string());
    }

    let should_attempt_start = {
        let mut auto_started = DOCKER_AUTO_STARTED.lock().unwrap();
        if *auto_started {
            false
        } else {
            *auto_started = true;
            true
        }
    };

    if !should_attempt_start {
        if kittynode_core::api::is_docker_running().await {
            return Ok("running".to_string());
        }

        return Ok("already_started".to_string());
    }

    info!("Starting Docker Desktop");

    if let Err(err) = kittynode_core::api::start_docker().await {
        let mut auto_started = DOCKER_AUTO_STARTED.lock().unwrap();
        *auto_started = false;
        return Err(err.to_string());
    }

    Ok("starting".to_string())
}

#[tauri::command]
async fn install_package(name: String, server_url: String) -> Result<(), String> {
    if !server_url.is_empty() {
        let url = format!("{}/install_package/{}", server_url, name);
        let res = HTTP_CLIENT
            .post(&url)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        if !res.status().is_success() {
            return Err(format!("Failed to install package: {}", res.status()));
        }
    } else {
        kittynode_core::api::install_package(&name)
            .await
            .map_err(|e| e.to_string())?;
    }

    info!("Successfully installed package: {}", name);
    Ok(())
}

#[tauri::command]
async fn delete_package(
    name: String,
    include_images: bool,
    server_url: String,
) -> Result<(), String> {
    if !server_url.is_empty() {
        let url = format!("{}/delete_package/{}", server_url, name);
        let res = HTTP_CLIENT
            .post(&url)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        if !res.status().is_success() {
            return Err(format!("Failed to delete package: {}", res.status()));
        }
    } else {
        kittynode_core::api::delete_package(&name, include_images)
            .await
            .map_err(|e| e.to_string())?;
    }

    info!("Successfully deleted package: {}", name);
    Ok(())
}

#[tauri::command]
async fn delete_kittynode(server_url: String) -> Result<(), String> {
    info!("Deleting .kittynode directory");

    if !server_url.is_empty() {
        let url = format!("{}/delete_kittynode", server_url);
        let res = HTTP_CLIENT
            .post(&url)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        if !res.status().is_success() {
            return Err(format!("Failed to delete Kittynode: {}", res.status()));
        }
        Ok(())
    } else {
        kittynode_core::api::delete_kittynode().map_err(|e| e.to_string())
    }
}

#[tauri::command]
async fn system_info(server_url: String) -> Result<SystemInfo, String> {
    info!("Getting system info");

    if !server_url.is_empty() {
        let url = format!("{}/get_system_info", server_url);
        let res = HTTP_CLIENT
            .get(&url)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !res.status().is_success() {
            return Err(format!("Failed to get system info: {}", res.status()));
        }

        res.json::<SystemInfo>().await.map_err(|e| e.to_string())
    } else {
        kittynode_core::api::get_system_info().map_err(|e| e.to_string())
    }
}

#[tauri::command]
async fn init_kittynode(server_url: String) -> Result<(), String> {
    info!("Initializing Kittynode");

    if !server_url.is_empty() {
        let url = format!("{}/init_kittynode", server_url);
        let res = HTTP_CLIENT
            .post(&url)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        if !res.status().is_success() {
            return Err(format!("Failed to initialize Kittynode: {}", res.status()));
        }
        Ok(())
    } else {
        kittynode_core::api::init_kittynode().map_err(|e| e.to_string())
    }
}

#[tauri::command]
async fn get_container_logs(
    container_name: String,
    tail_lines: Option<usize>,
    server_url: String,
) -> Result<Vec<String>, String> {
    info!(
        "Getting logs for container: {} (tail: {:?})",
        container_name, tail_lines
    );

    if !server_url.is_empty() {
        let url = format!("{}/logs/{}", server_url, container_name);
        // Add tail_lines to query params if present
        let url = if let Some(n) = tail_lines {
            format!("{}?tail={}", url, n)
        } else {
            url
        };

        let res = HTTP_CLIENT
            .get(&url)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        if !res.status().is_success() {
            return Err(format!("Failed to get logs: {}", res.status()));
        }
        res.json::<Vec<String>>().await.map_err(|e| e.to_string())
    } else {
        kittynode_core::api::get_container_logs(&container_name, tail_lines)
            .await
            .map_err(|e| e.to_string())
    }
}

#[tauri::command]
async fn get_package_config(name: String, server_url: String) -> Result<PackageConfig, String> {
    if !server_url.is_empty() {
        let url = format!("{}/get_package_config/{}", server_url, name);
        let res = HTTP_CLIENT
            .get(&url)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        res.json::<PackageConfig>().await.map_err(|e| e.to_string())
    } else {
        kittynode_core::api::get_package_config(&name)
            .await
            .map_err(|e| e.to_string())
    }
}

#[tauri::command]
async fn update_package_config(
    name: String,
    config: PackageConfig,
    server_url: String,
) -> Result<(), String> {
    if !server_url.is_empty() {
        let url = format!("{}/update_package_config/{}", server_url, name);
        let res = HTTP_CLIENT
            .post(&url)
            .json(&config)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        if !res.status().is_success() {
            return Err(format!("Failed to update package config: {}", res.status()));
        }
        Ok(())
    } else {
        kittynode_core::api::update_package_config(&name, config)
            .await
            .map_err(|e| e.to_string())
    }
}

#[tauri::command]
fn get_onboarding_completed() -> Result<bool, String> {
    info!("Getting onboarding completed status");
    kittynode_core::api::get_onboarding_completed().map_err(|e| e.to_string())
}

#[tauri::command]
fn set_onboarding_completed(completed: bool) -> Result<(), String> {
    info!("Setting onboarding completed to: {}", completed);
    kittynode_core::api::set_onboarding_completed(completed).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_config() -> Result<Config, String> {
    info!("Loading Kittynode configuration");
    kittynode_core::api::get_config().map_err(|e| e.to_string())
}

#[tauri::command]
fn set_auto_start_docker(enabled: bool) -> Result<(), String> {
    info!("Updating auto start docker preference to: {}", enabled);
    kittynode_core::api::set_auto_start_docker(enabled).map_err(|e| e.to_string())
}

#[tauri::command]
async fn restart_app(app_handle: tauri::AppHandle) {
    info!("Restarting application");
    app_handle.restart();
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() -> Result<()> {
    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_shell::init());

    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    let builder = builder.plugin(tauri_plugin_updater::Builder::new().build());

    builder
        .setup(|app| {
            // Ensure window is focused to show on restart
            if let Some(window) = app.get_webview_window("main") {
                window.set_focus().ok();
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_packages,
            get_installed_packages,
            is_docker_running,
            start_docker_if_needed,
            install_package,
            delete_package,
            delete_kittynode,
            system_info,
            init_kittynode,
            add_capability,
            remove_capability,
            get_capabilities,
            get_container_logs,
            get_package_config,
            update_package_config,
            get_onboarding_completed,
            set_onboarding_completed,
            get_config,
            set_auto_start_docker,
            restart_app
        ])
        .run(tauri::generate_context!())
        .map_err(|e| eyre::eyre!(e.to_string()))?;

    Ok(())
}
