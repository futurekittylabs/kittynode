use eyre::{Context, ContextCompat, Result};
use serde::{Deserialize, Serialize};
use std::env;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::AppHandle;
use tauri::async_runtime::Receiver;
use tauri_plugin_shell::ShellExt;
use tauri_plugin_shell::process::{Command as ShellCommand, CommandChild, CommandEvent};
use tracing::{debug, warn};
use zeroize::Zeroize;

const DEPOSIT_BIN_NAME: &str = "deposit";
const DEPOSIT_BIN_ENV: &str = "KITTYNODE_DEPOSIT_BIN";
const DEFAULT_LANGUAGE: &str = "english";
const MNEMONIC_WORD_COUNT: usize = 24;

#[derive(Debug, Deserialize)]
pub struct GenerateMnemonicRequest {
    pub num_validators: u32,
    pub chain: String,
    pub keystore_password: String,
    pub withdrawal_address: Option<String>,
    #[serde(default)]
    pub compounding: bool,
    pub amount: Option<String>,
    pub mnemonic_language: Option<String>,
    #[serde(default)]
    pub pbkdf2: bool,
}

#[derive(Debug, Serialize)]
pub struct GenerateMnemonicResponse {
    pub mnemonic: String,
    pub run_directory: String,
    pub validator_keys_dir: String,
    pub deposit_files: Vec<String>,
    pub keystore_files: Vec<String>,
    pub stdout: Vec<String>,
    pub stderr: Vec<String>,
}

pub async fn generate_new_mnemonic(
    app: &AppHandle,
    params: GenerateMnemonicRequest,
) -> Result<GenerateMnemonicResponse> {
    let (run_dir, keys_dir_hint) = prepare_output_dirs()?;

    let GenerateMnemonicRequest {
        num_validators,
        chain,
        mut keystore_password,
        withdrawal_address,
        compounding,
        amount,
        mnemonic_language,
        pbkdf2,
    } = params;

    let mnemonic_language = mnemonic_language.unwrap_or_else(|| DEFAULT_LANGUAGE.to_string());

    let mut command = new_deposit_command(app)?
        .arg("--language")
        .arg(DEFAULT_LANGUAGE)
        .arg("--non_interactive")
        .arg("new-mnemonic")
        .arg("--mnemonic_language")
        .arg(&mnemonic_language)
        .arg("--num_validators")
        .arg(num_validators.to_string())
        .arg("--chain")
        .arg(&chain)
        .arg("--keystore_password")
        .arg(&keystore_password)
        .arg("--folder")
        .arg(run_dir.to_string_lossy().as_ref());

    if compounding {
        command = command.arg("--compounding");
    } else {
        command = command.arg("--regular-withdrawal");
    }

    if let Some(amount) = amount
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        command = command.arg("--amount").arg(amount);
    }

    validate_password(&keystore_password)?;

    if pbkdf2 {
        command = command.arg("--pbkdf2");
    }

    if let Some(addr) = withdrawal_address
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        command = command.arg("--withdrawal_address").arg(addr);
    }

    let (mut rx, mut child) = spawn_command(command)?;

    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let mut mnemonic: Option<String> = None;
    let mut keys_dir_reported: Option<PathBuf> = None;
    let exit_status = read_process_stream(
        &mut rx,
        &mut child,
        |line, process| {
            if mnemonic.is_none()
                && let Some(candidate) = extract_mnemonic(line)
            {
                mnemonic = Some(candidate);
            }

            if line.contains("Please type your mnemonic") {
                if let Some(value) = mnemonic.as_ref() {
                    if let Err(err) = process.write(format!("{}\n", value).as_bytes()) {
                        warn!(
                            "failed to send mnemonic confirmation to deposit CLI: {}",
                            err
                        );
                    }
                } else {
                    warn!("Mnemonic confirmation requested before mnemonic captured");
                }
            }

            if let Some(path) = extract_keys_path(line) {
                keys_dir_reported = Some(path);
            }

            stdout.push(line.to_string());
        },
        |line| {
            stderr.push(line.to_string());
        },
    )
    .await?;

    ensure_success(exit_status)?;

    let mut mnemonic = mnemonic.context("Failed to capture mnemonic from deposit CLI output")?;
    let keys_dir = keys_dir_reported.unwrap_or_else(|| keys_dir_hint.join("validator_keys"));

    let deposit_files = collect_matching_files(&keys_dir, |name| name.starts_with("deposit_data"))?;
    let keystore_files = collect_matching_files(&keys_dir, |name| name.starts_with("keystore"))?;

    let response = GenerateMnemonicResponse {
        mnemonic: mnemonic.clone(),
        run_directory: run_dir.to_string_lossy().into(),
        validator_keys_dir: keys_dir.to_string_lossy().into(),
        deposit_files,
        keystore_files,
        stdout,
        stderr,
    };

    mnemonic.zeroize();
    keystore_password.zeroize();

    Ok(response)
}

fn new_deposit_command(app: &AppHandle) -> Result<ShellCommand> {
    if let Ok(explicit_path) = env::var(DEPOSIT_BIN_ENV) {
        let path = PathBuf::from(explicit_path);
        if !path.exists() {
            return Err(eyre::eyre!(
                "{} points to {}, but the file does not exist",
                DEPOSIT_BIN_ENV,
                path.display()
            ));
        }

        return Ok(app.shell().command(path.as_os_str()));
    }

    app.shell().sidecar(DEPOSIT_BIN_NAME).map_err(|sidecar_err| {
        let expected = if cfg!(windows) {
            format!("src-tauri/bin/{}.exe", DEPOSIT_BIN_NAME)
        } else {
            format!("src-tauri/bin/{}", DEPOSIT_BIN_NAME)
        };

        eyre::eyre!(
            "failed to locate deposit CLI. Place an executable at {expected} or export {env}=</full/path/to/deposit>: {sidecar_err}",
            expected = expected,
            env = DEPOSIT_BIN_ENV
        )
    })
}

