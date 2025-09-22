use eyre::{Result, WrapErr};
use kittynode_core::api::types::{
    Config, OperationalMode, OperationalState, Package, PackageConfig, SystemInfo,
};
use kittynode_core::api::{
    self, CreateDepositDataParams, DEFAULT_WEB_PORT, GenerateKeysParams, validate_web_port,
};
use std::collections::HashMap;
use std::env;
use std::path::PathBuf;

pub async fn get_packages() -> Result<()> {
    let packages = api::get_packages()?;
    let mut entries: Vec<(&String, &Package)> = packages.iter().collect();
    entries.sort_by(|(a, _), (b, _)| a.cmp(b));
    for (_name, package) in entries {
        println!("{}", package);
    }
    Ok(())
}

pub async fn get_installed_packages() -> Result<()> {
    let packages = api::get_installed_packages().await?;
    let names: Vec<String> = packages.iter().map(|pkg| pkg.name().to_string()).collect();
    let runtime_states = match api::get_packages_runtime_state(&names).await {
        Ok(map) => map,
        Err(error) => {
            tracing::warn!(%error, "failed to retrieve runtime state information");
            HashMap::new()
        }
    };

    if packages.is_empty() {
        println!("No packages are currently installed");
        return Ok(());
    }

    for package in &packages {
        let state = runtime_states
            .get(package.name())
            .map(|runtime| {
                if runtime.running {
                    "running"
                } else {
                    "stopped"
                }
            })
            .unwrap_or("unknown");

        println!("{} [status: {state}]", package.name());
        println!("  {}", package.description());
        println!("  Network: {}", package.network_name());
        println!();
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

pub async fn stop_package(name: String) -> Result<()> {
    api::stop_package(&name)
        .await
        .wrap_err_with(|| format!("Failed to stop {name}"))?;
    tracing::info!("stopped {name}");
    Ok(())
}

pub async fn resume_package(name: String) -> Result<()> {
    api::resume_package(&name)
        .await
        .wrap_err_with(|| format!("Failed to resume {name}"))?;
    tracing::info!("resumed {name}");
    Ok(())
}

pub async fn system_info() -> Result<()> {
    let info = api::get_system_info()?;
    print_system_info_text(&info);
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

pub async fn get_container_logs(container: String, tail: Option<usize>) -> Result<()> {
    let logs = api::get_container_logs(&container, tail).await?;
    for line in logs {
        println!("{line}");
    }
    Ok(())
}

pub fn get_config() -> Result<()> {
    let config = api::get_config()?;
    print_config_text(&config);
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

pub async fn get_package_config(name: String) -> Result<()> {
    let config = api::get_package_config(&name).await?;
    if config.values.is_empty() {
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

pub fn get_capabilities() -> Result<()> {
    let capabilities = api::get_capabilities()?;
    if capabilities.is_empty() {
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

pub async fn is_docker_running() -> Result<()> {
    let running = api::is_docker_running().await;
    println!(
        "{}",
        if running {
            "Docker is running"
        } else {
            "Docker is not running"
        }
    );
    Ok(())
}

pub async fn start_docker_if_needed() -> Result<()> {
    let status = api::start_docker_if_needed().await?;
    println!("{}", status.as_str());
    Ok(())
}

pub async fn get_operational_state() -> Result<()> {
    let state = api::get_operational_state().await?;
    print_operational_state_text(&state);
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

pub fn validator_generate_keys(
    output_dir: PathBuf,
    file_name: Option<String>,
    entropy: String,
    overwrite: bool,
) -> Result<()> {
    let params = GenerateKeysParams {
        output_dir,
        file_name,
        entropy,
        overwrite,
    };
    let key_path = params.key_path();
    let key = api::generate_keys(params)?;
    println!("Stored validator key at {}", key_path.display());
    println!("Public key: {}", key.public_key);
    Ok(())
}

pub fn validator_create_deposit_data(
    key_path: PathBuf,
    output_path: PathBuf,
    withdrawal_credentials: String,
    amount_gwei: u64,
    fork_version_hex: String,
    genesis_root_hex: String,
    overwrite: bool,
) -> Result<()> {
    let params = CreateDepositDataParams::from_hex_inputs(
        key_path,
        output_path.clone(),
        withdrawal_credentials,
        amount_gwei,
        &fork_version_hex,
        &genesis_root_hex,
        overwrite,
    )?;

    let deposit = api::create_deposit_data(params)?;
    println!("Stored deposit data at {}", output_path.display());
    println!("Deposit data root: {}", deposit.deposit_data_root);
    Ok(())
}

pub fn start_web_service(port: Option<u16>) -> Result<()> {
    let binary = env::current_exe().wrap_err("Failed to locate kittynode binary")?;
    let port = port.map(validate_web_port).transpose()?;
    let status = api::start_web_service(port, &binary, &["web", WEB_INTERNAL_SUBCOMMAND])?;
    println!("{}", status);
    Ok(())
}

pub fn stop_web_service() -> Result<()> {
    let status = api::stop_web_service()?;
    println!("{}", status);
    Ok(())
}

pub async fn run_web_service(port: Option<u16>, service_token: Option<String>) -> Result<()> {
    let port = validate_web_port(port.unwrap_or(DEFAULT_WEB_PORT))?;
    let Some(_token) = service_token else {
        return Err(eyre::eyre!("web service run invoked without token"));
    };
    kittynode_web::run_with_port(port).await?;
    Ok(())
}

pub const WEB_INTERNAL_SUBCOMMAND: &str = "__internal-run";
