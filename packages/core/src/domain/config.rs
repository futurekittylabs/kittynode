use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct Config {
    pub capabilities: Vec<String>,
    #[serde(alias = "serverUrl")]
    pub server_url: String,
    #[serde(default, alias = "onboardingCompleted")]
    pub onboarding_completed: bool,
    #[serde(default, alias = "autoStartDocker")]
    pub auto_start_docker: bool,
}
