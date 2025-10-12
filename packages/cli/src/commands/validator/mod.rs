use std::{
    collections::HashMap,
    fs,
    io::{self, Write, stdout},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{Duration, Instant},
};

use bip39::{Language, Mnemonic, MnemonicType};
use crossterm::{
    cursor::MoveTo,
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    execute,
    terminal::{
        Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode,
        enable_raw_mode,
    },
};
use dialoguer::{Confirm, Input, Password, Select, theme::ColorfulTheme};
use eyre::{Report, Result, eyre};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Paragraph, Wrap},
};
use tokio::runtime::Handle;
use tracing::error;
use zeroize::Zeroizing;

// Docker (bollard) for starting the validator container
use bollard::{
    models::{ContainerCreateBody, EndpointSettings, NetworkingConfig},
    query_parameters::{CreateContainerOptionsBuilder, CreateImageOptionsBuilder},
    secret::HostConfig,
};
use tokio_stream::StreamExt;

#[cfg(target_os = "linux")]
use kittynode_core::api::validator::swap_active;
use kittynode_core::api::{
    self,
    types::PackageConfig,
    validator::{
        EPHEMERY_NETWORK_NAME, ValidatorKeygenOutcome, ValidatorKeygenRequest, ValidatorProgress,
        available_networks, check_internet_connectivity, ensure_ephemery_config,
        format_eth_from_gwei, generate_validator_files_with_progress, normalize_withdrawal_address,
        parse_deposit_amount_gwei, parse_validator_count, resolve_withdrawal_address,
        validate_password,
    },
};

fn desired_supported_networks() -> Vec<&'static str> {
    const DESIRED: &[&str] = &[EPHEMERY_NETWORK_NAME, "hoodi", "sepolia"];
    let available = available_networks();
    DESIRED
        .iter()
        .copied()
        .filter(|network| available.iter().any(|candidate| candidate == network))
        .collect()
}

pub struct KeygenSummary {
    pub deposit_data_path: PathBuf,
    pub output_dir: PathBuf,
    pub fee_recipient: String,
    pub network: String,
}

