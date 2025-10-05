use async_trait::async_trait;
use eyre::{Context, Result};
use kittynode_core::api;
use kittynode_core::api::DockerStartStatus;
use kittynode_core::api::types::{
    Config, OperationalState, Package, PackageConfig, PackageRuntimeState, SystemInfo,
};
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tauri_plugin_http::reqwest::{Client, Response, StatusCode};

fn normalize_base_url(server_url: &str) -> Option<String> {
    let trimmed = server_url.trim();
    let has_valid_scheme = trimmed.starts_with("http://") || trimmed.starts_with("https://");
    if trimmed.is_empty() || !has_valid_scheme {
        None
    } else {
        Some(trimmed.trim_end_matches('/').to_string())
    }
}

#[async_trait]
pub trait CoreClient: Send + Sync + std::any::Any {
    /// Retrieve the current package catalog from the core. Implementations may call the
    /// core directly (local) or proxy the HTTP API (remote) but must return the same data shape.
    async fn get_packages(&self) -> Result<HashMap<String, Package>>;
    /// List the enabled capabilities.
    async fn get_capabilities(&self) -> Result<Vec<String>>;
    /// Enable a capability on the node.
    async fn add_capability(&self, name: &str) -> Result<()>;
    /// Disable a capability on the node.
    async fn remove_capability(&self, name: &str) -> Result<()>;
    /// Return the installed packages as reported by the core.
    async fn get_installed_packages(&self) -> Result<Vec<Package>>;
    /// Retrieve system information (CPU, memory, storage).
    async fn system_info(&self) -> Result<SystemInfo>;
    /// Fetch container logs; implementations should proxy the same parameters to the core.
    async fn get_container_logs(
        &self,
        container_name: &str,
        tail_lines: Option<usize>,
    ) -> Result<Vec<String>>;
    /// Install a package via the core.
    async fn install_package(&self, name: &str) -> Result<()>;
    /// Delete a package, optionally removing images.
    async fn delete_package(&self, name: &str, include_images: bool) -> Result<()>;
    /// Stop all containers for a package.
    async fn stop_package(&self, name: &str) -> Result<()>;
    /// Resume previously stopped containers for a package.
    async fn resume_package(&self, name: &str) -> Result<()>;
    /// Retrieve the runtime state for a package.
    async fn get_package_runtime_state(&self, name: &str) -> Result<PackageRuntimeState>;
    /// Remove Kittynode data from disk.
    async fn delete_kittynode(&self) -> Result<()>;
    /// Initialize Kittynode configuration and directories.
    async fn init_kittynode(&self) -> Result<()>;
    /// Fetch the persisted configuration for a package.
    async fn get_package_config(&self, name: &str) -> Result<PackageConfig>;
    /// Persist new configuration values for a package.
    async fn update_package_config(&self, name: &str, config: PackageConfig) -> Result<()>;
    /// Check whether Docker is reachable for the current mode.
    async fn is_docker_running(&self) -> Result<bool>;
    /// Attempt to start Docker if auto-start policies allow it.
    async fn start_docker_if_needed(&self) -> Result<DockerStartStatus>;
    /// Report the operational capabilities (install/manage, diagnostics, etc.).
    async fn get_operational_state(&self) -> Result<OperationalState>;
    /// Retrieve runtime states for multiple packages in a single request.
    async fn get_package_runtime_states(
        &self,
        names: &[String],
    ) -> Result<HashMap<String, PackageRuntimeState>>;
}

#[cfg(test)]
pub(crate) trait CoreClientTestExt {
    fn as_any(&self) -> &(dyn std::any::Any + Send + Sync);
}

#[cfg(test)]
impl CoreClientTestExt for dyn CoreClient {
    fn as_any(&self) -> &(dyn std::any::Any + Send + Sync) {
        self
    }
}

#[cfg(test)]
impl CoreClientTestExt for Arc<dyn CoreClient> {
    fn as_any(&self) -> &(dyn std::any::Any + Send + Sync) {
        (**self).as_any()
    }
}

