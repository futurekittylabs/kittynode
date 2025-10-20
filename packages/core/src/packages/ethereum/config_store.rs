use std::{fs, path::PathBuf};

use eyre::{Context, Result};

use crate::infra::file::kittynode_path;

use super::config::EthereumConfig;
use super::docker_plan::ETHEREUM_NAME;

const CONFIG_FILE_NAME: &str = "config.toml";

fn config_path() -> Result<PathBuf> {
    let base_dir = kittynode_path()?;
    Ok(base_dir
        .join("packages")
        .join(ETHEREUM_NAME)
        .join(CONFIG_FILE_NAME))
}

pub fn load() -> Result<Option<EthereumConfig>> {
    let path = config_path()?;
    if !path.exists() {
        return Ok(None);
    }
    let raw = fs::read_to_string(&path).wrap_err_with(|| {
        format!(
            "Failed to read Ethereum configuration from {}",
            path.display()
        )
    })?;
    let cfg = toml::from_str(&raw).wrap_err_with(|| {
        format!(
            "Failed to parse Ethereum configuration from {}",
            path.display()
        )
    })?;
    Ok(Some(cfg))
}

pub fn save(cfg: &EthereumConfig) -> Result<()> {
    let path = config_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .wrap_err_with(|| format!("Failed to prepare {}", parent.display()))?;
    }
    let raw = toml::to_string_pretty(cfg).wrap_err("Failed to serialize Ethereum configuration")?;
    fs::write(&path, raw).wrap_err_with(|| {
        format!(
            "Failed to write Ethereum configuration to {}",
            path.display()
        )
    })?;
    Ok(())
}
