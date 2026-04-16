use crate::paths::kittynode_path;
use eyre::{Result, WrapErr, eyre};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::ffi::{OsStr, OsString};
use std::fmt;
use std::fs::{self, OpenOptions};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::Duration;
use sysinfo::{Pid, Process, System};
use tracing::info;

pub const DEFAULT_SERVER_PORT: u16 = 3000;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ServerStatus {
    Started { pid: u32, port: u16 },
    AlreadyRunning { pid: u32, port: u16 },
    Stopped { pid: u32, port: u16 },
    NotRunning,
}

impl fmt::Display for ServerStatus {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ServerStatus::Started { pid, port } => {
                write!(
                    formatter,
                    "Kittynode server started on port {port} (pid {pid})"
                )
            }
            ServerStatus::AlreadyRunning { pid, port } => {
                write!(
                    formatter,
                    "Kittynode server already running on port {port} (pid {pid})"
                )
            }
            ServerStatus::Stopped { pid, port } => {
                write!(
                    formatter,
                    "Kittynode server stopped on port {port} (pid {pid})"
                )
            }
            ServerStatus::NotRunning => write!(formatter, "Kittynode server is not running"),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct ServerProcessState {
    pid: u32,
    port: u16,
    binary: PathBuf,
    #[serde(default)]
    token: Option<String>,
    #[serde(default)]
    log_path: Option<PathBuf>,
}

const STATE_FILE_NAME: &str = "kittynode-server.json";
const RUNTIME_DIR: &str = "runtime";
const LOG_FILE_NAME: &str = "kittynode-server.log";

pub fn start_server(port: Option<u16>, binary_path: &Path, args: &[&str]) -> Result<ServerStatus> {
    let port = validate_server_port(port.unwrap_or(DEFAULT_SERVER_PORT))?;

    if let Some(mut state) = load_state()? {
        if process_matches(&state) {
            if state.log_path.is_none() {
                state.log_path = Some(log_file_path()?);
                let _ = save_state(&state);
            }

            return Ok(ServerStatus::AlreadyRunning {
                pid: state.pid,
                port: state.port,
            });
        }

        clear_state()?;
    }

    let binary_path = binary_path
        .canonicalize()
        .unwrap_or_else(|_| binary_path.to_path_buf());
    let token = generate_service_token();
    let log_path = log_file_path()?;

    if let Some(parent) = log_path.parent() {
        fs::create_dir_all(parent)
            .wrap_err("Failed to create directories for kittynode-server log file")?;
    }

    let log_file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&log_path)
        .wrap_err("Failed to open kittynode-server log file")?;
    let stdout = log_file
        .try_clone()
        .wrap_err("Failed to duplicate log handle for stdout")?;

    let mut command = Command::new(&binary_path);
    command
        .stdin(Stdio::null())
        .stdout(Stdio::from(stdout))
        .stderr(Stdio::from(log_file))
        .args(args)
        .arg("--port")
        .arg(port.to_string())
        .arg("--service-token")
        .arg(&token);

    let mut child = command
        .spawn()
        .wrap_err("Failed to spawn kittynode-server process")?;

    wait_for_service_ready(&mut child, port)?;

    let pid = child.id();
    let state = ServerProcessState {
        pid,
        port,
        binary: binary_path.clone(),
        token: Some(token),
        log_path: Some(log_path.clone()),
    };

    if let Err(error) = save_state(&state) {
        let kill_result = child.kill();
        if kill_result.is_ok() {
            let _ = child.wait();
        }

        let error = match kill_result {
            Ok(_) => error.wrap_err(format!(
                "Failed to persist kittynode-server state for pid {} on port {} and terminated spawned process",
                pid, port
            )),
            Err(kill_error) => error.wrap_err(format!(
                "Failed to persist kittynode-server state for pid {} on port {} and could not terminate spawned process: {kill_error}",
                pid, port
            )),
        };

        return Err(error);
    }

    drop(child);

    info!(
        pid,
        port = state.port,
        binary = %binary_path.display(),
        "Started kittynode-server service"
    );

    Ok(ServerStatus::Started {
        pid: state.pid,
        port: state.port,
    })
}