pub struct LocalCoreClient;

#[async_trait]
impl CoreClient for LocalCoreClient {
    async fn get_packages(&self) -> Result<HashMap<String, Package>> {
        api::get_packages()
    }

    async fn get_capabilities(&self) -> Result<Vec<String>> {
        api::get_capabilities()
    }

    async fn add_capability(&self, name: &str) -> Result<()> {
        api::add_capability(name)
    }

    async fn remove_capability(&self, name: &str) -> Result<()> {
        api::remove_capability(name)
    }

    async fn get_installed_packages(&self) -> Result<Vec<Package>> {
        api::get_installed_packages().await
    }

    async fn system_info(&self) -> Result<SystemInfo> {
        api::get_system_info()
    }

    async fn get_container_logs(
        &self,
        container_name: &str,
        tail_lines: Option<usize>,
    ) -> Result<Vec<String>> {
        api::get_container_logs(container_name, tail_lines).await
    }

    async fn install_package(&self, name: &str) -> Result<()> {
        api::install_package(name).await
    }

    async fn delete_package(&self, name: &str, include_images: bool) -> Result<()> {
        api::delete_package(name, include_images).await
    }

    async fn stop_package(&self, name: &str) -> Result<()> {
        api::stop_package(name).await
    }

    async fn resume_package(&self, name: &str) -> Result<()> {
        api::resume_package(name).await
    }

    async fn get_package_runtime_state(&self, name: &str) -> Result<PackageRuntimeState> {
        api::get_package_runtime_state(name).await
    }

    async fn delete_kittynode(&self) -> Result<()> {
        api::delete_kittynode()
    }

    async fn init_kittynode(&self) -> Result<()> {
        api::init_kittynode()
    }

    async fn get_package_config(&self, name: &str) -> Result<PackageConfig> {
        api::get_package_config(name).await
    }

    async fn update_package_config(&self, name: &str, config: PackageConfig) -> Result<()> {
        api::update_package_config(name, config).await
    }

    async fn is_docker_running(&self) -> Result<bool> {
        Ok(api::is_docker_running().await)
    }

    async fn start_docker_if_needed(&self) -> Result<DockerStartStatus> {
        api::start_docker_if_needed().await
    }

    async fn get_operational_state(&self) -> Result<OperationalState> {
        api::get_operational_state().await
    }

    async fn get_package_runtime_states(
        &self,
        names: &[String],
    ) -> Result<HashMap<String, PackageRuntimeState>> {
        api::get_packages_runtime_state(names).await
    }
}

pub struct HttpCoreClient {
    base_url: String,
    client: Client,
}

impl HttpCoreClient {
    pub fn new(server_url: &str) -> Result<Self> {
        let base_url = normalize_base_url(server_url)
            .ok_or_else(|| eyre::eyre!("Server URL must not be empty for HTTP client"))?;
        Ok(Self {
            base_url,
            client: Client::new(),
        })
    }

    fn url(&self, path: &str) -> String {
        if path.starts_with('/') {
            format!("{}{}", self.base_url, path)
        } else {
            format!("{}/{}", self.base_url, path)
        }
    }

