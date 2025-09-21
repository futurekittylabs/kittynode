use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OperationalMode {
    Local,
    Remote,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OperationalState {
    pub mode: OperationalMode,
    pub docker_running: bool,
    pub can_install: bool,
    pub can_manage: bool,
    pub diagnostics: Vec<String>,
}
