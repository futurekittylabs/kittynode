pub mod validator;

use eyre::{Result, WrapErr, eyre};
use kittynode_core::api::types::{
    Config, OperationalMode, OperationalState, Package, PackageConfig, SystemInfo, WebServiceStatus,
};
use kittynode_core::api::{self, DEFAULT_WEB_PORT, validate_web_port};
use std::collections::{HashMap, VecDeque};
use std::env;
use std::fs::OpenOptions;
use std::io::{BufRead, BufReader, ErrorKind, Seek, SeekFrom, Write};
use std::path::Path;
use std::process::Command;

macro_rules! writeln_string {
    ($dst:expr, $($arg:tt)*) => {{
        use std::fmt::Write as _;
        writeln!($dst, $($arg)*).expect("writing to string cannot fail")
    }};
}

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

pub async fn install_package(name: String, network: Option<&str>) -> Result<()> {
    if name == "ethereum" && network.is_none() {
        return Err(eyre!(
            "Network must be provided when installing ethereum. Use --network <hoodi|mainnet|sepolia|ephemery>"
        ));
    }
    api::install_package_with_network(&name, network)
        .await
        .wrap_err_with(|| format!("Failed to install {name}"))?;
    tracing::info!("installed {name}");
    Ok(())
}

pub async fn delete_package(name: String, include_images: bool) -> Result<()> {
    let packages = api::get_installed_packages()
        .await
        .wrap_err("Failed to list installed packages")?;
    let Some(pkg) = packages.iter().find(|pkg| pkg.name() == name) else {
        println!("Package {name} is not installed");
        return Ok(());
    };
    let resolved_name = pkg.name();

    api::delete_package(resolved_name, include_images)
        .await
        .wrap_err_with(|| format!("Failed to delete {resolved_name}"))?;
    tracing::info!("deleted {resolved_name}");
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
    print!("{}", render_system_info(info));
}

fn render_system_info(info: &SystemInfo) -> String {
    let mut output = String::new();
    writeln_string!(
        &mut output,
        "Processor: {} ({} cores, {:.2} GHz)",
        info.processor.name,
        info.processor.cores,
        info.processor.frequency_ghz
    );
    writeln_string!(&mut output, "Memory: {}", info.memory.total_display);
    writeln_string!(&mut output, "Storage:");
    for disk in &info.storage.disks {
        writeln_string!(
            &mut output,
            "  {} mounted on {}",
            disk.name,
            disk.mount_point
        );
        writeln_string!(&mut output, "    Total: {}", disk.total_display);
        writeln_string!(&mut output, "    Available: {}", disk.available_display);
    }
    output
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
    print!("{}", render_config(config));
}

fn render_config(config: &Config) -> String {
    let mut output = String::new();
    let server = if config.server_url.is_empty() {
        "(local)"
    } else {
        config.server_url.as_str()
    };
    writeln_string!(&mut output, "Server URL: {server}");
    writeln_string!(&mut output, "Capabilities:");
    for capability in &config.capabilities {
        writeln_string!(&mut output, "  - {capability}");
    }
    let onboarding = if config.onboarding_completed {
        "yes"
    } else {
        "no"
    };
    writeln_string!(&mut output, "Onboarding completed: {onboarding}");
    let auto_start = if config.auto_start_docker {
        "enabled"
    } else {
        "disabled"
    };
    writeln_string!(&mut output, "Auto start Docker: {auto_start}");
    output
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
    print!("{}", render_operational_state(state));
}

fn render_operational_state(state: &OperationalState) -> String {
    let mut output = String::new();
    let mode = match state.mode {
        OperationalMode::Local => "local",
        OperationalMode::Remote => "remote",
    };
    writeln_string!(&mut output, "Mode: {mode}");
    let docker_running = if state.docker_running { "yes" } else { "no" };
    writeln_string!(&mut output, "Docker running: {docker_running}");
    let can_install = if state.can_install { "yes" } else { "no" };
    writeln_string!(&mut output, "Can install: {can_install}");
    let can_manage = if state.can_manage { "yes" } else { "no" };
    writeln_string!(&mut output, "Can manage: {can_manage}");
    if !state.diagnostics.is_empty() {
        writeln_string!(&mut output, "Diagnostics:");
        for entry in &state.diagnostics {
            writeln_string!(&mut output, "  - {entry}");
        }
    }
    output
}