    async fn get_json<T>(&self, path: &str) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let response = self
            .client
            .get(self.url(path))
            .send()
            .await
            .wrap_err_with(|| format!("Failed to GET {}", path))?;
        let response = Self::ensure_success(response, path, "requesting").await?;
        response
            .json::<T>()
            .await
            .wrap_err_with(|| format!("Failed to deserialize response from {}", path))
    }

    async fn post_unit<B>(&self, path: &str, body: Option<&B>) -> Result<()>
    where
        B: Serialize + Sync,
    {
        let response = self.post_request(path, body).await?;
        Self::ensure_success(response, path, "posting").await?;
        Ok(())
    }

    async fn post_json<T, B>(&self, path: &str, body: Option<&B>) -> Result<T>
    where
        T: DeserializeOwned,
        B: Serialize + Sync,
    {
        let response = self.post_request(path, body).await?;
        let response = Self::ensure_success(response, path, "posting").await?;
        response
            .json::<T>()
            .await
            .wrap_err_with(|| format!("Failed to deserialize response from {}", path))
    }

    async fn post_request<B>(&self, path: &str, body: Option<&B>) -> Result<Response>
    where
        B: Serialize + Sync,
    {
        let request = self.client.post(self.url(path));
        let request = if let Some(body) = body {
            request.json(body)
        } else {
            request
        };
        request
            .send()
            .await
            .wrap_err_with(|| format!("Failed to POST {}", path))
    }

    async fn ensure_success(response: Response, path: &str, action: &str) -> Result<Response> {
        if response.status().is_success() {
            Ok(response)
        } else {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            Err(eyre::eyre!(
                "HTTP {} when {} {}: {}",
                status,
                action,
                path,
                body
            ))
        }
    }
}

#[async_trait]
impl CoreClient for HttpCoreClient {
    async fn get_packages(&self) -> Result<HashMap<String, Package>> {
        self.get_json("/get_packages").await
    }

    async fn get_capabilities(&self) -> Result<Vec<String>> {
        self.get_json("/get_capabilities").await
    }

    async fn add_capability(&self, name: &str) -> Result<()> {
        self.post_unit(&format!("/add_capability/{name}"), Option::<&()>::None)
            .await
    }

    async fn remove_capability(&self, name: &str) -> Result<()> {
        self.post_unit(&format!("/remove_capability/{name}"), Option::<&()>::None)
            .await
    }

    async fn get_installed_packages(&self) -> Result<Vec<Package>> {
        self.get_json("/get_installed_packages").await
    }

    async fn system_info(&self) -> Result<SystemInfo> {
        self.get_json("/get_system_info").await
    }

    async fn get_container_logs(
        &self,
        container_name: &str,
        tail_lines: Option<usize>,
    ) -> Result<Vec<String>> {
        let mut path = format!("/logs/{}", container_name);
        if let Some(tail) = tail_lines {
            path = format!("{}?tail={}", path, tail);
        }
        self.get_json(&path).await
    }

    async fn install_package(&self, name: &str) -> Result<()> {
        self.post_unit(&format!("/install_package/{name}"), Option::<&()>::None)
            .await
    }

    async fn delete_package(&self, name: &str, include_images: bool) -> Result<()> {
        let mut path = format!("/delete_package/{name}");
        if include_images {
            path.push_str("?include_images=true");
        }
        self.post_unit(&path, Option::<&()>::None).await
    }

    async fn stop_package(&self, name: &str) -> Result<()> {
        self.post_unit(&format!("/stop_package/{name}"), Option::<&()>::None)
            .await
    }

    async fn resume_package(&self, name: &str) -> Result<()> {
        self.post_unit(&format!("/resume_package/{name}"), Option::<&()>::None)
            .await
    }

    async fn get_package_runtime_state(&self, name: &str) -> Result<PackageRuntimeState> {
        self.get_json(&format!("/package_runtime/{name}")).await
    }

    async fn delete_kittynode(&self) -> Result<()> {
        self.post_unit("/delete_kittynode", Option::<&()>::None)
            .await
    }

    async fn init_kittynode(&self) -> Result<()> {
        self.post_unit("/init_kittynode", Option::<&()>::None).await
    }

    async fn get_package_config(&self, name: &str) -> Result<PackageConfig> {
        self.get_json(&format!("/get_package_config/{name}")).await
    }

    async fn update_package_config(&self, name: &str, config: PackageConfig) -> Result<()> {
        self.post_unit(&format!("/update_package_config/{name}"), Some(&config))
            .await
    }