pub fn stop_server() -> Result<ServerStatus> {
    let Some(state) = load_state()? else {
        return Ok(ServerStatus::NotRunning);
    };

    if !process_matches(&state) {
        clear_state()?;
        return Ok(ServerStatus::NotRunning);
    }

    let system = System::new_all();
    let pid = Pid::from_u32(state.pid);

    let Some(process) = system.process(pid) else {
        clear_state()?;
        return Ok(ServerStatus::NotRunning);
    };

    if !process.kill() {
        return Err(eyre!(
            "Failed to stop kittynode-server process with pid {}",
            state.pid
        ));
    }

    clear_state()?;
    info!(
        pid = state.pid,
        port = state.port,
        "Stopped kittynode-server service"
    );

    Ok(ServerStatus::Stopped {
        pid: state.pid,
        port: state.port,
    })
}

pub fn get_server_status() -> Result<ServerStatus> {
    let Some(mut state) = load_state()? else {
        return Ok(ServerStatus::NotRunning);
    };

    if process_matches(&state) {
        if state.log_path.is_none() {
            state.log_path = Some(log_file_path()?);
            let _ = save_state(&state);
        }

        return Ok(ServerStatus::AlreadyRunning {
            pid: state.pid,
            port: state.port,
        });
    }

    clear_state()?;
    Ok(ServerStatus::NotRunning)
}

pub fn get_server_log_path() -> Result<PathBuf> {
    let path = log_file_path()?;
    if path.exists() {
        return Ok(path);
    }

    if let Some(state) = load_state()?
        && process_matches(&state)
    {
        return Err(eyre!(
            "Kittynode server logs are not available yet; restart the service to enable logging"
        ));
    }

    Err(eyre!(
        "Kittynode server is not running; start it with `kittynode server start`"
    ))
}

pub fn validate_server_port(port: u16) -> Result<u16> {
    if port == 0 {
        return Err(eyre!("Port must be greater than zero"));
    }

    Ok(port)
}

fn load_state() -> Result<Option<ServerProcessState>> {
    let path = state_file_path()?;
    if !path.exists() {
        return Ok(None);
    }

    let raw = fs::read_to_string(&path).wrap_err("Failed to read kittynode-server state file")?;
    let state =
        serde_json::from_str(&raw).wrap_err("Failed to deserialize kittynode-server state")?;
    Ok(Some(state))
}

fn save_state(state: &ServerProcessState) -> Result<()> {
    let path = state_file_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .wrap_err("Failed to create directories for kittynode-server state file")?;
    }

    let raw =
        serde_json::to_string(state).wrap_err("Failed to serialize kittynode-server state")?;
    fs::write(&path, raw).wrap_err("Failed to write kittynode-server state file")?;
    Ok(())
}

fn clear_state() -> Result<()> {
    let path = state_file_path()?;
    if path.exists() {
        fs::remove_file(path).wrap_err("Failed to remove kittynode-server state file")?;
    }
    Ok(())
}

fn state_file_path() -> Result<PathBuf> {
    Ok(kittynode_path()?.join(RUNTIME_DIR).join(STATE_FILE_NAME))
}

fn log_file_path() -> Result<PathBuf> {
    Ok(kittynode_path()?.join(RUNTIME_DIR).join(LOG_FILE_NAME))
}

fn process_matches(state: &ServerProcessState) -> bool {
    let system = System::new_all();
    let pid = Pid::from_u32(state.pid);
    if let Some(process) = system.process(pid)
        && let Some(exe_path) = process.exe()
    {
        return paths_match(exe_path, &state.binary)
            && cmd_contains_token(process, state.token.as_deref());
    }

    false
}

fn paths_match(left: &Path, right: &Path) -> bool {
    if left == right {
        return true;
    }

    match (left.canonicalize(), right.canonicalize()) {
        (Ok(left_canonical), Ok(right_canonical)) => left_canonical == right_canonical,
        _ => false,
    }
}

fn cmd_contains_token(process: &Process, token: Option<&str>) -> bool {
    args_contain_token(process.cmd(), token)
}

fn args_contain_token(command: &[OsString], token: Option<&str>) -> bool {
    let Some(token) = token else {
        return false;
    };

    let service_flag = OsStr::new("--service-token");
    let token_os = OsStr::new(token);
    let inline_flag = OsString::from(format!("--service-token={token}"));

    for window in command.windows(2) {
        if window[0] == service_flag && window[1] == token_os {
            return true;
        }
    }

    command.iter().any(|arg| arg == &inline_flag)
}

fn generate_service_token() -> String {
    let mut bytes = [0u8; 16];
    rand::rng().fill_bytes(&mut bytes);
    hex::encode(bytes)
}

