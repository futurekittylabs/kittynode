use axum::{
    Router,
    extract::{Path, Query},
    http::StatusCode,
    response::Json,
    routing::{get, post},
};
use kittynode_core::api::types::LogsQuery;
use kittynode_core::api::types::SystemInfo;
use kittynode_core::api::types::{Package, PackageConfig};
use std::collections::HashMap;

pub(crate) async fn hello_world() -> &'static str {
    "Hello World!"
}

pub(crate) async fn add_capability(
    Path(name): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    kittynode_core::api::add_capability(&name)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(StatusCode::OK)
}

pub(crate) async fn remove_capability(
    Path(name): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    kittynode_core::api::remove_capability(&name)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(StatusCode::OK)
}

pub(crate) async fn get_capabilities() -> Result<Json<Vec<String>>, (StatusCode, String)> {
    kittynode_core::api::get_capabilities()
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

pub(crate) async fn get_packages() -> Result<Json<HashMap<String, Package>>, (StatusCode, String)> {
    kittynode_core::api::get_packages()
        .map(|packages| {
            packages
                .into_iter()
                .map(|(name, package)| (name.to_string(), package))
                .collect::<HashMap<String, Package>>()
        })
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

pub(crate) async fn get_package_config(
    Path(name): Path<String>,
) -> Result<Json<PackageConfig>, (StatusCode, String)> {
    kittynode_core::api::get_package_config(&name)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

pub(crate) async fn update_package_config(
    Path(name): Path<String>,
    Json(config): Json<PackageConfig>,
) -> Result<StatusCode, (StatusCode, String)> {
    kittynode_core::api::update_package_config(&name, config)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(StatusCode::OK)
}

pub(crate) async fn install_package(
    Path(name): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    kittynode_core::api::install_package(&name)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(StatusCode::OK)
}

pub(crate) async fn delete_package(
    Path(name): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    kittynode_core::api::delete_package(&name, false)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(StatusCode::OK)
}

pub(crate) async fn get_installed_packages() -> Result<Json<Vec<Package>>, (StatusCode, String)> {
    kittynode_core::api::get_installed_packages()
        .await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

pub(crate) async fn is_docker_running() -> Result<StatusCode, (StatusCode, String)> {
    match kittynode_core::api::is_docker_running().await {
        true => Ok(StatusCode::OK),
        false => Err((
            StatusCode::SERVICE_UNAVAILABLE,
            "Docker is not running".to_string(),
        )),
    }
}

pub(crate) async fn init_kittynode() -> Result<StatusCode, (StatusCode, String)> {
    kittynode_core::api::init_kittynode()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(StatusCode::OK)
}

pub(crate) async fn delete_kittynode() -> Result<StatusCode, (StatusCode, String)> {
    kittynode_core::api::delete_kittynode()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(StatusCode::OK)
}

pub(crate) async fn get_system_info() -> Result<Json<SystemInfo>, (StatusCode, String)> {
    kittynode_core::api::get_system_info()
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

pub(crate) async fn get_container_logs(
    Path(container_name): Path<String>,
    Query(params): Query<LogsQuery>,
) -> Result<Json<Vec<String>>, (StatusCode, String)> {
    kittynode_core::api::get_container_logs(&container_name, params.tail)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/", get(hello_world))
        .route("/add_capability/:name", post(add_capability))
        .route("/remove_capability/:name", post(remove_capability))
        .route("/get_capabilities", get(get_capabilities))
        .route("/get_packages", get(get_packages))
        .route("/get_package_config/:name", get(get_package_config))
        .route("/update_package_config/:name", post(update_package_config))
        .route("/install_package/:name", post(install_package))
        .route("/delete_package/:name", post(delete_package))
        .route("/get_installed_packages", get(get_installed_packages))
        .route("/is_docker_running", get(is_docker_running))
        .route("/init_kittynode", post(init_kittynode))
        .route("/delete_kittynode", post(delete_kittynode))
        .route("/get_system_info", get(get_system_info))
        .route("/logs/:container_name", get(get_container_logs));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