pub fn keygen(preselected_network: Option<&str>) -> Result<Option<KeygenSummary>> {
    let theme = ColorfulTheme::default();

    // Pre-check warnings block
    let mut warnings: Vec<String> = Vec::new();

    // Internet connectivity warning
    if check_internet_connectivity() {
        warnings.push(
            "Internet connectivity detected. You should never generate keys on a device that's ever been connected to the internet.".to_string(),
        );
    }

    // Swap detection or limitation
    #[cfg(target_os = "linux")]
    {
        if swap_active() {
            warnings.push(
                "System swap detected. Sensitive key material can be written to disk via swap and persist.".to_string(),
            );
        }
    }
    #[cfg(not(target_os = "linux"))]
    {
        warnings.push(
            "Swap detection is unavailable on this platform. Ensure swap or pagefile is disabled before generating keys.".to_string(),
        );
    }

    // Non-Unix permission enforcement limitation
    #[cfg(not(unix))]
    {
        warnings.push(
            "This platform does not support enforcing POSIX file permissions for keystores. Ensure the output directory is protected.".to_string(),
        );
    }

    if !warnings.is_empty() {
        println!("WARNING:");
        for w in &warnings {
            println!(" - {}", w);
        }
        let proceed = Confirm::with_theme(&theme)
            .with_prompt("Proceed despite the above warnings?")
            .default(false)
            .interact()?;
        if !proceed {
            println!("Aborting validator key generation.");
            return Ok(None);
        }
    }

    let validator_count_input = Input::<String>::with_theme(&theme)
        .with_prompt("How many validators do you wish to run?")
        .default("1".to_string())
        .validate_with(|text: &String| {
            parse_validator_count(text)
                .map(|_| ())
                .map_err(|error| error.to_string())
        })
        .interact_text()?;
    let validator_count = parse_validator_count(&validator_count_input)?;

    let network_labels = desired_supported_networks();
    if network_labels.is_empty() {
        return Err(eyre!(
            "No supported networks are available in this Lighthouse build. Please upgrade Lighthouse (and this CLI if needed)"
        ));
    }
    // Use preselected network if provided and supported; otherwise prompt.
    let network = if let Some(pre) = preselected_network {
        if network_labels.contains(&pre) {
            pre
        } else {
            let idx = Select::with_theme(&theme)
                .with_prompt(
                    "Selected network is not supported by this build. Choose a supported network",
                )
                .default(0)
                .items(&network_labels)
                .interact()?;
            network_labels
                .get(idx)
                .copied()
                .ok_or_else(|| eyre!("Invalid network selection"))?
        }
    } else {
        let network_index = Select::with_theme(&theme)
            .with_prompt("Select the network")
            .default(0)
            .items(&network_labels)
            .interact()?;
        network_labels
            .get(network_index)
            .copied()
            .ok_or_else(|| eyre!("Invalid network selection"))?
    };

    let add_withdrawal_address = Confirm::with_theme(&theme)
        .with_prompt("Add a withdrawal address (y/n)")
        .default(true)
        .interact()?;
    let (withdrawal_address_display, withdrawal_address_normalized) = if add_withdrawal_address {
        let input = Input::<String>::with_theme(&theme)
            .with_prompt("Enter the withdrawal address")
            .validate_with(|text: &String| {
                normalize_withdrawal_address(text)
                    .map(|_| ())
                    .map_err(|error| error.to_string())
            })
            .interact_text()?;
        let normalized = normalize_withdrawal_address(&input)?;
        (Some(input.trim().to_string()), Some(normalized))
    } else {
        (None, None)
    };

    let compounding = Confirm::with_theme(&theme)
        .with_prompt("Use 0x02 compounding validators?")
        .default(true)
        .interact()?;

    // Deposit per validator (ETH). Only prompt when using compounding validators to match deposit-cli UX.
    let deposit_amount_gwei_per_validator: u64 = if compounding {
        let input = Input::<String>::with_theme(&theme)
            .with_prompt("Deposit per validator (ETH)")
            .default("32".to_string())
            .validate_with(|text: &String| {
                const MIN_DEPOSIT_GWEI: u64 = 1_000_000_000; // 1 ETH
                const MAX_DEPOSIT_GWEI: u64 = 2_048_000_000_000; // 2048 ETH per deposit entry
                match parse_deposit_amount_gwei(text) {
                    Ok(gwei) => {
                        if gwei < MIN_DEPOSIT_GWEI {
                            Err("Per-validator deposit must be at least 1 ETH".to_string())
                        } else if gwei > MAX_DEPOSIT_GWEI {
                            Err("Per-validator deposit cannot exceed 2048 ETH".to_string())
                        } else {
                            Ok(())
                        }
                    }
                    Err(error) => Err(error.to_string()),
                }
            })
            .interact_text()?;
        parse_deposit_amount_gwei(&input)?
    } else {
        32_000_000_000 // exactly 32 ETH for non-compounding
    };
    let validator_count_u64 = u64::from(validator_count);
    let total_deposit_gwei = deposit_amount_gwei_per_validator * validator_count_u64;
    let deposit_amount_per_validator_eth_str =
        format_eth_from_gwei(deposit_amount_gwei_per_validator);

    // Allow user to select output directory for keys.
    let output_dir_input = Input::<String>::with_theme(&theme)
        .with_prompt("Output directory for validator keys")
        .default("./validator-keys".to_string())
        .interact_text()?;
    let output_dir = PathBuf::from(output_dir_input.trim());
    let output_dir_clone = output_dir.clone();

    println!("Validator key generation summary:");
    println!("  Validators: {validator_count}");
    println!("  Network: {}", network);
    let withdrawal_summary = withdrawal_address_display
        .as_deref()
        .unwrap_or("First generated Ethereum address");
    println!("  Withdrawal address: {}", withdrawal_summary);
    println!(
        "  0x02 compounding validators: {}",
        if compounding { "yes" } else { "no" }
    );
    let total_deposit_eth_str = format_eth_from_gwei(total_deposit_gwei);
    println!("  Total deposit: {} ETH", total_deposit_eth_str);
    println!(
        "  Deposit per validator: {} ETH",
        deposit_amount_per_validator_eth_str
    );
    println!("  Output directory: {}", output_dir.display());

    let confirm_details = Confirm::with_theme(&theme)
        .with_prompt("Are these details correct?")
        .default(true)
        .interact()?;
    if !confirm_details {
        println!("Aborting validator key generation.");
        return Ok(None);
    }

    let mnemonic = Mnemonic::new(MnemonicType::Words24, Language::English);
    let mnemonic_phrase = Zeroizing::new(mnemonic.to_string());
    drop(mnemonic);

    display_mnemonic_securely(mnemonic_phrase.as_str())?;
    let mnemonic_verified = validate_mnemonic_once(&theme, mnemonic_phrase.as_str())?;
    if let Err(error) = clear_clipboard() {
        error!("Failed to clear system clipboard, mnemonic may remain in clipboard: {error}");
    }
    if !mnemonic_verified {
        println!("✘ Mnemonic verification failed. Aborting validator key generation.");
        return Ok(None);
    }
    println!("Mnemonic successfully verified!");

    let password = Password::with_theme(&theme)
        .with_prompt("Enter a password to secure the keystore")
        .with_confirmation("Re-enter the password to confirm", "Passwords do not match")
        .validate_with(|value: &String| validate_password(value).map_err(|error| error.to_string()))
        .interact()?;

    let password = Zeroizing::new(password);

    let withdrawal_address = resolve_withdrawal_address(
        withdrawal_address_normalized.as_deref(),
        mnemonic_phrase.as_str(),
    )?;
    let fee_recipient = format!("{:#x}", withdrawal_address);
    if withdrawal_address_normalized.is_none() {
        println!(
            "Using withdrawal address derived from mnemonic: {:#x}",
            withdrawal_address
        );
    }
    println!("Generating {validator_count} validator(s)...");

    let outcome = generate_validator_files_with_progress(
        ValidatorKeygenRequest {
            mnemonic_phrase,
            validator_count,
            withdrawal_address,
            network: network.to_string(),
            deposit_gwei: deposit_amount_gwei_per_validator,
            compounding,
            password,
            output_dir,
        },
        |progress: ValidatorProgress| {
            println!("  → Validator {} of {}", progress.current, progress.total);
        },
    )?;

    let ValidatorKeygenOutcome {
        keystore_paths,
        deposit_data_path,
    } = outcome;

    println!(
        "✔ Generated {} validator keystore(s):",
        keystore_paths.len()
    );
    for path in &keystore_paths {
        println!("   {}", path.display());
    }
    println!("✔ Deposit data written to {}", deposit_data_path.display());

    println!("Store the password safely—it is not saved anywhere else.");

    Ok(Some(KeygenSummary {
        deposit_data_path,
        output_dir: output_dir_clone,
        fee_recipient,
        network: network.to_string(),
    }))
}

