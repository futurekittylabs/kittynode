use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct Config {
    pub capabilities: Vec<String>,
    pub server_url: String,
    #[serde(default)]
    pub onboarding_completed: bool,
}