    async fn is_docker_running(&self) -> Result<bool> {
        let response = self
            .client
            .get(self.url("/is_docker_running"))
            .send()
            .await
            .wrap_err("Failed to check Docker status")?;

        match response.status() {
            StatusCode::OK => Ok(true),
            StatusCode::SERVICE_UNAVAILABLE => Ok(false),
            status => {
                let body = response.text().await.unwrap_or_default();
                Err(eyre::eyre!(
                    "Unexpected status {} when checking Docker status: {}",
                    status,
                    body
                ))
            }
        }
    }

    async fn start_docker_if_needed(&self) -> Result<DockerStartStatus> {
        self.post_json::<DockerStartStatus, ()>("/start_docker_if_needed", None)
            .await
    }

    async fn get_operational_state(&self) -> Result<OperationalState> {
        // The remote HTTP service reports its own mode (usually local); we override it to mark
        // that this client is operating in remote mode while preserving the rest of the payload.
        self.get_json::<OperationalState>("/get_operational_state")
            .await
            .map(|mut state| {
                state.mode = kittynode_core::api::types::OperationalMode::Remote;
                state
            })
    }

    async fn get_package_runtime_states(
        &self,
        names: &[String],
    ) -> Result<HashMap<String, PackageRuntimeState>> {
        #[derive(Serialize)]
        struct RuntimeStatesRequest<'a> {
            names: &'a [String],
        }

        self.post_json("/package_runtime", Some(&RuntimeStatesRequest { names }))
            .await
    }
}

pub struct CoreClientManager {
    inner: Mutex<Arc<dyn CoreClient>>,
}

impl CoreClientManager {
    pub fn new(config: &Config) -> Result<Self> {
        let client = create_client(config)?;
        Ok(Self {
            inner: Mutex::new(client),
        })
    }

    pub fn client(&self) -> Arc<dyn CoreClient> {
        self.inner
            .lock()
            .map(|guard| guard.clone())
            .unwrap_or_else(|poisoned| poisoned.into_inner().clone())
    }

    pub fn reload(&self, config: &Config) -> Result<()> {
        let mut guard = self
            .inner
            .lock()
            .map_err(|err| eyre::eyre!("failed to reload core client: {err}"))?;
        *guard = create_client(config)?;
        Ok(())
    }
}

