use assert_cmd::Command;
use axum::extract::{Path, Query};
use escargot::CargoBuild;
use eyre::{Result, eyre};
use kittynode_core::api;
use serde_json::{self, Value};
use std::path::PathBuf;
use std::sync::OnceLock;

const STORAGE_TOLERANCE_BYTES: i64 = 5_000_000;
const CPU_FREQUENCY_TOLERANCE_GHZ: f64 = 0.2;

fn cli_json(args: &[&str]) -> Result<Value> {
    let mut cmd = cli_command()?;
    let output = cmd
        .args(["--format", "json"])
        .args(args)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    Ok(serde_json::from_slice(&output)?)
}

fn cli_command() -> Result<Command> {
    if let Ok(cmd) = Command::cargo_bin("kittynode") {
        return Ok(cmd);
    }

    let path = cli_bin_path()?;
    Ok(Command::new(path))
}

fn cli_bin_path() -> Result<&'static std::path::Path> {
    static CLI_BIN: OnceLock<PathBuf> = OnceLock::new();

    if let Some(path) = CLI_BIN.get() {
        return Ok(path.as_path());
    }

    let path = if let Ok(env_path) = std::env::var("CARGO_BIN_EXE_kittynode") {
        PathBuf::from(env_path)
    } else {
        CargoBuild::new()
            .bin("kittynode")
            .run()
            .map_err(|err| eyre!("failed to build kittynode CLI: {err}"))?
            .path()
            .to_path_buf()
    };

    CLI_BIN
        .set(path)
        .map_err(|_| eyre!("failed to cache CLI binary path"))?;

    Ok(CLI_BIN
        .get()
        .expect("CLI binary path should be initialized")
        .as_path())
}

fn align_storage_available_bytes(reference: &Value, candidate: &mut Value) -> Result<()> {
    let reference_disks = reference
        .get("storage")
        .and_then(|storage| storage.get("disks"))
        .and_then(Value::as_array)
        .ok_or_else(|| eyre!("reference storage disks not found"))?;

    let candidate_disks = candidate
        .get_mut("storage")
        .and_then(|storage| storage.get_mut("disks"))
        .and_then(Value::as_array_mut)
        .ok_or_else(|| eyre!("candidate storage disks not found"))?;

    for (ref_disk, candidate_disk) in reference_disks.iter().zip(candidate_disks.iter_mut()) {
        let ref_bytes = ref_disk
            .get("available_bytes")
            .and_then(Value::as_i64)
            .ok_or_else(|| eyre!("missing reference disk available bytes"))?;
        let candidate_bytes = candidate_disk
            .get("available_bytes")
            .and_then(Value::as_i64)
            .ok_or_else(|| eyre!("missing candidate disk available bytes"))?;

        if (ref_bytes - candidate_bytes).abs() > STORAGE_TOLERANCE_BYTES {
            return Err(eyre!(
                "available bytes differ more than tolerance: reference {ref_bytes}, candidate {candidate_bytes}"
            ));
        }

        if let Some(field) = candidate_disk.get_mut("available_bytes") {
            *field = Value::from(ref_bytes);
        }

        if let Some(ref_available_display) = ref_disk.get("available_display")
            && let Some(candidate_available_display) = candidate_disk.get_mut("available_display")
        {
            *candidate_available_display = ref_available_display.clone();
        }

        if let Some(ref_used_display) = ref_disk.get("used_display")
            && let Some(candidate_used_display) = candidate_disk.get_mut("used_display")
        {
            *candidate_used_display = ref_used_display.clone();
        }

        if let Some(ref_total_display) = ref_disk.get("total_display")
            && let Some(candidate_total_display) = candidate_disk.get_mut("total_display")
        {
            *candidate_total_display = ref_total_display.clone();
        }
    }

    align_processor(reference, candidate)?;

    Ok(())
}

