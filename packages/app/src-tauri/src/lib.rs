mod core_client;

use crate::core_client::CoreClientManager;
use eyre::Result;
use kittynode_core::api;
use kittynode_core::api::DockerStartStatus;
use kittynode_core::api::types::{
    Config, DepositData, OperationalState, Package, PackageConfig, PackageRuntimeState, SystemInfo,
    ValidatorKey,
};
use kittynode_core::api::{CreateDepositDataParams, GenerateKeysParams};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::LazyLock;
use tauri::{Manager, State};
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
async fn get_packages(
    client_state: State<'_, CoreClientManager>,
) -> Result<HashMap<String, Package>, String> {
    let client = client_state.client();
    client.get_packages().await.map_err(|e| e.to_string())
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
    client_state: State<'_, CoreClientManager>,
) -> Result<(), String> {
    info!("Installing package: {}", name);
    let client = client_state.client();
    client
        .install_package(&name)
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
async fn resume_package(
    name: String,
    client_state: State<'_, CoreClientManager>,
) -> Result<(), String> {
    info!("Resuming package: {}", name);
    let client = client_state.client();
    client
        .resume_package(&name)
        .await
        .map(|_| info!("Successfully resumed package: {}", name))
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_package_runtime_state(
    name: String,
    client_state: State<'_, CoreClientManager>,
) -> Result<PackageRuntimeState, String> {
    debug!("Fetching runtime state for package: {}", name);
    let client = client_state.client();
    client
        .get_package_runtime_state(&name)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_package_runtime_states(
    names: Vec<String>,
    client_state: State<'_, CoreClientManager>,
) -> Result<HashMap<String, PackageRuntimeState>, String> {
    debug!("Fetching runtime state for packages: {:?}", names);
    let client = client_state.client();
    client
        .get_package_runtime_states(&names)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn delete_kittynode(client_state: State<'_, CoreClientManager>) -> Result<(), String> {
    info!("Deleting .kittynode directory");
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
async fn get_operational_state(
    client_state: State<'_, CoreClientManager>,
) -> Result<OperationalState, String> {
    let client = client_state.client();
    client
        .get_operational_state()
        .await
        .map_err(|e| e.to_string())
}

#[derive(Deserialize)]
struct GenerateValidatorKeysArgs {
    output_dir: String,
    file_name: Option<String>,
    entropy: String,
    overwrite: bool,
}

#[tauri::command]
async fn generate_validator_keys(
    args: GenerateValidatorKeysArgs,
    client_state: State<'_, CoreClientManager>,
) -> Result<ValidatorKey, String> {
    let client = client_state.client();
    let params = GenerateKeysParams {
        output_dir: PathBuf::from(&args.output_dir),
        file_name: args.file_name,
        entropy: args.entropy,
        overwrite: args.overwrite,
    };
    client
        .generate_validator_keys(params)
        .await
        .map_err(|e| e.to_string())
}

#[derive(Deserialize)]
struct CreateDepositDataArgs {
    key_path: String,
    output_path: String,
    withdrawal_address: String,
    amount_gwei: u64,
    overwrite: bool,
    network: String,
}

#[tauri::command]
async fn create_validator_deposit_data(
    args: CreateDepositDataArgs,
    client_state: State<'_, CoreClientManager>,
) -> Result<DepositData, String> {
    let CreateDepositDataArgs {
        key_path,
        output_path,
        withdrawal_address,
        amount_gwei,
        overwrite,
        network,
    } = args;

    let params = CreateDepositDataParams::for_network(
        PathBuf::from(&key_path),
        PathBuf::from(&output_path),
        &withdrawal_address,
        amount_gwei,
        &network,
        overwrite,
    )
    .map_err(|err| err.to_string())?;
    let client = client_state.client();
    client
        .create_validator_deposit_data(params)
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
    let builder = builder.plugin(tauri_plugin_updater::Builder::new().build());

    builder
        .setup(|app| {
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
            stop_package,
            resume_package,
            get_package_runtime_state,
            get_package_runtime_states,
            delete_kittynode,
            system_info,
            init_kittynode,
            add_capability,
            remove_capability,
            get_capabilities,
            get_container_logs,
            get_package_config,
            update_package_config,
            get_operational_state,
            generate_validator_keys,
            create_validator_deposit_data,
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
