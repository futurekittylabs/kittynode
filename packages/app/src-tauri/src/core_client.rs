use kittynode_core::api;
use kittynode_core::api::types::{Package, PackageConfig, SystemInfo};
use serde::de::DeserializeOwned;
use std::collections::HashMap;
use std::sync::{Arc, LazyLock, RwLock};
use tauri_plugin_http::reqwest;

pub static HTTP_CLIENT: LazyLock<reqwest::Client> = LazyLock::new(reqwest::Client::new);

pub struct CoreClient {
    inner: CoreClientInner,
}

enum CoreClientInner {
    Local,
    Remote(HttpCoreClient),
}

impl CoreClient {
    fn local() -> Self {
        Self {
            inner: CoreClientInner::Local,
        }
    }

    fn remote(base_url: String) -> Self {
        Self {
            inner: CoreClientInner::Remote(HttpCoreClient::new(base_url)),
        }
    }

    pub async fn add_capability(&self, name: &str) -> Result<(), String> {
        match &self.inner {
            CoreClientInner::Local => api::add_capability(name).map_err(|e| e.to_string()),
            CoreClientInner::Remote(client) => client.add_capability(name).await,
        }
    }

    pub async fn remove_capability(&self, name: &str) -> Result<(), String> {
        match &self.inner {
            CoreClientInner::Local => api::remove_capability(name).map_err(|e| e.to_string()),
            CoreClientInner::Remote(client) => client.remove_capability(name).await,
        }
    }

    pub async fn get_capabilities(&self) -> Result<Vec<String>, String> {
        match &self.inner {
            CoreClientInner::Local => api::get_capabilities().map_err(|e| e.to_string()),
            CoreClientInner::Remote(client) => client.get_capabilities().await,
        }
    }

    pub async fn get_packages(&self) -> Result<HashMap<String, Package>, String> {
        match &self.inner {
            CoreClientInner::Local => api::get_packages().map_err(|e| e.to_string()),
            CoreClientInner::Remote(client) => client.get_packages().await,
        }
    }

    pub async fn get_installed_packages(&self) -> Result<Vec<Package>, String> {
        match &self.inner {
            CoreClientInner::Local => api::get_installed_packages()
                .await
                .map_err(|e| e.to_string()),
            CoreClientInner::Remote(client) => client.get_installed_packages().await,
        }
    }

    pub async fn is_docker_running(&self) -> Result<bool, String> {
        match &self.inner {
            CoreClientInner::Local => Ok(api::is_docker_running().await),
            CoreClientInner::Remote(client) => client.is_docker_running().await,
        }
    }

    pub async fn install_package(&self, name: &str) -> Result<(), String> {
        match &self.inner {
            CoreClientInner::Local => api::install_package(name).await.map_err(|e| e.to_string()),
            CoreClientInner::Remote(client) => client.install_package(name).await,
        }
    }

    pub async fn delete_package(&self, name: &str, include_images: bool) -> Result<(), String> {
        match &self.inner {
            CoreClientInner::Local => api::delete_package(name, include_images)
                .await
                .map_err(|e| e.to_string()),
            CoreClientInner::Remote(client) => client.delete_package(name, include_images).await,
        }
    }

    pub async fn delete_kittynode(&self) -> Result<(), String> {
        match &self.inner {
            CoreClientInner::Local => api::delete_kittynode().map_err(|e| e.to_string()),
            CoreClientInner::Remote(client) => client.delete_kittynode().await,
        }
    }

    pub async fn init_kittynode(&self) -> Result<(), String> {
        match &self.inner {
            CoreClientInner::Local => api::init_kittynode().map_err(|e| e.to_string()),
            CoreClientInner::Remote(client) => client.init_kittynode().await,
        }
    }

    pub async fn get_system_info(&self) -> Result<SystemInfo, String> {
        match &self.inner {
            CoreClientInner::Local => api::get_system_info().map_err(|e| e.to_string()),
            CoreClientInner::Remote(client) => client.get_system_info().await,
        }
    }

    pub async fn get_container_logs(
        &self,
        container_name: &str,
        tail_lines: Option<usize>,
    ) -> Result<Vec<String>, String> {
        match &self.inner {
            CoreClientInner::Local => api::get_container_logs(container_name, tail_lines)
                .await
                .map_err(|e| e.to_string()),
            CoreClientInner::Remote(client) => {
                client.get_container_logs(container_name, tail_lines).await
            }
        }
    }

    pub async fn get_package_config(&self, name: &str) -> Result<PackageConfig, String> {
        match &self.inner {
            CoreClientInner::Local => api::get_package_config(name)
                .await
                .map_err(|e| e.to_string()),
            CoreClientInner::Remote(client) => client.get_package_config(name).await,
        }
    }

    pub async fn update_package_config(
        &self,
        name: &str,
        config: PackageConfig,
    ) -> Result<(), String> {
        match &self.inner {
            CoreClientInner::Local => api::update_package_config(name, config)
                .await
                .map_err(|e| e.to_string()),
            CoreClientInner::Remote(client) => client.update_package_config(name, config).await,
        }
    }
}