const DOCKER_DOCS_URL: &str = "https://docs.docker.com/get-docker/";
const NETWORK_OPTIONS: [&str; 3] = [EPHEMERY_NETWORK_NAME, "hoodi", "sepolia"];
const EXECUTION_OPTIONS: [&str; 1] = ["reth (only option, others coming soon)"];
const CONSENSUS_OPTIONS: [&str; 1] = ["lighthouse (only option, others coming soon)"];

#[derive(Clone, Copy, PartialEq, Eq)]
enum Step {
    Docker,
    Network,
    Execution,
    Consensus,
    Keygen,
    Summary,
    Launch,
    Deposit,
    Done,
}

struct StartState {
    step: Step,
    docker_running: bool,
    network_index: usize,
    execution_index: usize,
    consensus_index: usize,
    keygen_summary: Option<KeygenSummary>,
    status: Option<String>,
    aborted: bool,
}

impl StartState {
    fn new() -> Self {
        Self {
            step: Step::Docker,
            docker_running: false,
            network_index: 0,
            execution_index: 0,
            consensus_index: 0,
            keygen_summary: None,
            status: None,
            aborted: false,
        }
    }

    fn network(&self) -> &'static str {
        NETWORK_OPTIONS[self.network_index]
    }
}

pub async fn start() -> Result<()> {
    tokio::task::block_in_place(run_start_blocking)
}