fn prepare_output_dirs() -> Result<(PathBuf, PathBuf)> {
    let home = home::home_dir().ok_or_else(|| eyre::eyre!("failed to resolve home directory"))?;
    let base = home.join(".kittynode").join("validator-assets");
    fs::create_dir_all(&base).wrap_err("failed to ensure validator-assets directory")?;

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let run_dir = base.join(format!("run-{}", timestamp));
    fs::create_dir_all(&run_dir).wrap_err("failed to create validator run directory")?;

    Ok((run_dir.clone(), run_dir))
}

fn extract_mnemonic(line: &str) -> Option<String> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return None;
    }

    let words: Vec<&str> = trimmed.split_whitespace().collect();
    if words.len() == MNEMONIC_WORD_COUNT {
        Some(trimmed.to_string())
    } else {
        None
    }
}

fn extract_keys_path(line: &str) -> Option<PathBuf> {
    const PREFIX: &str = "Your keys can be found at:";
    if let Some(idx) = line.find(PREFIX) {
        let path_part = line[idx + PREFIX.len()..].trim();
        if !path_part.is_empty() {
            return Some(PathBuf::from(path_part));
        }
    }
    None
}

#[allow(clippy::collapsible_if)]
fn collect_matching_files<F>(dir: &Path, predicate: F) -> Result<Vec<String>>
where
    F: Fn(&str) -> bool,
{
    let mut matches = Vec::new();
    if !dir.exists() {
        return Ok(matches);
    }

    for entry in
        fs::read_dir(dir).wrap_err_with(|| format!("failed to read directory {}", dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        if let Some(name) = path.file_name().and_then(OsStr::to_str) {
            if predicate(name) {
                matches.push(path.to_string_lossy().into());
            }
        }
    }

    matches.sort();
    Ok(matches)
}

fn spawn_command(
    command: tauri_plugin_shell::process::Command,
) -> Result<(Receiver<CommandEvent>, CommandChild)> {
    command
        .spawn()
        .map_err(|err| eyre::eyre!("failed to spawn deposit CLI: {err}"))
}

async fn read_process_stream<SO, SE>(
    rx: &mut Receiver<CommandEvent>,
    child: &mut CommandChild,
    mut on_stdout: SO,
    mut on_stderr: SE,
) -> Result<Option<i32>>
where
    SO: FnMut(&str, &mut CommandChild),
    SE: FnMut(&str),
{
    let mut exit_code: Option<i32> = None;

    while let Some(event) = rx.recv().await {
        match event {
            CommandEvent::Stdout(line) => {
                let text = clean_line(&line);
                debug!(target: "validator_cli", "stdout: {}", text);
                on_stdout(&text, child);
            }
            CommandEvent::Stderr(line) => {
                let text = clean_line(&line);
                debug!(target: "validator_cli", "stderr: {}", text);
                on_stderr(&text);
            }
            CommandEvent::Error(err) => {
                warn!("deposit CLI emitted error event: {}", err);
                on_stderr(&err);
            }
            CommandEvent::Terminated(payload) => {
                exit_code = payload.code;
                break;
            }
            _ => {}
        }
    }

    // Ensure stdin is closed so the process can exit cleanly
    if let Err(err) = child.write(&[]) {
        warn!("failed to signal EOF to deposit CLI stdin: {}", err);
    }

    Ok(exit_code)
}

fn clean_line(bytes: &[u8]) -> String {
    let text = String::from_utf8_lossy(bytes);
    text.trim_end_matches(['\r', '\n']).to_string()
}

fn ensure_success(code: Option<i32>) -> Result<()> {
    match code {
        Some(0) => Ok(()),
        Some(value) => Err(eyre::eyre!("deposit CLI exited with status {value}")),
        None => Err(eyre::eyre!("deposit CLI terminated without an exit code")),
    }
}

fn validate_password(password: &str) -> Result<()> {
    if password.contains('\n') {
        Err(eyre::eyre!(
            "keystore password must not contain newline characters"
        ))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_mnemonic_accepts_ascii_words() {
        let phrase = "word ".repeat(MNEMONIC_WORD_COUNT).trim().to_string();
        assert_eq!(extract_mnemonic(&phrase), Some(phrase));
    }

    #[test]
    fn extract_mnemonic_accepts_unicode() {
        let words = vec!["áccent"; MNEMONIC_WORD_COUNT].join(" ");
        assert_eq!(extract_mnemonic(&words), Some(words.clone()));

        let with_tabs = format!("\t{}\t", words);
        assert_eq!(extract_mnemonic(&with_tabs), Some(words));
    }

    #[test]
    fn extract_mnemonic_rejects_incorrect_length() {
        assert!(extract_mnemonic("one two three").is_none());
    }

    #[test]
    fn extract_keys_path_parses_output_line() {
        let line = "Success!\nYour keys can be found at: /tmp/keys";
        let path = extract_keys_path(line).expect("path should be parsed");
        assert_eq!(path, PathBuf::from("/tmp/keys"));
    }

    #[test]
    fn extract_keys_path_returns_none_when_missing() {
        assert!(extract_keys_path("nope").is_none());
    }

    #[test]
    fn clean_line_trims_newlines() {
        assert_eq!(clean_line(b"hello\n"), "hello");
        assert_eq!(clean_line(b"hello\r\n"), "hello");
    }

    #[test]
    fn validate_password_rejects_newlines() {
        assert!(validate_password("good-pass").is_ok());
        assert!(validate_password("bad\npass").is_err());
    }
}
