mod test_harness;

use axum::{
    Router,
    extract::{Path, Query},
    http::StatusCode,
    response::Json,
    routing::{get, post},
};
use eyre::Result;
use kittynode_core::api::types::{
    Config, LogsQuery, OperationalState, Package, PackageConfig, PackageRuntimeState, SystemInfo,
};
use kittynode_core::api::{DEFAULT_WEB_PORT, DockerStartStatus};
use serde::Deserialize;
use std::collections::HashMap;
use std::net::{Ipv4Addr, SocketAddr};
use test_harness::{self as harness, api, validate_web_port};
use tokio::net::TcpListener;

#[doc(hidden)]
pub use test_harness::{Harness, HarnessGuard, ResponseQueue, read_state, with_harness};

pub async fn hello_world() -> &'static str {
    "Hello World!"
}

pub async fn add_capability(Path(name): Path<String>) -> Result<StatusCode, (StatusCode, String)> {
    api::add_capability(&name).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(StatusCode::OK)
}

pub async fn remove_capability(
    Path(name): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    api::remove_capability(&name)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(StatusCode::OK)
}

pub async fn get_capabilities() -> Result<Json<Vec<String>>, (StatusCode, String)> {
    api::get_capabilities()
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

pub async fn get_packages() -> Result<Json<HashMap<String, Package>>, (StatusCode, String)> {
    api::get_packages()
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

pub async fn get_config() -> Result<Json<Config>, (StatusCode, String)> {
    api::get_config()
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

pub async fn install_package(Path(name): Path<String>) -> Result<StatusCode, (StatusCode, String)> {
    api::install_package(&name)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(StatusCode::OK)
}

#[derive(Default, Deserialize)]
pub struct DeletePackageQuery {
    include_images: Option<bool>,
}

pub async fn delete_package(
    Path(name): Path<String>,
    Query(params): Query<DeletePackageQuery>,
) -> Result<StatusCode, (StatusCode, String)> {
    let include_images = params.include_images.unwrap_or(false);
    api::delete_package(&name, include_images)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(StatusCode::OK)
}

pub async fn stop_package(Path(name): Path<String>) -> Result<StatusCode, (StatusCode, String)> {
    api::stop_package(&name)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(StatusCode::OK)
}

pub async fn resume_package(Path(name): Path<String>) -> Result<StatusCode, (StatusCode, String)> {
    api::resume_package(&name)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(StatusCode::OK)
}

#[derive(Deserialize)]
pub struct RuntimeStatesRequest {
    names: Vec<String>,
}

pub async fn get_package_runtime_state(
    Path(name): Path<String>,
) -> Result<Json<PackageRuntimeState>, (StatusCode, String)> {
    api::get_package_runtime_state(&name)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

pub async fn get_package_runtime_states(
    Json(payload): Json<RuntimeStatesRequest>,
) -> Result<Json<HashMap<String, PackageRuntimeState>>, (StatusCode, String)> {
    api::get_packages_runtime_state(&payload.names)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

pub async fn get_installed_packages() -> Result<Json<Vec<Package>>, (StatusCode, String)> {
    api::get_installed_packages()
        .await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

pub async fn is_docker_running() -> Result<StatusCode, (StatusCode, String)> {
    match api::is_docker_running().await {
        true => Ok(StatusCode::OK),
        false => Err((
            StatusCode::SERVICE_UNAVAILABLE,
            "Docker is not running".to_string(),
        )),
    }
}

pub async fn init_kittynode() -> Result<StatusCode, (StatusCode, String)> {
    api::init_kittynode().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(StatusCode::OK)
}

pub async fn delete_kittynode() -> Result<StatusCode, (StatusCode, String)> {
    api::delete_kittynode().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(StatusCode::OK)
}

pub async fn get_system_info() -> Result<Json<SystemInfo>, (StatusCode, String)> {
    api::get_system_info()
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

pub async fn get_container_logs(
    Path(container_name): Path<String>,
    Query(params): Query<LogsQuery>,
) -> Result<Json<Vec<String>>, (StatusCode, String)> {
    api::get_container_logs(&container_name, params.tail)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

pub async fn get_package_config(
    Path(name): Path<String>,
) -> Result<Json<PackageConfig>, (StatusCode, String)> {
    api::get_package_config(&name)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

pub async fn update_package_config(
    Path(name): Path<String>,
    Json(config): Json<PackageConfig>,
) -> Result<StatusCode, (StatusCode, String)> {
    api::update_package_config(&name, config)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(StatusCode::OK)
}

pub async fn start_docker_if_needed() -> Result<Json<DockerStartStatus>, (StatusCode, String)> {
    api::start_docker_if_needed()
        .await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

pub async fn get_operational_state() -> Result<Json<OperationalState>, (StatusCode, String)> {
    api::get_operational_state()
        .await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

pub fn app() -> Router {
    Router::new()
        .route("/", get(hello_world))
        .route("/add_capability/{name}", post(add_capability))
        .route("/remove_capability/{name}", post(remove_capability))
        .route("/get_capabilities", get(get_capabilities))
        .route("/get_packages", get(get_packages))
        .route("/get_config", get(get_config))
        .route("/install_package/{name}", post(install_package))
        .route("/delete_package/{name}", post(delete_package))
        .route("/stop_package/{name}", post(stop_package))
        .route("/resume_package/{name}", post(resume_package))
        .route("/get_installed_packages", get(get_installed_packages))
        .route("/package_runtime", post(get_package_runtime_states))
        .route("/package_runtime/{name}", get(get_package_runtime_state))
        .route("/is_docker_running", get(is_docker_running))
        .route("/init_kittynode", post(init_kittynode))
        .route("/delete_kittynode", post(delete_kittynode))
        .route("/get_system_info", get(get_system_info))
        .route("/logs/{container_name}", get(get_container_logs))
        .route("/get_package_config/{name}", get(get_package_config))
        .route("/update_package_config/{name}", post(update_package_config))
        .route("/start_docker_if_needed", post(start_docker_if_needed))
        .route("/get_operational_state", get(get_operational_state))
}

pub async fn run() -> Result<()> {
    let _ = tracing_subscriber::fmt::try_init();
    run_with_port(DEFAULT_WEB_PORT).await
}

pub async fn run_with_port(port: u16) -> Result<()> {
    validate_web_port(port)?;
    if harness::should_skip_server_start() {
        return Ok(());
    }
    let app = app();
    let address = SocketAddr::from((Ipv4Addr::UNSPECIFIED, port));
    let listener = TcpListener::bind(address).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