fn run_start_blocking() -> Result<()> {
    let handle = Handle::current();
    let mut stdout = stdout();
    enable_raw_mode()?;
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;
    let mut state = StartState::new();
    let mut last_docker_check = Instant::now() - Duration::from_secs(2);

    loop {
        if state.step == Step::Docker && last_docker_check.elapsed() >= Duration::from_secs(1) {
            state.docker_running = handle.block_on(api::is_docker_running());
            last_docker_check = Instant::now();
            if state.docker_running {
                state.status = Some("Docker is running. Continuing setup.".to_string());
                state.step = Step::Network;
            } else {
                state.status = Some(format!(
                    "Docker is not running. Install or start it: {DOCKER_DOCS_URL}"
                ));
            }
        }

        terminal.draw(|frame| render(frame, &state))?;

        if state.aborted || state.step == Step::Done {
            break;
        }

        if event::poll(Duration::from_millis(120))?
            && let Event::Key(key) = event::read()?
        {
            if key.kind != KeyEventKind::Press {
                continue;
            }
            if matches!(key.code, KeyCode::Char('q')) {
                state.aborted = true;
                break;
            }
            handle_event(key, &mut state, &handle, &mut terminal)?;
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

fn handle_event(
    key: KeyEvent,
    state: &mut StartState,
    handle: &Handle,
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
) -> Result<()> {
    match state.step {
        Step::Docker => {
            if matches!(key.code, KeyCode::Enter) {
                state.status = Some("Re-checking Docker...".to_string());
            }
        }
        Step::Network => match key.code {
            KeyCode::Up if state.network_index > 0 => state.network_index -= 1,
            KeyCode::Down if state.network_index + 1 < NETWORK_OPTIONS.len() => {
                state.network_index += 1;
            }
            KeyCode::Enter => {
                state.step = Step::Execution;
                state.status = None;
            }
            _ => {}
        },
        Step::Execution => match key.code {
            KeyCode::Up if state.execution_index > 0 => state.execution_index -= 1,
            KeyCode::Down if state.execution_index + 1 < EXECUTION_OPTIONS.len() => {
                state.execution_index += 1;
            }
            KeyCode::Enter => {
                state.step = Step::Consensus;
                state.status = None;
            }
            _ => {}
        },
        Step::Consensus => match key.code {
            KeyCode::Up if state.consensus_index > 0 => state.consensus_index -= 1,
            KeyCode::Down if state.consensus_index + 1 < CONSENSUS_OPTIONS.len() => {
                state.consensus_index += 1;
            }
            KeyCode::Enter => {
                state.step = Step::Keygen;
                state.status = None;
            }
            _ => {}
        },
        Step::Keygen => {
            if matches!(key.code, KeyCode::Enter) {
                state.status = Some("Launching key generation...".to_string());
                if let Some(summary) = run_keygen_flow(terminal, handle, Some(state.network()))? {
                    // Ensure network consistency: if keygen chose a different network,
                    // update the wizard selection to match before launch.
                    if summary.network.as_str() != state.network()
                        && let Some(new_index) = NETWORK_OPTIONS
                            .iter()
                            .position(|n| *n == summary.network.as_str())
                    {
                        state.network_index = new_index;
                        state.status = Some(format!(
                            "Adjusted network to match keygen: {}",
                            state.network()
                        ));
                    }
                    // Store summary and proceed to Summary step
                    state.keygen_summary = Some(summary);
                    state.step = Step::Summary;
                    state.status = Some("Keys generated successfully.".to_string());
                } else {
                    state.status = Some("Key generation aborted.".to_string());
                }
            }
        }
        Step::Summary => {
            if matches!(key.code, KeyCode::Enter) {
                state.step = Step::Launch;
                state.status = None;
            }
        }
        Step::Launch => {
            if matches!(key.code, KeyCode::Enter)
                && let Some(summary) = state.keygen_summary.as_ref()
            {
                match run_launch_flow(terminal, handle, summary, state.network()) {
                    Ok(()) => {
                        state.status = Some("Clients started successfully.".to_string());
                        state.step = Step::Deposit;
                    }
                    Err(error) => {
                        state.status = Some(format!("Failed to start clients: {error}"));
                    }
                }
            }
        }
        Step::Deposit => {
            state.step = Step::Done;
        }
        Step::Done => {}
    }
    Ok(())
}

fn render(frame: &mut Frame, state: &StartState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(frame.area());
    let body = chunks[0];
    let footer = chunks[1];

    let mut lines = Vec::new();
    let title_style = Style::default().fg(Color::Cyan);

    match state.step {
        Step::Docker => {
            lines.push(Line::styled("Checking Docker", title_style));
            if state.docker_running {
                lines.push(Line::from("Docker detected. Preparing next step..."));
            } else {
                lines.push(Line::from("Waiting for Docker to become available..."));
                lines.push(Line::from(format!(
                    "Install instructions: {DOCKER_DOCS_URL}"
                )));
            }
        }
        Step::Network => {
            lines.push(Line::styled("Select a network", title_style));
            lines.push(Line::from("Use ↑/↓ and press Enter."));
            lines.push(Line::from(""));
            lines.extend(option_lines(state.network_index, &NETWORK_OPTIONS));
        }
        Step::Execution => {
            lines.push(Line::styled("Select an execution client", title_style));
            lines.push(Line::from("Use ↑/↓ and press Enter."));
            lines.push(Line::from(""));
            lines.extend(option_lines(state.execution_index, &EXECUTION_OPTIONS));
        }
        Step::Consensus => {
            lines.push(Line::styled("Select a consensus client", title_style));
            lines.push(Line::from("Use ↑/↓ and press Enter."));
            lines.push(Line::from(""));
            lines.extend(option_lines(state.consensus_index, &CONSENSUS_OPTIONS));
        }
        Step::Keygen => {
            lines.push(Line::styled("Generate validator keys", title_style));
            lines.push(Line::from(
                "Press Enter to launch the interactive keygen workflow.",
            ));
            lines.push(Line::from(
                "The Kittynode UI will resume automatically afterwards.",
            ));
        }
        Step::Summary => {
            lines.push(Line::styled("Key generation completed", title_style));
            if let Some(summary) = &state.keygen_summary {
                lines.push(Line::from(format!(
                    "Deposit data: {}",
                    summary.deposit_data_path.display()
                )));
                lines.push(Line::from(format!(
                    "Keystore directory: {}",
                    summary.output_dir.display()
                )));
                lines.push(Line::from(format!(
                    "Fee recipient: {}",
                    summary.fee_recipient
                )));
                lines.push(Line::from(format!("Network: {}", summary.network)));
                lines.push(Line::from("Write these paths down before continuing."));
            } else {
                lines.push(Line::from("No key information available."));
            }
            lines.push(Line::from(""));
            lines.push(Line::from("Press Enter when you are ready to continue."));
        }
        Step::Launch => {
            lines.push(Line::styled("Start clients", title_style));
            lines.push(Line::from(format!(
                "This will configure Reth and Lighthouse for {} and import your keys.",
                state.network()
            )));
            lines.push(Line::from(
                "Docker output will appear in the terminal while this runs.",
            ));
            lines.push(Line::from(""));
            lines.push(Line::from("Press Enter to continue."));
        }
        Step::Deposit => {
            let launchpad = match state.network() {
                "sepolia" => "https://sepolia.launchpad.ethereum.org",
                "ephemery" => "https://ephemery.dev/",
                _ => "https://hoodi.launchpad.ethereum.org",
            };
            lines.push(Line::styled("Final steps", title_style));
            if let Some(summary) = &state.keygen_summary {
                lines.push(Line::from(format!(
                    "Deposit file: {}",
                    summary.deposit_data_path.display()
                )));
            }
            lines.push(Line::from(format!(
                "Visit {launchpad} to submit the deposit data and 32 ETH per validator."
            )));
            lines.push(Line::from(
                "The validator client will wait for activation. Press any key to exit.",
            ));
            lines.push(Line::from(
                "Monitor progress later with `kittynode validator monitor`.",
            ));
        }
        Step::Done => {}
    }

    let paragraph = Paragraph::new(lines)
        .wrap(Wrap { trim: true })
        .alignment(ratatui::layout::Alignment::Left);
    frame.render_widget(paragraph, body);

    let status = state.status.as_deref().unwrap_or("");
    let foot_line = Line::from(vec![
        Span::raw(status),
        Span::raw(if status.is_empty() { "" } else { "  " }),
        Span::styled("press q to quit", Style::default().fg(Color::DarkGray)),
    ]);
    frame.render_widget(Paragraph::new(foot_line), footer);
}

fn option_lines(selected: usize, options: &[&str]) -> Vec<Line<'static>> {
    options
        .iter()
        .enumerate()
        .map(|(index, option)| {
            if index == selected {
                Line::styled(format!("> {option}"), Style::default().fg(Color::Yellow))
            } else {
                Line::from(format!("  {option}"))
            }
        })
        .collect()
}

fn run_keygen_flow(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    _handle: &Handle,
    preselected_network: Option<&str>,
) -> Result<Option<KeygenSummary>> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    // keygen flow is synchronous; use preselected network if provided
    let outcome = keygen(preselected_network);
    enable_raw_mode()?;
    execute!(terminal.backend_mut(), EnterAlternateScreen)?;
    terminal.hide_cursor()?;
    terminal.clear()?;
    outcome
}

fn run_launch_flow(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    handle: &Handle,
    summary: &KeygenSummary,
    network: &str,
) -> Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    let result = (|| -> Result<()> {
        println!("Configuring Ethereum clients for {network}...");
        let mut values = HashMap::new();
        values.insert("network".to_string(), network.to_string());
        let config = PackageConfig { values };
        let mut needs_install = false;
        let update_result = handle
            .clone()
            .block_on(async { api::update_package_config("Ethereum", config.clone()).await });
        if let Err(error) = update_result {
            if is_missing_volume_error(&error) {
                needs_install = true;
            } else {
                return Err(error);
            }
        }
        if needs_install {
            handle.block_on(async { api::install_package("Ethereum").await })?;
        }

        println!("Importing validator keys with Lighthouse...");
        let lighthouse_dir = lighthouse_root()?;
        run_validator_import(summary, network, &lighthouse_dir)?;

        println!("Starting Lighthouse validator client...");
        start_validator_container(handle, network, &lighthouse_dir, &summary.fee_recipient)?;
        println!("Execution and validator clients are running.");
        Ok(())
    })();

    println!("Press Enter to return to Kittynode UI.");
    let mut buffer = String::new();
    let _ = io::stdin().read_line(&mut buffer);

    enable_raw_mode()?;
    execute!(terminal.backend_mut(), EnterAlternateScreen)?;
    terminal.hide_cursor()?;
    terminal.clear()?;

    result
}

