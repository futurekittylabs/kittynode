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
    pub has_remote_server: bool,
    #[serde(default, alias = "onboarding_completed")]
    pub onboarding_completed: bool,
    #[serde(default, alias = "auto_start_docker")]
    pub auto_start_docker: bool,
}

impl Config {
    pub fn normalize(&mut self) {
        self.server_url = self.server_url.trim().to_string();
        let has_server_url = !self.server_url.is_empty();

        if self.last_server_url.trim().is_empty() && has_server_url {
            self.last_server_url = self.server_url.clone();
        } else {
            self.last_server_url = self.last_server_url.trim().to_string();
        }

        self.has_remote_server = has_server_url;
    }
}

#[cfg(test)]
mod tests {
    use super::Config;

    #[test]
    fn normalizes_server_and_last_urls() {
        let mut config = Config {
            server_url: " https://example.com ".to_string(),
            last_server_url: String::new(),
            has_remote_server: false,
            ..Default::default()
        };

        config.normalize();

        assert_eq!(config.server_url, "https://example.com");
        assert_eq!(config.last_server_url, "https://example.com");
        assert!(config.has_remote_server);
    }

    #[test]
    fn normalize_trims_last_url_and_updates_flag() {
        let mut config = Config {
            server_url: String::new(),
            last_server_url: " https://cached.example.com ".to_string(),
            has_remote_server: true,
            ..Default::default()
        };

        config.normalize();

        assert_eq!(config.server_url, "");
        assert_eq!(config.last_server_url, "https://cached.example.com");
        assert!(!config.has_remote_server);
    }
}