pub fn start_web_service(port: Option<u16>) -> Result<()> {
    let binary = env::current_exe().wrap_err("Failed to locate kittynode binary")?;
    let port = port.map(validate_web_port).transpose()?;
    let status = api::start_web_service(port, &binary, &["web", WEB_INTERNAL_SUBCOMMAND])?;
    println!("{}", status);
    if let Ok(path) = api::get_web_service_log_path() {
        println!("Logs: {}", path.display());
    }
    Ok(())
}

pub fn stop_web_service() -> Result<()> {
    let status = api::stop_web_service()?;
    println!("{}", status);
    Ok(())
}

pub fn restart_web_service(port: Option<u16>) -> Result<()> {
    let port = match port {
        Some(port) => Some(port),
        None => match api::get_web_service_status()? {
            WebServiceStatus::Started { port, .. }
            | WebServiceStatus::AlreadyRunning { port, .. } => Some(port),
            _ => None,
        },
    };

    let status = api::stop_web_service()?;
    println!("{}", status);

    start_web_service(port)
}

pub fn web_status() -> Result<()> {
    match api::get_web_service_status()? {
        WebServiceStatus::Started { pid, port }
        | WebServiceStatus::AlreadyRunning { pid, port } => {
            println!("Kittynode web service running on port {port} (pid {pid})");
            if let Ok(path) = api::get_web_service_log_path() {
                println!("Logs: {}", path.display());
            }
        }
        WebServiceStatus::Stopped { pid, port } => {
            println!("Kittynode web service stopped (last seen pid {pid}, port {port})");
        }
        WebServiceStatus::NotRunning => {
            println!("Kittynode web service is not running");
        }
    }
    Ok(())
}

pub fn web_logs(follow: bool, tail: Option<usize>) -> Result<()> {
    let tail = tail.filter(|value| *value > 0);
    let path =
        api::get_web_service_log_path().wrap_err("Failed to locate kittynode web service logs")?;
    stream_log_file(&path, tail, follow)
        .wrap_err_with(|| format!("Failed to stream logs from {}", path.display()))?;
    Ok(())
}

fn stream_log_file(path: &Path, tail: Option<usize>, follow: bool) -> Result<()> {
    let stdout = std::io::stdout();
    let mut handle = stdout.lock();
    stream_log_file_with_writer(path, tail, follow, &mut handle)
}

fn stream_log_file_with_writer(
    path: &Path,
    tail: Option<usize>,
    follow: bool,
    writer: &mut dyn Write,
) -> Result<()> {
    let mut file = OpenOptions::new()
        .read(true)
        .open(path)
        .map_err(|err| eyre!("Failed to open log file: {err}"))?;

    let snapshot = collect_initial_log_output(
        BufReader::new(
            file.try_clone()
                .map_err(|err| eyre!("Failed to clone log file handle: {err}"))?,
        ),
        tail,
    )?;

    writer
        .write_all(snapshot.as_bytes())
        .map_err(|err| eyre!("Failed to write log output: {err}"))?;

    writer
        .flush()
        .map_err(|err| eyre!("Failed to flush stdout: {err}"))?;

    if !follow {
        return Ok(());
    }

    file.seek(SeekFrom::End(0))
        .map_err(|err| eyre!("Failed to seek log file: {err}"))?;
    let mut reader = BufReader::new(file);
    let mut line = String::new();
    loop {
        match reader.read_line(&mut line) {
            Ok(0) => {
                line.clear();
                std::thread::sleep(std::time::Duration::from_millis(250));
            }
            Ok(_) => {
                writer
                    .write_all(line.as_bytes())
                    .map_err(|err| eyre!("Failed to write log output: {err}"))?;
                writer
                    .flush()
                    .map_err(|err| eyre!("Failed to flush stdout: {err}"))?;
                line.clear();
            }
            Err(err) if err.kind() == ErrorKind::Interrupted => continue,
            Err(err) => return Err(eyre!("Failed while streaming logs: {err}")),
        }
    }
}

fn collect_initial_log_output<R: BufRead>(mut reader: R, tail: Option<usize>) -> Result<String> {
    if let Some(limit) = tail {
        read_tail_lines(&mut reader, limit)
    } else {
        let mut content = String::new();
        reader
            .read_to_string(&mut content)
            .map_err(|err| eyre!("Failed to read kittynode web log file: {err}"))?;
        Ok(content)
    }
}