fn is_missing_volume_error(error: &Report) -> bool {
    error.to_string().to_lowercase().contains("no such volume")
}

fn run_validator_import(
    summary: &KeygenSummary,
    network: &str,
    lighthouse_dir: &Path,
) -> Result<()> {
    fs::create_dir_all(lighthouse_dir)?;
    let lighthouse_mount = canonicalize_path(lighthouse_dir);
    let keys_mount = canonicalize_path(&summary.output_dir);
    let ephemery = if network == EPHEMERY_NETWORK_NAME {
        Some(ensure_ephemery_config()?)
    } else {
        None
    };
    let metadata_mount = ephemery
        .as_ref()
        .map(|config| canonicalize_path(&config.metadata_dir));

    let mut command = Command::new("docker");
    command
        .arg("run")
        .arg("--rm")
        .arg("-i")
        .arg("-t")
        .arg("-v")
        .arg(format!("{}:/root/.lighthouse", lighthouse_mount.display()))
        .arg("-v")
        .arg(format!("{}:/root/validator_keys", keys_mount.display()));
    if let Some(mount) = metadata_mount.as_ref() {
        command
            .arg("-v")
            .arg(format!("{}:/root/networks/ephemery:ro", mount.display()));
    }
    command.arg("sigp/lighthouse").arg("lighthouse");
    if ephemery.is_some() {
        command.arg("--testnet-dir").arg("/root/networks/ephemery");
    } else {
        command.arg("--network").arg(network);
    }
    command
        .arg("account")
        .arg("validator")
        .arg("import")
        .arg("--directory")
        .arg("/root/validator_keys");
    let status = command
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;

    if status.success() {
        Ok(())
    } else {
        Err(eyre!("lighthouse import exited with status {status}"))
    }
}

