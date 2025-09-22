use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub capabilities: Vec<String>,
    #[serde(alias = "server_url")]
    pub server_url: String,
    #[serde(default, alias = "last_server_url")]
    pub last_server_url: String,
    #[serde(default, alias = "remote_connected")]
    pub remote_connected: bool,
    #[serde(default, alias = "onboarding_completed")]
    pub onboarding_completed: bool,
    #[serde(default, alias = "auto_start_docker")]
    pub auto_start_docker: bool,
}

impl Config {
    pub fn normalize(mut self) -> Self {
        self.server_url = self.server_url.trim().to_string();
        let has_server_url = !self.server_url.is_empty();

        let trimmed_last = self.last_server_url.trim();
        self.last_server_url = if trimmed_last.is_empty() {
            if has_server_url {
                self.server_url.clone()
            } else {
                String::new()
            }
        } else {
            trimmed_last.to_string()
        };

        self.remote_connected = has_server_url;

        self
    }
}
