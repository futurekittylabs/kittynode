mod core_client;

use crate::core_client::{CoreClientManager, HttpCoreClient};
use eyre::Result;
use kittynode_core::api;
use kittynode_core::api::DockerStartStatus;
use kittynode_core::api::types::{
    Config, OperationalState, Package, PackageConfig, PackageState, SystemInfo,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::LazyLock;
// RunEvent::Reopen only exists on macOS (handles dock icon click when no windows visible)
#[cfg(target_os = "macos")]
use tauri::RunEvent;
use tauri::{
    Manager, State, WindowEvent,
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
};
use tauri_plugin_http::reqwest;
use tracing::{debug, info};

#[derive(Serialize, Deserialize)]
struct LatestManifest {
    version: String,
}

pub static HTTP_CLIENT: LazyLock<reqwest::Client> = LazyLock::new(reqwest::Client::new);

#[tauri::command]
async fn fetch_latest_manifest(url: String) -> Result<LatestManifest, String> {
    info!("Fetching latest manifest from: {}", url);

    let response = HTTP_CLIENT
        .get(&url)
        .header(reqwest::header::CACHE_CONTROL, "no-cache")
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
async fn get_package_catalog(
    client_state: State<'_, CoreClientManager>,
) -> Result<HashMap<String, Package>, String> {
    let client = client_state.client();
    client
        .get_package_catalog()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_capabilities(
    client_state: State<'_, CoreClientManager>,
) -> Result<Vec<String>, String> {
    let client = client_state.client();
    client.get_capabilities().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn add_capability(
    name: String,
    client_state: State<'_, CoreClientManager>,
) -> Result<(), String> {
    info!("Adding capability: {}", name);
    let client = client_state.client();
    client
        .add_capability(&name)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn remove_capability(
    name: String,
    client_state: State<'_, CoreClientManager>,
) -> Result<(), String> {
    info!("Removing capability: {}", name);
    let client = client_state.client();
    client
        .remove_capability(&name)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_installed_packages(
    client_state: State<'_, CoreClientManager>,
) -> Result<Vec<Package>, String> {
    info!("Getting installed packages");
    let client = client_state.client();
    client
        .get_installed_packages()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn is_docker_running(client_state: State<'_, CoreClientManager>) -> Result<bool, String> {
    info!("Checking if Docker is running");
    let client = client_state.client();
    client.is_docker_running().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn start_docker_if_needed(
    client_state: State<'_, CoreClientManager>,
) -> Result<String, String> {
    info!("Checking if Docker needs to be started");
    let client = client_state.client();
    client
        .start_docker_if_needed()
        .await
        .map(DockerStartStatus::as_str)
        .map(str::to_string)
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn install_package(
    name: String,
    network: Option<String>,
    client_state: State<'_, CoreClientManager>,
) -> Result<(), String> {
    if let Some(ref network) = network {
        info!("Installing package: {} on {}", name, network);
    } else {
        info!("Installing package: {}", name);
    }
    let client = client_state.client();
    client
        .install_package(&name, network.as_deref())
        .await
        .map(|_| info!("Successfully installed package: {}", name))
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn delete_package(
    name: String,
    include_images: bool,
    client_state: State<'_, CoreClientManager>,
) -> Result<(), String> {
    info!(
        "Deleting package: {} (include_images: {})",
        name, include_images
    );
    let client = client_state.client();
    client
        .delete_package(&name, include_images)
        .await
        .map(|_| info!("Successfully deleted package: {}", name))
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn stop_package(
    name: String,
    client_state: State<'_, CoreClientManager>,
) -> Result<(), String> {
    info!("Stopping package: {}", name);
    let client = client_state.client();
    client
        .stop_package(&name)
        .await
        .map(|_| info!("Successfully stopped package: {}", name))
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn start_package(
    name: String,
    client_state: State<'_, CoreClientManager>,
) -> Result<(), String> {
    info!("Starting package: {}", name);
    let client = client_state.client();
    client
        .start_package(&name)
        .await
        .map(|_| info!("Successfully started package: {}", name))
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_package(
    name: String,
    client_state: State<'_, CoreClientManager>,
) -> Result<PackageState, String> {
    debug!("Fetching package state: {}", name);
    let client = client_state.client();
    client.get_package(&name).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_packages(
    names: Vec<String>,
    client_state: State<'_, CoreClientManager>,
) -> Result<HashMap<String, PackageState>, String> {
    debug!("Fetching package states: {:?}", names);
    let client = client_state.client();
    let name_refs: Vec<&str> = names.iter().map(|name| name.as_str()).collect();
    client
        .get_packages(&name_refs)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn delete_kittynode(client_state: State<'_, CoreClientManager>) -> Result<(), String> {
    info!("Deleting ~/.config/kittynode directory");
    let client = client_state.client();
    client.delete_kittynode().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn system_info(client_state: State<'_, CoreClientManager>) -> Result<SystemInfo, String> {
    info!("Getting system info");
    let client = client_state.client();
    client.system_info().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn init_kittynode(client_state: State<'_, CoreClientManager>) -> Result<(), String> {
    info!("Initializing Kittynode");
    let client = client_state.client();
    client.init_kittynode().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_container_logs(
    container_name: String,
    tail_lines: Option<usize>,
    client_state: State<'_, CoreClientManager>,
) -> Result<Vec<String>, String> {
    debug!(
        "Getting logs for container: {} (tail: {:?})",
        container_name, tail_lines
    );
    let client = client_state.client();
    client
        .get_container_logs(&container_name, tail_lines)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn is_validator_installed(
    client_state: State<'_, CoreClientManager>,
) -> Result<bool, String> {
    let client = client_state.client();
    client
        .is_validator_installed()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_package_config(
    name: String,
    client_state: State<'_, CoreClientManager>,
) -> Result<PackageConfig, String> {
    let client = client_state.client();
    client
        .get_package_config(&name)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn update_package_config(
    name: String,
    config: PackageConfig,
    client_state: State<'_, CoreClientManager>,
) -> Result<(), String> {
    let client = client_state.client();
    client
        .update_package_config(&name, config)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn check_remote_health(endpoint: String) -> Result<(), String> {
    let client = HttpCoreClient::new(&endpoint).map_err(|e| e.to_string())?;
    client.health_check().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_operational_state(
    client_state: State<'_, CoreClientManager>,
) -> Result<OperationalState, String> {
    let client = client_state.client();
    client
        .get_operational_state()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn get_onboarding_completed() -> Result<bool, String> {
    info!("Getting onboarding completed status");
    api::get_onboarding_completed().map_err(|e| e.to_string())
}

#[tauri::command]
fn set_onboarding_completed(completed: bool) -> Result<(), String> {
    info!("Setting onboarding completed to: {}", completed);
    api::set_onboarding_completed(completed).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_config() -> Result<Config, String> {
    info!("Loading Kittynode configuration");
    api::get_config().map_err(|e| e.to_string())
}

#[tauri::command]
fn set_auto_start_docker(enabled: bool) -> Result<(), String> {
    info!("Updating auto start docker preference to: {}", enabled);
    api::set_auto_start_docker(enabled).map_err(|e| e.to_string())
}

#[tauri::command]
fn set_show_tray_icon(app_handle: tauri::AppHandle, enabled: bool) -> Result<(), String> {
    info!("Updating show tray icon preference to: {}", enabled);
    api::set_show_tray_icon(enabled).map_err(|e| e.to_string())?;
    if let Some(tray) = app_handle.tray_by_id("main-tray") {
        tray.set_visible(enabled).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
fn set_server_url(
    client_state: State<'_, CoreClientManager>,
    endpoint: String,
) -> Result<(), String> {
    info!("Updating server URL to: {}", endpoint);
    api::set_server_url(endpoint).map_err(|e| e.to_string())?;
    let config = api::get_config().map_err(|e| e.to_string())?;
    client_state.reload(&config).map_err(|e| e.to_string())
}

#[tauri::command]
async fn restart_app(app_handle: tauri::AppHandle) {
    info!("Restarting application");
    app_handle.restart();
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() -> Result<()> {
    let config = api::get_config()?;
    let core_client = CoreClientManager::new(&config)?;

    let builder = tauri::Builder::default()
        .manage(core_client)
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_shell::init());

    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    let builder = {
        let updater_plugin = tauri_plugin_updater::Builder::new()
            .header("Cache-Control", "no-cache")
            .map_err(|e| eyre::eyre!("failed to configure updater header: {e}"))?
            .build();
        builder.plugin(updater_plugin)
    };

    builder
        .setup(|app| {
            if let Some(window) = app.get_webview_window("main") {
                window.set_focus().ok();
            }

            // Create system tray menu
            let show_item = MenuItem::with_id(app, "show", "Show Kittynode", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_item, &quit_item])?;

            // Get config to check tray icon visibility preference
            let show_tray = match api::get_config() {
                Ok(c) => c.show_tray_icon,
                Err(e) => {
                    tracing::warn!("Failed to load config for tray visibility: {e}");
                    true
                }
            };

            // Build tray icon with menu on left-click
            let tray = TrayIconBuilder::with_id("main-tray")
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .show_menu_on_left_click(true)
                .tooltip("Kittynode")
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.unminimize();
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .build(app)?;

            // Apply initial visibility from config
            tray.set_visible(show_tray).ok();

            Ok(())
        })
        .on_window_event(|window, event| {
            // Hide window instead of closing so app stays in tray
            if let WindowEvent::CloseRequested { api, .. } = event {
                let _ = window.hide();
                api.prevent_close();
            }
        })
        .invoke_handler(tauri::generate_handler![
            fetch_latest_manifest,
            get_package_catalog,
            get_installed_packages,
            is_docker_running,
            start_docker_if_needed,
            install_package,
            delete_package,
            stop_package,
            start_package,
            get_package,
            get_packages,
            delete_kittynode,
            system_info,
            init_kittynode,
            add_capability,
            remove_capability,
            get_capabilities,
            get_container_logs,
            is_validator_installed,
            get_package_config,
            update_package_config,
            get_operational_state,
            get_onboarding_completed,
            set_onboarding_completed,
            get_config,
            set_auto_start_docker,
            set_show_tray_icon,
            set_server_url,
            check_remote_health,
            restart_app
        ])
        .build(tauri::generate_context!())
        .map_err(|e| eyre::eyre!(e.to_string()))?
        .run(|app_handle, event| {
            // Reopen: dock icon clicked when no windows visible (macOS-only, no equivalent on Linux/Windows)
            #[cfg(target_os = "macos")]
            if let RunEvent::Reopen { .. } = event
                && let Some(window) = app_handle.get_webview_window("main")
            {
                let _ = window.show();
                let _ = window.set_focus();
            }

            let _ = (&app_handle, &event);
        });

    Ok(())
}
