use crate::output::OutputFormat;
use eyre::{Result, WrapErr};
use kittynode_core::api;
use kittynode_core::api::types::{
    Config, OperationalMode, OperationalState, Package, PackageConfig, SystemInfo,
};
use serde::Serialize;
use std::collections::HashMap;
use std::io::{self, Write};

fn print_json<T: Serialize>(value: &T) -> Result<()> {
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    serde_json::to_writer(&mut handle, value)?;
    handle.write_all(b"\n")?;
    Ok(())
}

pub async fn get_packages(format: OutputFormat) -> Result<()> {
    let packages = api::get_packages()?;
    if format.is_json() {
        print_json(&packages)?;
    } else {
        let mut entries: Vec<(&String, &Package)> = packages.iter().collect();
        entries.sort_by(|(a, _), (b, _)| a.cmp(b));
        for (_name, package) in entries {
            println!("{}", package);
        }
    }
    Ok(())
}

pub async fn get_installed_packages(format: OutputFormat) -> Result<()> {
    let packages = api::get_installed_packages().await?;
    if format.is_json() {
        print_json(&packages)?;
    } else {
        for package in &packages {
            println!("{}", package);
        }
    }
    Ok(())
}

pub async fn install_package(name: String) -> Result<()> {
    api::install_package(&name)
        .await
        .wrap_err_with(|| format!("Failed to install {name}"))?;
    tracing::info!("installed {name}");
    Ok(())
}

pub async fn delete_package(name: String, include_images: bool) -> Result<()> {
    api::delete_package(&name, include_images)
        .await
        .wrap_err_with(|| format!("Failed to delete {name}"))?;
    tracing::info!("deleted {name}");
    Ok(())
}

pub async fn system_info(format: OutputFormat) -> Result<()> {
    let info = api::get_system_info()?;
    if format.is_json() {
        print_json(&info)?;
    } else {
        print_system_info_text(&info);
    }
    Ok(())
}

fn print_system_info_text(info: &SystemInfo) {
    println!(
        "Processor: {} ({} cores, {:.2} GHz)",
        info.processor.name, info.processor.cores, info.processor.frequency_ghz
    );
    println!("Memory: {}", info.memory.total_display);
    println!("Storage:");
    for disk in &info.storage.disks {
        println!("  {} mounted on {}", disk.name, disk.mount_point);
        println!("    Total: {}", disk.total_display);
        println!("    Available: {}", disk.available_display);
    }
}

pub async fn get_container_logs(
    format: OutputFormat,
    container: String,
    tail: Option<usize>,
) -> Result<()> {
    let logs = api::get_container_logs(&container, tail).await?;
    if format.is_json() {
        print_json(&logs)?;
    } else {
        for line in logs {
            println!("{line}");
        }
    }
    Ok(())
}

pub fn get_config(format: OutputFormat) -> Result<()> {
    let config = api::get_config()?;
    if format.is_json() {
        print_json(&config)?;
    } else {
        print_config_text(&config);
    }
    Ok(())
}

fn print_config_text(config: &Config) {
    println!(
        "Server URL: {}",
        if config.server_url.is_empty() {
            "(local)"
        } else {
            &config.server_url
        }
    );
    println!("Capabilities:");
    for capability in &config.capabilities {
        println!("  - {capability}");
    }
    println!(
        "Onboarding completed: {}",
        if config.onboarding_completed {
            "yes"
        } else {
            "no"
        }
    );
    println!(
        "Auto start Docker: {}",
        if config.auto_start_docker {
            "enabled"
        } else {
            "disabled"
        }
    );
}

pub async fn get_package_config(format: OutputFormat, name: String) -> Result<()> {
    let config = api::get_package_config(&name).await?;
    if format.is_json() {
        print_json(&config)?;
    } else if config.values.is_empty() {
        println!("No overrides set for {name}");
    } else {
        println!("Overrides for {name}:");
        for (key, value) in &config.values {
            println!("  {key}={value}");
        }
    }
    Ok(())
}

pub async fn update_package_config(name: String, values: Vec<(String, String)>) -> Result<()> {
    let mut map = HashMap::new();
    for (key, value) in values {
        map.insert(key, value);
    }
    let config = PackageConfig { values: map };
    api::update_package_config(&name, config)
        .await
        .wrap_err_with(|| format!("Failed to update config for {name}"))?;
    tracing::info!("updated config for {name}");
    Ok(())
}

pub fn get_capabilities(format: OutputFormat) -> Result<()> {
    let capabilities = api::get_capabilities()?;
    if format.is_json() {
        print_json(&capabilities)?;
    } else if capabilities.is_empty() {
        println!("No capabilities configured");
    } else {
        for capability in &capabilities {
            println!("{capability}");
        }
    }
    Ok(())
}

pub fn add_capability(name: String) -> Result<()> {
    api::add_capability(&name).wrap_err_with(|| format!("Failed to add capability {name}"))?;
    tracing::info!("added capability {name}");
    Ok(())
}

pub fn remove_capability(name: String) -> Result<()> {
    api::remove_capability(&name)
        .wrap_err_with(|| format!("Failed to remove capability {name}"))?;
    tracing::info!("removed capability {name}");
    Ok(())
}

pub fn init_kittynode() -> Result<()> {
    api::init_kittynode().wrap_err("Failed to initialize Kittynode")?;
    tracing::info!("initialized kittynode");
    Ok(())
}

pub fn delete_kittynode() -> Result<()> {
    api::delete_kittynode().wrap_err("Failed to delete Kittynode data")?;
    tracing::info!("deleted kittynode data");
    Ok(())
}

pub async fn is_docker_running(format: OutputFormat) -> Result<()> {
    let running = api::is_docker_running().await;
    if format.is_json() {
        print_json(&running)?;
    } else {
        println!(
            "{}",
            if running {
                "Docker is running"
            } else {
                "Docker is not running"
            }
        );
    }
    Ok(())
}

pub async fn start_docker_if_needed(format: OutputFormat) -> Result<()> {
    let status = api::start_docker_if_needed().await?;
    if format.is_json() {
        print_json(&status)?;
    } else {
        println!("{}", status.as_str());
    }
    Ok(())
}

pub async fn get_operational_state(format: OutputFormat) -> Result<()> {
    let state = api::get_operational_state().await?;
    if format.is_json() {
        print_json(&state)?;
    } else {
        print_operational_state_text(&state);
    }
    Ok(())
}

fn print_operational_state_text(state: &OperationalState) {
    let mode = match state.mode {
        OperationalMode::Local => "local",
        OperationalMode::Remote => "remote",
    };
    println!("Mode: {mode}");
    println!(
        "Docker running: {}",
        if state.docker_running { "yes" } else { "no" }
    );
    println!(
        "Can install: {}",
        if state.can_install { "yes" } else { "no" }
    );
    println!(
        "Can manage: {}",
        if state.can_manage { "yes" } else { "no" }
    );
    if !state.diagnostics.is_empty() {
        println!("Diagnostics:");
        for entry in &state.diagnostics {
            println!("  - {entry}");
        }
    }
}