pub struct HttpCoreClient {
    base_url: String,
}

impl HttpCoreClient {
    pub fn new(base_url: String) -> Self {
        let clean = base_url.trim_end_matches('/').to_string();
        Self { base_url: clean }
    }

    fn url(&self, path: &str) -> String {
        let path = path.trim_start_matches('/');
        format!("{}/{}", self.base_url, path)
    }

    async fn get_json<T>(&self, path: &str) -> Result<T, String>
    where
        T: DeserializeOwned,
    {
        let response = HTTP_CLIENT
            .get(self.url(path))
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(format!("Request failed with status {}: {}", status, body));
        }

        response.json::<T>().await.map_err(|e| e.to_string())
    }

    async fn post_empty(&self, path: &str) -> Result<(), String> {
        let response = HTTP_CLIENT
            .post(self.url(path))
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !response.status().is_success() {
            return Err(format!("Request failed with status {}", response.status()));
        }

        Ok(())
    }
    async fn add_capability(&self, name: &str) -> Result<(), String> {
        self.post_empty(&format!("add_capability/{}", name)).await
    }

    async fn remove_capability(&self, name: &str) -> Result<(), String> {
        self.post_empty(&format!("remove_capability/{}", name))
            .await
    }

    async fn get_capabilities(&self) -> Result<Vec<String>, String> {
        self.get_json("get_capabilities").await
    }

    async fn get_packages(&self) -> Result<HashMap<String, Package>, String> {
        self.get_json("get_packages").await
    }

    async fn get_installed_packages(&self) -> Result<Vec<Package>, String> {
        self.get_json("get_installed_packages").await
    }

    async fn is_docker_running(&self) -> Result<bool, String> {
        let response = HTTP_CLIENT
            .get(self.url("is_docker_running"))
            .send()
            .await
            .map_err(|e| e.to_string())?;

        match response.status() {
            status if status.is_success() => Ok(true),
            reqwest::StatusCode::SERVICE_UNAVAILABLE => Ok(false),
            status => {
                let body = response.text().await.unwrap_or_default();
                Err(format!(
                    "Docker status request failed with status {}: {}",
                    status, body
                ))
            }
        }
    }

    async fn install_package(&self, name: &str) -> Result<(), String> {
        self.post_empty(&format!("install_package/{}", name)).await
    }

    async fn delete_package(&self, name: &str, _include_images: bool) -> Result<(), String> {
        self.post_empty(&format!("delete_package/{}", name)).await
    }

    async fn delete_kittynode(&self) -> Result<(), String> {
        self.post_empty("delete_kittynode").await
    }

    async fn init_kittynode(&self) -> Result<(), String> {
        self.post_empty("init_kittynode").await
    }

    async fn get_system_info(&self) -> Result<SystemInfo, String> {
        self.get_json("get_system_info").await
    }

    async fn get_container_logs(
        &self,
        container_name: &str,
        tail_lines: Option<usize>,
    ) -> Result<Vec<String>, String> {
        let mut url = self.url(&format!("logs/{}", container_name));
        if let Some(lines) = tail_lines {
            url = format!("{}?tail={}", url, lines);
        }

        let response = HTTP_CLIENT
            .get(url)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(format!(
                "Failed to fetch container logs: {} - {}",
                status, body
            ));
        }

        response
            .json::<Vec<String>>()
            .await
            .map_err(|e| e.to_string())
    }

    async fn get_package_config(&self, name: &str) -> Result<PackageConfig, String> {
        self.get_json(&format!("get_package_config/{}", name)).await
    }

    async fn update_package_config(&self, name: &str, config: PackageConfig) -> Result<(), String> {
        let response = HTTP_CLIENT
            .post(self.url(&format!("update_package_config/{}", name)))
            .json(&config)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !response.status().is_success() {
            return Err(format!(
                "Failed to update package config: {}",
                response.status()
            ));
        }

        Ok(())
    }
}

pub struct CoreClientState {
    client: RwLock<Arc<CoreClient>>,
}

impl CoreClientState {
    pub fn initialize() -> Self {
        let server_url = api::get_config()
            .map(|config| config.server_url)
            .unwrap_or_default();

        Self {
            client: RwLock::new(Self::build_client(server_url)),
        }
    }

    pub fn client(&self) -> Arc<CoreClient> {
        self.client
            .read()
            .map(|client| Arc::clone(&client))
            .unwrap_or_else(|_| Self::build_client(String::new()))
    }

    pub fn set_server_url(&self, server_url: String) -> Result<(), String> {
        api::set_server_url(server_url.clone()).map_err(|e| e.to_string())?;
        let mut guard = self
            .client
            .write()
            .map_err(|_| "Failed to lock core client state".to_string())?;
        *guard = Self::build_client(server_url);
        Ok(())
    }

    fn build_client(server_url: String) -> Arc<CoreClient> {
        if server_url.trim().is_empty() {
            Arc::new(CoreClient::local())
        } else {
            Arc::new(CoreClient::remote(server_url))
        }
    }
}