fn start_validator_container(
    handle: &Handle,
    network: &str,
    lighthouse_dir: &Path,
    fee_recipient: &str,
) -> Result<()> {
    let lighthouse_mount = canonicalize_path(lighthouse_dir);
    let ephemery = if network == EPHEMERY_NETWORK_NAME {
        Some(ensure_ephemery_config()?)
    } else {
        None
    };
    let metadata_mount = ephemery
        .as_ref()
        .map(|config| canonicalize_path(&config.metadata_dir));

    handle.block_on(async move {
        let docker = kittynode_core::api::get_docker().await?;
        // Ensure daemon is reachable
        docker.version().await.map_err(eyre::Report::from)?;

        // Ensure image is present (pull if needed)
        let pull_opts = Some(
            CreateImageOptionsBuilder::default()
                .from_image("sigp/lighthouse")
                .tag("latest")
                .build(),
        );
        let mut pull_stream = docker.create_image(pull_opts, None, None);
        while let Some(_item) = pull_stream
            .next()
            .await
            .transpose()
            .map_err(eyre::Report::from)?
        {}

        // Build binds
        let mut binds = vec![format!("{}:/root/.lighthouse", lighthouse_mount.display())];
        if let Some(mount) = metadata_mount.as_ref() {
            binds.push(format!("{}:/root/networks/ephemery:ro", mount.display()));
        }

        // Build command
        let mut cmd: Vec<String> = vec!["lighthouse".into()];
        if ephemery.is_some() {
            cmd.push("--testnet-dir".into());
            cmd.push("/root/networks/ephemery".into());
        } else {
            cmd.push("--network".into());
            cmd.push(network.to_string());
        }
        // Prefer service discovery inside the user-defined Docker network
        // so this works consistently across Docker Desktop and Colima.
        // The beacon container is named 'lighthouse-node' and exposes 5052.
        let beacon_endpoint = "http://lighthouse-node:5052";
        cmd.extend([
            "vc".into(),
            "--beacon-nodes".into(),
            beacon_endpoint.into(),
            "--suggested-fee-recipient".into(),
            fee_recipient.to_string(),
        ]);

        // Create container (idempotent: remove existing with same name first)
        let container_name = "lighthouse-validator";
        // Best-effort forced removal by name to avoid 409 on create
        let _ = docker
            .remove_container(
                container_name,
                Some(bollard::query_parameters::RemoveContainerOptions {
                    force: true,
                    link: false,
                    v: false,
                }),
            )
            .await;

        // Create container attached to the same user-defined network
        // as the Ethereum package, enabling service-name resolution.
        let host_config = HostConfig {
            binds: Some(binds),
            ..Default::default()
        };

        // Resolve the Docker network name for the Ethereum package.
        let docker_network = kittynode_core::api::get_packages()
            .ok()
            .and_then(|pkgs| pkgs.get("Ethereum").map(|p| p.network_name().to_string()))
            .unwrap_or_else(|| "ethereum-network".to_string());

        let networking_config = NetworkingConfig {
            endpoints_config: Some(HashMap::from([(
                docker_network.clone(),
                EndpointSettings::default(),
            )])),
        };
        let config = ContainerCreateBody {
            image: Some("sigp/lighthouse".into()),
            cmd: Some(cmd),
            host_config: Some(host_config),
            networking_config: Some(networking_config),
            ..Default::default()
        };
        let create_opts = Some(
            CreateContainerOptionsBuilder::default()
                .name(container_name)
                .build(),
        );
        let created = docker
            .create_container(create_opts, config)
            .await
            .map_err(eyre::Report::from)?;
        docker
            .start_container(
                &created.id,
                None::<bollard::query_parameters::StartContainerOptions>,
            )
            .await
            .map_err(eyre::Report::from)?;
        Ok::<(), eyre::Report>(())
    })
}

