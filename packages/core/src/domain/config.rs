use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub capabilities: Vec<String>,
    #[serde(alias = "server_url")]
    pub server_url: String,
    #[serde(default, alias = "onboarding_completed")]
    pub onboarding_completed: bool,
    #[serde(default, alias = "auto_start_docker")]
    pub auto_start_docker: bool,
}