fn align_processor(reference: &Value, candidate: &mut Value) -> Result<()> {
    let reference_processor = reference
        .get("processor")
        .ok_or_else(|| eyre!("reference processor section missing"))?;
    let candidate_processor = candidate
        .get_mut("processor")
        .ok_or_else(|| eyre!("candidate processor section missing"))?;

    let reference_frequency = reference_processor
        .get("frequency_ghz")
        .and_then(Value::as_f64)
        .ok_or_else(|| eyre!("reference processor frequency missing"))?;

    if let Some(candidate_frequency_value) = candidate_processor.get_mut("frequency_ghz")
        && let Some(candidate_frequency) = candidate_frequency_value.as_f64()
        && (reference_frequency - candidate_frequency).abs() <= CPU_FREQUENCY_TOLERANCE_GHZ
    {
        *candidate_frequency_value = Value::from(reference_frequency);
    }

    Ok(())
}

#[tokio::test]
async fn parity_get_packages() -> Result<()> {
    let core_value = serde_json::to_value(api::get_packages()?)?;
    let cli_value = cli_json(&["get-packages"])?;
    let web_value = {
        let response = kittynode_web::get_packages()
            .await
            .map_err(|(status, msg)| eyre!("get_packages failed via web: {status} {msg}"))?;
        serde_json::to_value(response.0)?
    };

    assert_eq!(cli_value, core_value);
    assert_eq!(web_value, core_value);
    Ok(())
}

#[tokio::test]
async fn parity_get_config() -> Result<()> {
    let core_value = serde_json::to_value(api::get_config()?)?;
    let cli_value = cli_json(&["get-config"])?;
    let web_value = {
        let response = kittynode_web::get_config()
            .await
            .map_err(|(status, msg)| eyre!("get_config failed via web: {status} {msg}"))?;
        serde_json::to_value(response.0)?
    };

    assert_eq!(cli_value, core_value);
    assert_eq!(web_value, core_value);
    Ok(())
}

#[tokio::test]
async fn parity_system_info() -> Result<()> {
    let core_value = serde_json::to_value(api::get_system_info()?)?;
    let mut cli_value = cli_json(&["system-info"])?;
    let mut web_value = {
        let response = kittynode_web::get_system_info()
            .await
            .map_err(|(status, msg)| eyre!("get_system_info failed via web: {status} {msg}"))?;
        serde_json::to_value(response.0)?
    };

    align_storage_available_bytes(&core_value, &mut cli_value)?;
    align_storage_available_bytes(&core_value, &mut web_value)?;

    assert_eq!(cli_value, core_value);
    assert_eq!(web_value, core_value);
    Ok(())
}

#[tokio::test]
async fn parity_get_installed_packages() -> Result<()> {
    if !api::is_docker_running().await {
        eprintln!("Skipping installed packages parity test: Docker not running");
        return Ok(());
    }

    let core_value = serde_json::to_value(api::get_installed_packages().await?)?;
    let cli_value = cli_json(&["get-installed-packages"])?;
    let web_value = {
        let response = kittynode_web::get_installed_packages()
            .await
            .map_err(|(status, msg)| {
                eyre!("get_installed_packages failed via web: {status} {msg}")
            })?;
        serde_json::to_value(response.0)?
    };

    assert_eq!(cli_value, core_value);
    assert_eq!(web_value, core_value);
    Ok(())
}

#[tokio::test]
async fn parity_get_container_logs() -> Result<()> {
    if !api::is_docker_running().await {
        eprintln!("Skipping container logs parity test: Docker not running");
        return Ok(());
    }

    const CONTAINER: &str = "reth-node";

    let core_logs = match api::get_container_logs(CONTAINER, Some(5)).await {
        Ok(logs) => logs,
        Err(_) => {
            eprintln!("Skipping container logs parity test: container {CONTAINER} unavailable");
            return Ok(());
        }
    };

    let cli_logs: Vec<String> = {
        let value = cli_json(&["get-container-logs", "--tail", "5", CONTAINER])?;
        serde_json::from_value(value)?
    };

    let web_logs: Vec<String> = {
        let response = kittynode_web::get_container_logs(
            Path(CONTAINER.to_string()),
            Query(kittynode_core::api::types::LogsQuery { tail: Some(5) }),
        )
        .await
        .map_err(|(status, msg)| eyre!("get_container_logs failed via web: {status} {msg}"))?;
        response.0
    };

    assert_eq!(cli_logs, core_logs);
    assert_eq!(web_logs, core_logs);
    Ok(())
}