fn wait_for_service_ready(child: &mut Child, port: u16) -> Result<()> {
    const MAX_ATTEMPTS: u32 = 50;
    const RETRY_DELAY: Duration = Duration::from_millis(100);

    let targets = [
        SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port),
        SocketAddr::new(IpAddr::V6(Ipv6Addr::LOCALHOST), port),
    ];

    for _ in 0..MAX_ATTEMPTS {
        if let Some(status) = child
            .try_wait()
            .wrap_err("Failed to poll kittynode-server process state")?
        {
            let detail = status
                .code()
                .map(|code| format!("exit code {code}"))
                .unwrap_or_else(|| "terminated by signal".to_string());
            return Err(eyre!(
                "kittynode-server process exited immediately ({detail}); check logs for details"
            ));
        }

        if targets
            .iter()
            .any(|addr| TcpStream::connect_timeout(addr, Duration::from_millis(50)).is_ok())
        {
            return Ok(());
        }

        thread::sleep(RETRY_DELAY);
    }

    let _ = child.kill();
    let _ = child.wait();
    Err(eyre!(
        "Timed out waiting for kittynode-server to bind on port {}",
        port
    ))
}

#[cfg(test)]
mod tests {
    use super::{
        DEFAULT_SERVER_PORT, ServerStatus, args_contain_token, generate_service_token, paths_match,
        validate_server_port,
    };
    use std::ffi::OsString;
    use std::fs::{self, File};
    use tempfile::tempdir;

    #[test]
    fn default_server_port_is_stable() {
        assert_eq!(DEFAULT_SERVER_PORT, 3000);
    }

    #[test]
    fn display_messages_are_human_readable() {
        let started = ServerStatus::Started { pid: 1, port: 3000 };
        assert_eq!(
            started.to_string(),
            "Kittynode server started on port 3000 (pid 1)"
        );

        let running = ServerStatus::AlreadyRunning { pid: 2, port: 8080 };
        assert_eq!(
            running.to_string(),
            "Kittynode server already running on port 8080 (pid 2)"
        );

        let stopped = ServerStatus::Stopped { pid: 3, port: 9000 };
        assert_eq!(
            stopped.to_string(),
            "Kittynode server stopped on port 9000 (pid 3)"
        );

        assert_eq!(
            ServerStatus::NotRunning.to_string(),
            "Kittynode server is not running"
        );
    }

    #[test]
    fn validate_server_port_rejects_zero() {
        assert!(validate_server_port(0).is_err());
        assert_eq!(validate_server_port(8080).unwrap(), 8080);
    }

    #[test]
    fn generate_service_token_emits_hex_string() {
        let token = generate_service_token();
        assert_eq!(token.len(), 32);
        assert!(token.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn generate_service_token_produces_unique_tokens() {
        let token1 = generate_service_token();
        let token2 = generate_service_token();
        assert_ne!(token1, token2);
    }

    #[test]
    fn args_contain_token_detects_split_arguments() {
        let args = vec![
            OsString::from("--flag"),
            OsString::from("--service-token"),
            OsString::from("abc123"),
        ];
        assert!(args_contain_token(&args, Some("abc123")));
    }

    #[test]
    fn args_contain_token_detects_inline_argument() {
        let args = vec![OsString::from("--service-token=abc123")];
        assert!(args_contain_token(&args, Some("abc123")));
    }

    #[test]
    fn args_contain_token_is_false_when_missing() {
        let args = vec![OsString::from("--service-token=abc123")];
        assert!(!args_contain_token(&args, Some("zzz")));
        assert!(!args_contain_token(&args, None));
    }

    #[test]
    fn paths_match_handles_equivalent_paths() {
        let temp = tempdir().expect("failed to create temp dir");
        let bin_dir = temp.path().join("bin");
        fs::create_dir_all(&bin_dir).expect("failed to create bin dir");
        let target = bin_dir.join("kittynode-server");
        File::create(&target).expect("failed to create dummy binary");

        let alternate = bin_dir.join(".").join("kittynode-server");
        assert!(paths_match(&target, &alternate));
    }

    #[test]
    fn paths_match_rejects_different_targets() {
        let temp = tempdir().expect("failed to create temp dir");
        let bin_dir = temp.path().join("bin");
        fs::create_dir_all(&bin_dir).expect("failed to create bin dir");

        let left = bin_dir.join("kittynode-server-a");
        let right = bin_dir.join("kittynode-server-b");
        File::create(&left).expect("create left");
        File::create(&right).expect("create right");

        assert!(!paths_match(&left, &right));
    }
}
