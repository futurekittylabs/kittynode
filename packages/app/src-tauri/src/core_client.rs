use async_trait::async_trait;
use eyre::{Context, Result};
use kittynode_core::api;
use kittynode_core::api::DockerStartStatus;
use kittynode_core::api::types::{Config, OperationalState, Package, PackageConfig, SystemInfo};
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tauri_plugin_http::reqwest::{Client, StatusCode};

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
pub trait CoreClient: Send + Sync {
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
    async fn delete_kittynode(&self) -> Result<()>;
    async fn init_kittynode(&self) -> Result<()>;
    async fn get_package_config(&self, name: &str) -> Result<PackageConfig>;
    async fn update_package_config(&self, name: &str, config: PackageConfig) -> Result<()>;
    async fn is_docker_running(&self) -> Result<bool>;
    async fn start_docker_if_needed(&self) -> Result<DockerStartStatus>;
    async fn get_operational_state(&self) -> Result<OperationalState>;
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

        if response.status().is_success() {
            Ok(response
                .json::<T>()
                .await
                .wrap_err_with(|| format!("Failed to deserialize response from {}", path))?)
        } else {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            Err(eyre::eyre!(
                "HTTP {} when requesting {}: {}",
                status,
                path,
                body
            ))
        }
    }

    async fn post_unit<B>(&self, path: &str, body: Option<&B>) -> Result<()>
    where
        B: Serialize + Sync,
    {
        let request = self.client.post(self.url(path));
        let request = if let Some(body) = body {
            request.json(body)
        } else {
            request
        };
        let response = request
            .send()
            .await
            .wrap_err_with(|| format!("Failed to POST {}", path))?;

        if response.status().is_success() {
            Ok(())
        } else {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            Err(eyre::eyre!(
                "HTTP {} when posting {}: {}",
                status,
                path,
                body
            ))
        }
    }

    async fn post_json<T, B>(&self, path: &str, body: Option<&B>) -> Result<T>
    where
        T: DeserializeOwned,
        B: Serialize + Sync,
    {
        let request = self.client.post(self.url(path));
        let request = if let Some(body) = body {
            request.json(body)
        } else {
            request
        };
        let response = request
            .send()
            .await
            .wrap_err_with(|| format!("Failed to POST {}", path))?;

        if response.status().is_success() {
            Ok(response
                .json::<T>()
                .await
                .wrap_err_with(|| format!("Failed to deserialize response from {}", path))?)
        } else {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            Err(eyre::eyre!(
                "HTTP {} when posting {}: {}",
                status,
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