fn lighthouse_root() -> Result<PathBuf> {
    Ok(api::kittynode_path()?.join(".lighthouse"))
}

fn canonicalize_path(path: &Path) -> PathBuf {
    fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}
fn clear_clipboard() -> Result<()> {
    let mut clipboard = arboard::Clipboard::new()
        .map_err(|error| eyre!("Failed to open system clipboard: {error}"))?;
    clipboard
        .set_text(String::new())
        .map_err(|error| eyre!("Failed to clear clipboard contents: {error}"))?;
    Ok(())
}

fn display_mnemonic_securely(mnemonic: &str) -> Result<()> {
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let result = (|| -> Result<()> {
        execute!(stdout, Clear(ClearType::All), MoveTo(0, 0))?;
        println!("IMPORTANT: Write down this mnemonic in a safe place. It will not be saved.\n");
        println!("Mnemonic phrase:\n");
        println!("{mnemonic}\n");
        println!("Press ENTER after you have written down the mnemonic to continue.");
        stdout.flush()?;

        let mut buffer = String::new();
        std::io::stdin().read_line(&mut buffer)?;
        Ok(())
    })();
    execute!(stdout, LeaveAlternateScreen)?;
    stdout.flush()?;
    result
}

fn validate_mnemonic_once(theme: &ColorfulTheme, mnemonic: &str) -> Result<bool> {
    let attempt = capture_mnemonic_securely(theme)?;
    Ok(normalize_mnemonic(&attempt) == normalize_mnemonic(mnemonic))
}

fn capture_mnemonic_securely(theme: &ColorfulTheme) -> Result<String> {
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let result = (|| -> Result<String> {
        execute!(stdout, Clear(ClearType::All), MoveTo(0, 0))?;
        println!("Please re-enter your mnemonic to confirm.\n");
        stdout.flush()?;

        Input::<String>::with_theme(theme)
            .with_prompt("Mnemonic phrase")
            .interact_text()
            .map_err(eyre::Report::from)
    })();
    execute!(stdout, LeaveAlternateScreen)?;
    stdout.flush()?;
    result
}

fn normalize_mnemonic(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}
