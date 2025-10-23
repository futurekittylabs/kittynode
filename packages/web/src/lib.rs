use axum::{
    Router,
    extract::{Path, Query},
    http::StatusCode,
    response::Json,
    routing::{get, post},
};
use eyre::Result;
use kittynode_core::api;
use kittynode_core::api::types::{
    Config, LogsQuery, OperationalState, Package, PackageConfig, PackageState, SystemInfo,
};
use kittynode_core::api::{DEFAULT_WEB_PORT, DockerStartStatus, validate_web_port};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::{Ipv4Addr, SocketAddr};
use tokio::net::TcpListener;

pub async fn hello_world() -> &'static str {
    "Hello World!"
}

fn to_http_error<E: std::fmt::Display>(err: E) -> (StatusCode, String) {
    let msg = err.to_string();
    if msg.to_ascii_lowercase().contains("not found") {
        (StatusCode::NOT_FOUND, msg)
    } else {
        (StatusCode::INTERNAL_SERVER_ERROR, msg)
    }
}

pub async fn add_capability(Path(name): Path<String>) -> Result<StatusCode, (StatusCode, String)> {
    api::add_capability(&name).map_err(to_http_error)?;
    Ok(StatusCode::OK)
}

pub async fn remove_capability(
    Path(name): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    api::remove_capability(&name).map_err(to_http_error)?;
    Ok(StatusCode::OK)
}

pub async fn get_capabilities() -> Result<Json<Vec<String>>, (StatusCode, String)> {
    api::get_capabilities().map(Json).map_err(to_http_error)
}

pub async fn get_package_catalog() -> Result<Json<HashMap<String, Package>>, (StatusCode, String)> {
    api::get_package_catalog().map(Json).map_err(to_http_error)
}

pub async fn get_config() -> Result<Json<Config>, (StatusCode, String)> {
    api::get_config().map(Json).map_err(to_http_error)
}

#[derive(Default, Deserialize)]
pub struct InstallPackageQuery {
    network: Option<String>,
}

pub async fn install_package(
    Path(name): Path<String>,
    Query(params): Query<InstallPackageQuery>,
) -> Result<StatusCode, (StatusCode, String)> {
    api::install_package_with_network(&name, params.network.as_deref())
        .await
        .map_err(to_http_error)?;
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
        .map_err(to_http_error)?;
    Ok(StatusCode::OK)
}

pub async fn stop_package(Path(name): Path<String>) -> Result<StatusCode, (StatusCode, String)> {
    api::stop_package(&name).await.map_err(to_http_error)?;
    Ok(StatusCode::OK)
}

pub async fn start_package(Path(name): Path<String>) -> Result<StatusCode, (StatusCode, String)> {
    api::start_package(&name).await.map_err(to_http_error)?;
    Ok(StatusCode::OK)
}

#[derive(Deserialize)]
pub struct RuntimeStatesRequest {
    names: Vec<String>,
}

pub async fn get_package(
    Path(name): Path<String>,
) -> Result<Json<PackageState>, (StatusCode, String)> {
    api::get_package(&name)
        .await
        .map(Json)
        .map_err(to_http_error)
}

pub async fn get_packages(
    Json(payload): Json<RuntimeStatesRequest>,
) -> Result<Json<HashMap<String, PackageState>>, (StatusCode, String)> {
    let name_refs: Vec<&str> = payload.names.iter().map(|name| name.as_str()).collect();
    api::get_packages(&name_refs)
        .await
        .map(Json)
        .map_err(to_http_error)
}

pub async fn get_installed_packages() -> Result<Json<Vec<Package>>, (StatusCode, String)> {
    api::get_installed_packages()
        .await
        .map(Json)
        .map_err(to_http_error)
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
    api::init_kittynode().map_err(to_http_error)?;
    Ok(StatusCode::OK)
}

pub async fn delete_kittynode() -> Result<StatusCode, (StatusCode, String)> {
    api::delete_kittynode().map_err(to_http_error)?;
    Ok(StatusCode::OK)
}

pub async fn get_system_info() -> Result<Json<SystemInfo>, (StatusCode, String)> {
    api::get_system_info().map(Json).map_err(to_http_error)
}

pub async fn is_validator_installed() -> Result<Json<bool>, (StatusCode, String)> {
    api::is_validator_installed()
        .await
        .map(Json)
        .map_err(to_http_error)
}

pub async fn get_container_logs(
    Path(container_name): Path<String>,
    Query(params): Query<LogsQuery>,
) -> Result<Json<Vec<String>>, (StatusCode, String)> {
    api::get_container_logs(&container_name, params.tail)
        .await
        .map(Json)
        .map_err(to_http_error)
}

pub async fn get_package_config(
    Path(name): Path<String>,
) -> Result<Json<PackageConfig>, (StatusCode, String)> {
    api::get_package_config(&name)
        .await
        .map(Json)
        .map_err(to_http_error)
}

pub async fn update_package_config(
    Path(name): Path<String>,
    Json(config): Json<PackageConfig>,
) -> Result<StatusCode, (StatusCode, String)> {
    api::update_package_config(&name, config)
        .await
        .map_err(to_http_error)?;
    Ok(StatusCode::OK)
}

pub async fn start_docker_if_needed() -> Result<Json<DockerStartStatus>, (StatusCode, String)> {
    api::start_docker_if_needed()
        .await
        .map(Json)
        .map_err(to_http_error)
}

pub async fn get_operational_state() -> Result<Json<OperationalState>, (StatusCode, String)> {
    api::get_operational_state()
        .await
        .map(Json)
        .map_err(to_http_error)
}

#[derive(Serialize)]
pub struct HealthResponse {
    status: &'static str,
}

pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
}

pub fn app() -> Router {
    Router::new()
        .route("/", get(hello_world))
        .route("/health", get(health))
        .route("/add_capability/{name}", post(add_capability))
        .route("/remove_capability/{name}", post(remove_capability))
        .route("/get_capabilities", get(get_capabilities))
        .route("/get_package_catalog", get(get_package_catalog))
        .route("/get_config", get(get_config))
        .route("/install_package/{name}", post(install_package))
        .route("/delete_package/{name}", post(delete_package))
        .route("/stop_package/{name}", post(stop_package))
        .route("/start_package/{name}", post(start_package))
        .route("/get_installed_packages", get(get_installed_packages))
        .route("/get_packages", post(get_packages))
        .route("/get_package/{name}", get(get_package))
        .route("/is_docker_running", get(is_docker_running))
        .route("/init_kittynode", post(init_kittynode))
        .route("/delete_kittynode", post(delete_kittynode))
        .route("/get_system_info", get(get_system_info))
        .route("/is_validator_installed", get(is_validator_installed))
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
    let app = app();
    let address = SocketAddr::from((Ipv4Addr::UNSPECIFIED, port));
    let listener = TcpListener::bind(address).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request};
    use tower::ServiceExt; // for `oneshot`

    #[tokio::test]
    async fn health_endpoint_returns_ok() {
        let app = app();
        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn delete_unknown_package_maps_to_404() {
        let app = app();
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/delete_package/does-not-exist")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