fn create_client(config: &Config) -> Result<Arc<dyn CoreClient>> {
    if let Some(base_url) = normalize_base_url(&config.server_url) {
        Ok(Arc::new(HttpCoreClient::new(&base_url)?) as Arc<dyn CoreClient>)
    } else {
        Ok(Arc::new(LocalCoreClient) as Arc<dyn CoreClient>)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        Json, Router,
        extract::{Path, State},
        http::StatusCode,
        response::{IntoResponse, Response},
        routing::{get, post},
    };
    use eyre::Result as EyreResult;
    use serde::Deserialize;
    use std::collections::HashMap as StdHashMap;
    use std::future::IntoFuture;
    use std::sync::Arc;
    use tokio::net::TcpListener;
    use tokio::sync::Mutex as AsyncMutex;
    use tokio::task::JoinHandle;

    #[test]
    fn normalize_base_url_handles_common_cases() {
        assert_eq!(
            super::normalize_base_url(" https://example.com/ "),
            Some("https://example.com".to_string())
        );
        assert_eq!(
            super::normalize_base_url("http://example.com"),
            Some("http://example.com".to_string())
        );
        assert_eq!(super::normalize_base_url(""), None);
        assert_eq!(super::normalize_base_url("example.com"), None);
        assert_eq!(super::normalize_base_url("ftp://example.com"), None);
    }

    #[tokio::test]
    async fn core_client_manager_reloads_between_local_and_remote() -> EyreResult<()> {
        let (base_url, server_handle) = spawn_test_server().await?;

        let mut config = Config::default();
        config.server_url.clear();
        let manager = CoreClientManager::new(&config)?;
        assert!(manager.client().as_any().is::<LocalCoreClient>());

        config.server_url = format!(" {}/ ", base_url);
        manager.reload(&config)?;
        let client = manager.client();
        assert!(client.as_any().is::<HttpCoreClient>());

        let capabilities = client.get_capabilities().await?;
        assert_eq!(capabilities, vec!["remote-capability".to_string()]);

        server_handle.abort();
        let _ = server_handle.await;
        Ok(())
    }

    #[tokio::test]
    async fn http_core_client_handles_success_and_error_paths() -> EyreResult<()> {
        let (base_url, server_handle) = spawn_test_server().await?;
        let client = HttpCoreClient::new(&base_url)?;

        assert_eq!(
            client.get_capabilities().await?,
            vec!["remote-capability".to_string()]
        );
        client.add_capability("beta").await?;
        client.install_package("ok").await?;
        let states = client
            .get_package_runtime_states(&["alpha".to_string()])
            .await?;
        assert!(states.get("alpha").is_some_and(|state| state.running));
        assert_eq!(
            client.start_docker_if_needed().await?,
            DockerStartStatus::Running
        );

        let install_err = client
            .install_package("fail")
            .await
            .expect_err("expected failure");
        assert!(install_err.to_string().contains("HTTP 500"));

        assert!(client.is_docker_running().await?);
        assert!(!client.is_docker_running().await?);
        let docker_err = client
            .is_docker_running()
            .await
            .expect_err("expected docker error");
        assert!(docker_err.to_string().contains("418"));

        server_handle.abort();
        let _ = server_handle.await;
        Ok(())
    }

    #[derive(Clone, Default)]
    struct TestState {
        docker_calls: Arc<AsyncMutex<usize>>,
    }

    #[derive(Deserialize)]
    struct RuntimeStatesRequest {
        names: Vec<String>,
    }

    async fn spawn_test_server() -> EyreResult<(String, JoinHandle<()>)> {
        let state = TestState::default();
        let router = Router::new()
            .route("/get_capabilities", get(get_capabilities))
            .route("/add_capability/:name", post(add_capability))
            .route("/install_package/:name", post(install_package))
            .route("/package_runtime", post(package_runtime_states))
            .route("/start_docker_if_needed", post(start_docker_if_needed))
            .route("/is_docker_running", get(is_docker_running))
            .with_state(state);

        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let addr = listener.local_addr()?;
        let server = axum::serve(listener, router);
        let handle = tokio::spawn(async move {
            let _ = server.into_future().await;
        });
        Ok((format!("http://{}", addr), handle))
    }

    async fn get_capabilities() -> Json<Vec<String>> {
        Json(vec!["remote-capability".to_string()])
    }

    async fn add_capability(Path(name): Path<String>) -> StatusCode {
        assert_eq!(name, "beta");
        StatusCode::NO_CONTENT
    }

    async fn install_package(Path(name): Path<String>) -> Response {
        if name == "fail" {
            (StatusCode::INTERNAL_SERVER_ERROR, "install failed").into_response()
        } else {
            StatusCode::NO_CONTENT.into_response()
        }
    }

    async fn package_runtime_states(
        Json(payload): Json<RuntimeStatesRequest>,
    ) -> Json<StdHashMap<String, PackageRuntimeState>> {
        let states = payload
            .names
            .into_iter()
            .map(|name| (name, PackageRuntimeState { running: true }))
            .collect();
        Json(states)
    }

    async fn start_docker_if_needed() -> Json<DockerStartStatus> {
        Json(DockerStartStatus::Running)
    }

    async fn is_docker_running(State(state): State<TestState>) -> Response {
        let mut calls = state.docker_calls.lock().await;
        *calls += 1;
        match *calls {
            1 => StatusCode::OK.into_response(),
            2 => StatusCode::SERVICE_UNAVAILABLE.into_response(),
            _ => (StatusCode::IM_A_TEAPOT, "teapot").into_response(),
        }
    }
}
