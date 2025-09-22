use crate::infra::file::kittynode_path;
use eyre::{Result, WrapErr};
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct WebProcessState {
    pub pid: u32,
    pub port: u16,
    pub binary: PathBuf,
    #[serde(default)]
    pub token: Option<String>,
    #[serde(default)]
    pub log_path: Option<PathBuf>,
}

const STATE_FILE_NAME: &str = "kittynode-web.json";
const RUNTIME_DIR: &str = "runtime";
const LOG_FILE_NAME: &str = "kittynode-web.log";

pub fn load_state() -> Result<Option<WebProcessState>> {
    let path = state_file_path()?;
    if !path.exists() {
        return Ok(None);
    }

    let data = fs::read_to_string(&path).wrap_err("Failed to read kittynode-web state file")?;
    let state =
        serde_json::from_str(&data).wrap_err("Failed to deserialize kittynode-web state")?;
    Ok(Some(state))
}

pub fn save_state(state: &WebProcessState) -> Result<()> {
    let path = state_file_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .wrap_err("Failed to create directories for kittynode-web state file")?;
    }

    let data = serde_json::to_string(state).wrap_err("Failed to serialize kittynode-web state")?;
    fs::write(&path, data).wrap_err("Failed to write kittynode-web state file")?;
    Ok(())
}

pub fn clear_state() -> Result<()> {
    let path = state_file_path()?;
    if path.exists() {
        fs::remove_file(path).wrap_err("Failed to remove kittynode-web state file")?;
    }
    Ok(())
}

fn state_file_path() -> Result<PathBuf> {
    let mut path = kittynode_path()?;
    path.push(RUNTIME_DIR);
    path.push(STATE_FILE_NAME);
    Ok(path)
}

pub fn log_file_path() -> Result<PathBuf> {
    let mut path = kittynode_path()?;
    path.push(RUNTIME_DIR);
    path.push(LOG_FILE_NAME);
    Ok(path)
}
