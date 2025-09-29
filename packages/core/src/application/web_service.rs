use crate::domain::web_service::{DEFAULT_WEB_PORT, WebServiceStatus};
use crate::infra::web_service::{self, WebProcessState};
use eyre::{Result, WrapErr, eyre};
use rand::RngCore;
use std::ffi::{OsStr, OsString};
use std::fs::{self, OpenOptions};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::Duration;
use sysinfo::{Pid, Process, System};
use tracing::info;

pub fn start_web_service(
    port: Option<u16>,
    binary_path: &Path,
    args: &[&str],
) -> Result<WebServiceStatus> {
    let port = validate_web_port(port.unwrap_or(DEFAULT_WEB_PORT))?;
    if let Some(mut state) = web_service::load_state()? {
        if process_matches(&state) {
            if state.log_path.is_none() {
                state.log_path = Some(web_service::log_file_path()?);
                let _ = web_service::save_state(&state);
            }
            return Ok(WebServiceStatus::AlreadyRunning {
                pid: state.pid,
                port: state.port,
            });
        }
        web_service::clear_state()?;
    }

    let binary_path = binary_path
        .canonicalize()
        .unwrap_or_else(|_| binary_path.to_path_buf());
    let token = generate_service_token();

    let mut command = Command::new(&binary_path);
    let log_path = web_service::log_file_path()?;
    if let Some(parent) = log_path.parent() {
        fs::create_dir_all(parent)
            .wrap_err("Failed to create directories for kittynode-web log file")?;
    }
    let log_file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&log_path)
        .wrap_err("Failed to open kittynode-web log file")?;
    let stdout = log_file
        .try_clone()
        .wrap_err("Failed to duplicate log handle for stdout")?;

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
        .wrap_err("Failed to spawn kittynode-web process")?;

    wait_for_service_ready(&mut child, port)?;

    let pid = child.id();

    let state = WebProcessState {
        pid,
        port,
        binary: binary_path.clone(),
        token: Some(token),
        log_path: Some(log_path.clone()),
    };
    if let Err(err) = web_service::save_state(&state) {
        let kill_result = child.kill();
        if kill_result.is_ok() {
            let _ = child.wait();
        }
        let err = match kill_result {
            Ok(_) => err.wrap_err(format!(
                "Failed to persist kittynode-web state for pid {} on port {} and terminated spawned process",
                pid, port
            )),
            Err(kill_err) => err.wrap_err(format!(
                "Failed to persist kittynode-web state for pid {} on port {} and could not terminate spawned process: {kill_err}",
                pid, port
            )),
        };
        return Err(err);
    }

    drop(child);

    info!(pid, port = state.port, binary = %binary_path.display(), "Started kittynode-web service");
    Ok(WebServiceStatus::Started {
        pid: state.pid,
        port: state.port,
    })
}

pub fn get_web_service_log_path() -> Result<PathBuf> {
    let path = web_service::log_file_path()?;
    if path.exists() {
        return Ok(path);
    }

    if let Some(state) = web_service::load_state()?
        && process_matches(&state)
    {
        return Err(eyre!(
            "Kittynode web service logs are not available yet; restart the service to enable logging"
        ));
    }

    Err(eyre!(
        "Kittynode web service is not running; start it with `kittynode web start`"
    ))
}

pub fn stop_web_service() -> Result<WebServiceStatus> {
    let Some(state) = web_service::load_state()? else {
        return Ok(WebServiceStatus::NotRunning);
    };

    if !process_matches(&state) {
        web_service::clear_state()?;
        return Ok(WebServiceStatus::NotRunning);
    }

    let system = System::new_all();
    let pid = Pid::from_u32(state.pid);

    let Some(process) = system.process(pid) else {
        web_service::clear_state()?;
        return Ok(WebServiceStatus::NotRunning);
    };

    if !process.kill() {
        return Err(eyre!(
            "Failed to stop kittynode-web process with pid {}",
            state.pid
        ));
    }

    web_service::clear_state()?;
    info!(
        pid = state.pid,
        port = state.port,
        "Stopped kittynode-web service"
    );
    Ok(WebServiceStatus::Stopped {
        pid: state.pid,
        port: state.port,
    })
}

pub fn get_web_service_status() -> Result<WebServiceStatus> {
    let Some(mut state) = web_service::load_state()? else {
        return Ok(WebServiceStatus::NotRunning);
    };

    if process_matches(&state) {
        if state.log_path.is_none() {
            state.log_path = Some(web_service::log_file_path()?);
            let _ = web_service::save_state(&state);
        }
        return Ok(WebServiceStatus::AlreadyRunning {
            pid: state.pid,
            port: state.port,
        });
    }

    web_service::clear_state()?;
    Ok(WebServiceStatus::NotRunning)
}

fn process_matches(state: &WebProcessState) -> bool {
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

fn args_contain_token(cmd: &[OsString], token: Option<&str>) -> bool {
    let Some(token) = token else {
        return false;
    };
    let service_flag = OsStr::new("--service-token");
    let token_os = OsStr::new(token);
    let token_flag = OsString::from(format!("--service-token={token}"));
    for window in cmd.windows(2) {
        if window[0] == service_flag && window[1] == token_os {
            return true;
        }
    }
    cmd.iter().any(|arg| arg == &token_flag)
}

fn generate_service_token() -> String {
    let mut buf = [0u8; 16];
    rand::rng().fill_bytes(&mut buf);
    hex::encode(buf)
}

pub fn validate_web_port(port: u16) -> Result<u16> {
    if port == 0 {
        return Err(eyre!("Port must be greater than zero"));
    }
    Ok(port)
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
            .wrap_err("Failed to poll kittynode-web process state")?
        {
            let detail = status
                .code()
                .map(|code| format!("exit code {code}"))
                .unwrap_or_else(|| "terminated by signal".to_string());
            return Err(eyre!(
                "kittynode-web process exited immediately ({detail}); check logs for details"
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
        "Timed out waiting for kittynode-web to bind on port {}",
        port
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsString;
    use std::fs::{self, File};
    use tempfile::tempdir;

    #[test]
    fn validate_web_port_rejects_zero() {
        assert!(validate_web_port(0).is_err());
        assert_eq!(validate_web_port(8080).unwrap(), 8080);
    }

    #[test]
    fn generate_service_token_emits_hex_string() {
        let token = generate_service_token();
        assert_eq!(token.len(), 32);
        assert!(token.chars().all(|c| c.is_ascii_hexdigit()));
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
        let target = bin_dir.join("kittynode-web");
        File::create(&target).expect("failed to create dummy binary");

        let alternate = bin_dir.join(".").join("kittynode-web");
        assert!(paths_match(&target, &alternate));
    }

    #[test]
    fn paths_match_rejects_different_targets() {
        let temp = tempdir().expect("failed to create temp dir");
        let bin_dir = temp.path().join("bin");
        fs::create_dir_all(&bin_dir).expect("failed to create bin dir");
        let a = bin_dir.join("kittynode-web");
        let b = bin_dir.join("other-binary");
        File::create(&a).expect("failed to create binary a");
        File::create(&b).expect("failed to create binary b");

        assert!(!paths_match(&a, &b));
    }
}
