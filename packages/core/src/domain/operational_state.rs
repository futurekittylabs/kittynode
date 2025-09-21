use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OperationalMode {
    Local,
    Remote,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OperationalState {
    pub mode: OperationalMode,
    #[serde(alias = "dockerRunning")]
    pub docker_running: bool,
    #[serde(alias = "canInstall")]
    pub can_install: bool,
    #[serde(alias = "canManage")]
    pub can_manage: bool,
    pub diagnostics: Vec<String>,
}
