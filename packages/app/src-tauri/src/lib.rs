mod core_client;

use core_client::{CoreClientState, HTTP_CLIENT};
use eyre::Result;
use kittynode_core::api::types::{Config, Package, PackageConfig, SystemInfo};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, LazyLock, Mutex};
use tauri::Manager;
use tracing::info;

#[derive(Serialize, Deserialize)]
struct LatestManifest {
    version: String,
}
// Tracks whether we've already attempted to start Docker automatically this session
pub static DOCKER_AUTO_STARTED: LazyLock<Arc<Mutex<bool>>> =
    LazyLock::new(|| Arc::new(Mutex::new(false)));

#[tauri::command]
async fn fetch_latest_manifest(url: String) -> Result<LatestManifest, String> {
    info!("Fetching latest manifest from: {}", url);

    let response = HTTP_CLIENT
        .get(&url)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let status = response.status();
    if !status.is_success() {
        return Err(format!("Failed to fetch latest manifest: {}", status));
    }

    response
        .json::<LatestManifest>()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn add_capability(
    client_state: tauri::State<'_, CoreClientState>,
    name: String,
) -> Result<(), String> {
    info!("Adding capability: {}", name);
    let client = client_state.client();
    client.add_capability(&name).await
}

#[tauri::command]
async fn remove_capability(
    client_state: tauri::State<'_, CoreClientState>,
    name: String,
) -> Result<(), String> {
    info!("Removing capability: {}", name);
    let client = client_state.client();
    client.remove_capability(&name).await
}

#[tauri::command]
async fn get_capabilities(
    client_state: tauri::State<'_, CoreClientState>,
) -> Result<Vec<String>, String> {
    info!("Getting capabilities");
    let client = client_state.client();
    client.get_capabilities().await
}

#[tauri::command]
async fn get_packages(
    client_state: tauri::State<'_, CoreClientState>,
) -> Result<HashMap<String, Package>, String> {
    info!("Getting packages");
    let client = client_state.client();
    client.get_packages().await
}

#[tauri::command]
async fn get_installed_packages(
    client_state: tauri::State<'_, CoreClientState>,
) -> Result<Vec<Package>, String> {
    info!("Getting installed packages");
    let client = client_state.client();
    client.get_installed_packages().await
}

#[tauri::command]
async fn is_docker_running(
    client_state: tauri::State<'_, CoreClientState>,
) -> Result<bool, String> {
    info!("Checking if Docker is running");
    let client = client_state.client();
    client.is_docker_running().await
}

#[tauri::command]
async fn start_docker_if_needed(
    client_state: tauri::State<'_, CoreClientState>,
) -> Result<String, String> {
    info!("Checking if Docker needs to be started");

    let client = client_state.client();
    let config = kittynode_core::api::get_config().map_err(|e| e.to_string())?;

    if !config.server_url.trim().is_empty() {
        return if client.is_docker_running().await? {
            Ok("running".to_string())
        } else {
            Ok("unavailable".to_string())
        };
    }

    let already_attempted = {
        let auto_started = DOCKER_AUTO_STARTED.lock().unwrap();
        *auto_started
    };

    if already_attempted {
        if client.is_docker_running().await? {
            return Ok("running".to_string());
        }

        return Ok("already_started".to_string());
    }

    if client.is_docker_running().await? {
        let mut auto_started = DOCKER_AUTO_STARTED.lock().unwrap();
        *auto_started = true;
        return Ok("running".to_string());
    }

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
        if client.is_docker_running().await? {
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
async fn install_package(
    client_state: tauri::State<'_, CoreClientState>,
    name: String,
) -> Result<(), String> {
    let client = client_state.client();
    client.install_package(&name).await?;
    info!("Successfully installed package: {}", name);
    Ok(())
}

#[tauri::command]
async fn delete_package(
    client_state: tauri::State<'_, CoreClientState>,
    name: String,
    include_images: bool,
) -> Result<(), String> {
    let client = client_state.client();
    client.delete_package(&name, include_images).await?;
    info!("Successfully deleted package: {}", name);
    Ok(())
}

#[tauri::command]
async fn delete_kittynode(client_state: tauri::State<'_, CoreClientState>) -> Result<(), String> {
    info!("Deleting .kittynode directory");
    let client = client_state.client();
    client.delete_kittynode().await
}

#[tauri::command]
async fn system_info(
    client_state: tauri::State<'_, CoreClientState>,
) -> Result<SystemInfo, String> {
    info!("Getting system info");
    let client = client_state.client();
    client.get_system_info().await
}

#[tauri::command]
async fn init_kittynode(client_state: tauri::State<'_, CoreClientState>) -> Result<(), String> {
    info!("Initializing Kittynode");
    let client = client_state.client();
    client.init_kittynode().await
}

#[tauri::command]
async fn get_container_logs(
    client_state: tauri::State<'_, CoreClientState>,
    container_name: String,
    tail_lines: Option<usize>,
) -> Result<Vec<String>, String> {
    info!(
        "Getting logs for container: {} (tail: {:?})",
        container_name, tail_lines
    );
    let client = client_state.client();
    client.get_container_logs(&container_name, tail_lines).await
}

#[tauri::command]
async fn get_package_config(
    client_state: tauri::State<'_, CoreClientState>,
    name: String,
) -> Result<PackageConfig, String> {
    let client = client_state.client();
    client.get_package_config(&name).await
}

#[tauri::command]
async fn update_package_config(
    client_state: tauri::State<'_, CoreClientState>,
    name: String,
    config: PackageConfig,
) -> Result<(), String> {
    let client = client_state.client();
    client.update_package_config(&name, config).await
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
fn set_server_url(
    client_state: tauri::State<'_, CoreClientState>,
    server_url: String,
) -> Result<(), String> {
    info!("Setting server URL to: {}", server_url);
    client_state.set_server_url(server_url)
}

#[tauri::command]
async fn restart_app(app_handle: tauri::AppHandle) {
    info!("Restarting application");
    app_handle.restart();
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() -> Result<()> {
    let core_client_state = CoreClientState::initialize();

    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_shell::init());

    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    let builder = builder.plugin(tauri_plugin_updater::Builder::new().build());

    let builder = builder.manage(core_client_state);

    builder
        .setup(|app| {
            // Ensure window is focused to show on restart
            if let Some(window) = app.get_webview_window("main") {
                window.set_focus().ok();
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            fetch_latest_manifest,
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
            set_server_url,
            restart_app
        ])
        .run(tauri::generate_context!())
        .map_err(|e| eyre::eyre!(e.to_string()))?;

    Ok(())
}