fn read_tail_lines<R: BufRead>(reader: &mut R, limit: usize) -> Result<String> {
    let mut buffer = VecDeque::with_capacity(limit);
    let mut line = String::new();
    loop {
        line.clear();
        let bytes = match reader.read_line(&mut line) {
            Ok(count) => count,
            Err(err) if err.kind() == ErrorKind::Interrupted => continue,
            Err(err) => {
                return Err(eyre!("Failed to read kittynode web log file: {err}"));
            }
        };
        if bytes == 0 {
            break;
        }
        if buffer.len() == limit {
            buffer.pop_front();
        }
        buffer.push_back(line.clone());
    }

    let mut collected = String::new();
    for entry in buffer {
        collected.push_str(&entry);
    }
    Ok(collected)
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

/// Launch the standalone updater installed by cargo-dist installers.
/// This expects a `kittynode-cli-update` binary to be on PATH.
pub fn run_updater() -> Result<()> {
    match Command::new("kittynode-cli-update").status() {
        Ok(status) => {
            if status.success() {
                Ok(())
            } else {
                Err(eyre!(format!(
                    "updater exited with code {code}",
                    code = status.code().unwrap_or(-1)
                )))
            }
        }
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Err(eyre!(
            "could not find 'kittynode-cli-update' in PATH; reinstall via the installer or ensure the updater is installed"
        )),
        Err(err) => Err(eyre!(format!("failed to launch updater: {err}"))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::io::Cursor;
    use tempfile::NamedTempFile;

    #[test]
    fn collect_initial_log_output_without_tail_returns_full_content() {
        let content = "first line\nsecond line\n";
        let result = collect_initial_log_output(Cursor::new(content.as_bytes()), None)
            .expect("reading snapshot without tail succeeds");

        assert_eq!(result, content);
    }

    #[test]
    fn collect_initial_log_output_with_tail_limits_to_requested_lines() {
        let content = "line1\nline2\nline3\n";
        let result = collect_initial_log_output(Cursor::new(content.as_bytes()), Some(2))
            .expect("reading snapshot with tail succeeds");

        assert_eq!(result, "line2\nline3\n");
    }

    #[test]
    fn render_config_formats_remote_server_with_capabilities() {
        let config = Config {
            capabilities: vec!["ethereum".into(), "solana".into()],
            server_url: "https://rpc.example".into(),
            onboarding_completed: true,
            auto_start_docker: false,
            ..Default::default()
        };

        let rendered = render_config(&config);
        let expected = "Server URL: https://rpc.example\nCapabilities:\n  - ethereum\n  - solana\nOnboarding completed: yes\nAuto start Docker: disabled\n";
        assert_eq!(rendered, expected);
    }

    #[test]
    fn render_operational_state_includes_diagnostics() {
        let state = OperationalState {
            mode: OperationalMode::Remote,
            docker_running: true,
            can_install: false,
            can_manage: true,
            diagnostics: vec!["restart docker".into(), "check firewall".into()],
        };

        let rendered = render_operational_state(&state);
        let expected = "Mode: remote\nDocker running: yes\nCan install: no\nCan manage: yes\nDiagnostics:\n  - restart docker\n  - check firewall\n";
        assert_eq!(rendered, expected);
    }

    #[test]
    fn render_system_info_lists_disks() {
        let info: SystemInfo = serde_json::from_value(json!({
            "processor": {
                "name": "Test CPU",
                "cores": 8,
                "frequencyGhz": 3.5,
                "architecture": "x86_64"
            },
            "memory": {
                "totalBytes": 34359738368u64,
                "totalDisplay": "32 GB"
            },
            "storage": {
                "disks": [
                    {
                        "name": "disk1",
                        "mountPoint": "/",
                        "totalBytes": 512000000000u64,
                        "availableBytes": 256000000000u64,
                        "totalDisplay": "512 GB",
                        "usedDisplay": "256 GB",
                        "availableDisplay": "256 GB",
                        "diskType": "apfs"
                    },
                    {
                        "name": "disk2",
                        "mountPoint": "/data",
                        "totalBytes": 1000000000000u64,
                        "availableBytes": 750000000000u64,
                        "totalDisplay": "1.00 TB",
                        "usedDisplay": "250.00 GB",
                        "availableDisplay": "750.00 GB",
                        "diskType": "ext4"
                    }
                ]
            }
        }))
        .expect("json literal is valid system info");

        let rendered = render_system_info(&info);
        let expected = "Processor: Test CPU (8 cores, 3.50 GHz)\nMemory: 32 GB\nStorage:\n  disk1 mounted on /\n    Total: 512 GB\n    Available: 256 GB\n  disk2 mounted on /data\n    Total: 1.00 TB\n    Available: 750.00 GB\n";
        assert_eq!(rendered, expected);
    }

    #[test]
    fn render_config_for_local_server_shows_placeholder() {
        let config = Config {
            server_url: String::new(),
            capabilities: vec![],
            onboarding_completed: false,
            auto_start_docker: true,
            ..Default::default()
        };

        let rendered = render_config(&config);
        let expected = "Server URL: (local)\nCapabilities:\nOnboarding completed: no\nAuto start Docker: enabled\n";
        assert_eq!(rendered, expected);
    }

    #[test]
    fn render_operational_state_without_diagnostics_omits_section() {
        let state = OperationalState {
            mode: OperationalMode::Local,
            docker_running: false,
            can_install: true,
            can_manage: false,
            diagnostics: vec![],
        };

        let rendered = render_operational_state(&state);
        assert!(
            !rendered.contains("Diagnostics"),
            "expected diagnostics section to be omitted, got {rendered}",
            rendered = rendered
        );
        let expected_prefix = "Mode: local\nDocker running: no\nCan install: yes\nCan manage: no\n";
        assert!(
            rendered.starts_with(expected_prefix),
            "expected output to start with {expected_prefix}, got {rendered}",
            expected_prefix = expected_prefix,
            rendered = rendered
        );
    }

    #[test]
    fn render_system_info_without_disks_still_lists_storage_section() {
        let info: SystemInfo = serde_json::from_value(json!({
            "processor": {
                "name": "Test CPU",
                "cores": 4,
                "frequencyGhz": 2.25,
                "architecture": "x86_64"
            },
            "memory": {
                "totalBytes": 17179869184u64,
                "totalDisplay": "16 GB"
            },
            "storage": {
                "disks": []
            }
        }))
        .expect("json literal is valid system info");

        let rendered = render_system_info(&info);
        let expected = "Processor: Test CPU (4 cores, 2.25 GHz)\nMemory: 16 GB\nStorage:\n";
        assert!(
            rendered.starts_with(expected),
            "expected output to start with {expected}, got {rendered}",
            expected = expected,
            rendered = rendered
        );
        assert_eq!(rendered.lines().count(), 3);
    }

    #[test]
    fn stream_log_file_without_tail_writes_entire_contents() {
        let mut temp = NamedTempFile::new().expect("failed to create temp log file");
        writeln!(temp, "first line").expect("failed to write first line");
        writeln!(temp, "second line").expect("failed to write second line");
        temp.flush().expect("failed to flush log file");

        let mut buffer = Vec::new();
        stream_log_file_with_writer(temp.path(), None, false, &mut buffer)
            .expect("streaming log file without tail should succeed");

        let output = String::from_utf8(buffer).expect("log output should be utf8");
        assert_eq!(output, "first line\nsecond line\n");
    }

    #[test]
    fn stream_log_file_with_tail_limits_output() {
        let mut temp = NamedTempFile::new().expect("failed to create temp log file");
        writeln!(temp, "line 1").expect("failed to write line 1");
        writeln!(temp, "line 2").expect("failed to write line 2");
        writeln!(temp, "line 3").expect("failed to write line 3");
        temp.flush().expect("failed to flush log file");

        let mut buffer = Vec::new();
        stream_log_file_with_writer(temp.path(), Some(2), false, &mut buffer)
            .expect("streaming log file with tail should succeed");

        let output = String::from_utf8(buffer).expect("log output should be utf8");
        assert_eq!(output, "line 2\nline 3\n");
    }

    #[test]
    fn stream_log_file_tail_longer_than_log_outputs_all_lines() {
        let mut temp = NamedTempFile::new().expect("failed to create temp log file");
        writeln!(temp, "alpha").expect("failed to write alpha");
        writeln!(temp, "beta").expect("failed to write beta");
        temp.flush().expect("failed to flush log file");

        let mut buffer = Vec::new();
        stream_log_file_with_writer(temp.path(), Some(5), false, &mut buffer)
            .expect("streaming log file with large tail should succeed");

        let output = String::from_utf8(buffer).expect("log output should be utf8");
        assert_eq!(output, "alpha\nbeta\n");
    }
}
