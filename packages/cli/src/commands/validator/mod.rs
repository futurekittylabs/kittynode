use std::{
    collections::HashMap,
    fs,
    io::{self, Write, stdout},
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use bip39::{Language, Mnemonic, MnemonicType};
use crossterm::{
    cursor::{MoveTo, Show},
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

/// Lighthouse validator container name shared across the CLI.
pub const VALIDATOR_CONTAINER_NAME: &str = "lighthouse-validator";

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

    let mut warnings: Vec<String> = Vec::new();

    if check_internet_connectivity() {
        warnings.push(
            "Internet connectivity detected. You should never generate keys on a device that's ever been connected to the internet.".to_string(),
        );
    }

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

struct RawTerminalGuard;

impl Drop for RawTerminalGuard {
    fn drop(&mut self) {
        if let Err(error) = disable_raw_mode() {
            error!("Failed to disable raw mode: {error}");
        }
        let mut out = stdout();
        let _ = execute!(out, LeaveAlternateScreen);
        let _ = execute!(out, Show);
    }
}

fn run_start_blocking() -> Result<()> {
    let handle = Handle::current();
    let mut stdout = stdout();
    enable_raw_mode()?;
    let _raw_terminal_guard = RawTerminalGuard;
    let _mute_logs = crate::log_control::mute_guard();
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
                match run_keygen_flow(terminal, handle, Some(state.network())) {
                    Ok(Some(summary)) => {
                        if summary.network.as_str() != state.network()
                            && let Some(new_index) = NETWORK_OPTIONS
                                .iter()
                                .position(|n| *n == summary.network.as_str())
                        {
                            state.network_index = new_index;
                            // Keep the wizard selection aligned with the network chosen during keygen.
                            state.status = Some(format!(
                                "Adjusted network to match keygen: {}",
                                state.network()
                            ));
                        }
                        state.keygen_summary = Some(summary);
                        state.step = Step::Summary;
                        state.status = Some("Keys generated successfully.".to_string());
                    }
                    Ok(None) => {
                        state.status = Some("Key generation aborted.".to_string());
                    }
                    Err(error) => {
                        state.status = Some(format!("Failed to generate keys: {error}"));
                    }
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
    // Allow logs to surface while the interactive prompts take over stdout.
    let _logs_on = crate::log_control::enable_guard();
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
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
    // Re-enable logging while the blocking container import flow executes.
    let _logs_on = crate::log_control::enable_guard();
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    let result = (|| -> Result<()> {
        handle.block_on(remove_validator_container_if_present());
        println!("Importing validator keys with Lighthouse...");
        let lighthouse_dir = lighthouse_root()?;
        run_validator_import(summary, network, &lighthouse_dir)?;
        println!("Configuring Ethereum clients for {network}...");
        let mut values = HashMap::new();
        values.insert("network".to_string(), network.to_string());
        values.insert("validator_enabled".to_string(), "true".to_string());
        values.insert(
            "validator_fee_recipient".to_string(),
            summary.fee_recipient.clone(),
        );
        let config = PackageConfig { values };
        let mut needs_install = false;
        let update_result = handle
            .clone()
            .block_on(async { api::update_package_config("Ethereum", config.clone()).await });
        if let Err(error) = update_result {
            if is_missing_docker_resource_error(&error) {
                needs_install = true;
            } else {
                return Err(error);
            }
        }
        if needs_install {
            handle.block_on(async { api::install_package("Ethereum").await })?;
        }
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

fn is_missing_docker_resource_error(error: &Report) -> bool {
    let msg = error.to_string().to_lowercase();
    // Missing Docker resources can happen on first install; trigger a reinstall path instead of failing.
    msg.contains("no such volume")
        || (msg.contains("volume") && msg.contains("not found"))
        || msg.contains("no such network")
        || (msg.contains("network") && msg.contains("not found"))
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

    use bollard::Docker;
    use bollard::models::ContainerCreateBody;
    use bollard::secret::HostConfig;
    use tokio::io::AsyncWriteExt;
    use tokio_stream::StreamExt;

    async fn import_with_bollard(
        lighthouse_mount: &Path,
        keys_mount: &Path,
        metadata_mount: Option<&Path>,
        network: &str,
        use_ephemery: bool,
        provided_password: &Zeroizing<String>,
    ) -> Result<()> {
        let docker: Docker = kittynode_core::api::get_docker().await?;

        let create_image_opts = Some(
            bollard::query_parameters::CreateImageOptionsBuilder::default()
                .from_image("sigp/lighthouse")
                .tag("latest")
                .build(),
        );
        let mut pull = docker.create_image(create_image_opts, None, None);
        while let Some(_progress) = pull.next().await {
            // Ignore pull progress; the CLI will surface meaningful output once the container starts.
        }

        let mut binds = vec![
            format!("{}:/root/.lighthouse", lighthouse_mount.display()),
            format!("{}:/root/validator_keys", keys_mount.display()),
        ];
        if let Some(meta) = metadata_mount {
            binds.push(format!("{}:/root/networks/ephemery:ro", meta.display()));
        }

        let mut cmd: Vec<String> = vec!["lighthouse".to_string(), "--stdin-inputs".to_string()];
        if use_ephemery {
            cmd.push("--testnet-dir".into());
            cmd.push("/root/networks/ephemery".into());
        } else {
            cmd.push("--network".into());
            cmd.push(network.to_string());
        }
        cmd.extend([
            "account".into(),
            "validator".into(),
            "import".into(),
            "--directory".into(),
            "/root/validator_keys".into(),
        ]);

        let host_config = HostConfig {
            binds: Some(binds),
            ..Default::default()
        };

        let config = ContainerCreateBody {
            image: Some("sigp/lighthouse".to_string()),
            cmd: Some(cmd),
            // Disable TTY so the password supplied over stdin is not echoed.
            tty: Some(false),
            attach_stdin: Some(true),
            attach_stdout: Some(true),
            attach_stderr: Some(true),
            open_stdin: Some(true),
            host_config: Some(host_config),
            ..Default::default()
        };

        let create_opts: Option<bollard::query_parameters::CreateContainerOptions> = None;
        let created = docker.create_container(create_opts, config).await?;

        docker
            .start_container(
                &created.id,
                None::<bollard::query_parameters::StartContainerOptions>,
            )
            .await?;

        let bollard::container::AttachContainerResults {
            mut output,
            mut input,
        } = docker
            .attach_container(
                &created.id,
                Some(
                    bollard::query_parameters::AttachContainerOptionsBuilder::default()
                        .stdin(true)
                        .stdout(true)
                        .stderr(true)
                        .stream(true)
                        .build(),
                ),
            )
            .await?;
        input.write_all(provided_password.as_bytes()).await.ok();
        input.write_all(b"\n").await.ok();
        drop(input);

        let mut exit_code: Option<i64> = None;
        let mut wait_stream = docker.wait_container(
            &created.id,
            None::<bollard::query_parameters::WaitContainerOptions>,
        );

        let output_task = tokio::spawn(async move {
            use std::io::Write as _;
            let mut out = std::io::stdout();
            while let Some(next) = output.next().await {
                match next {
                    Ok(chunk) => {
                        let bytes = chunk.into_bytes();
                        let _ = out.write_all(&bytes);
                        let _ = out.flush();
                    }
                    Err(_) => break,
                }
            }
        });

        if let Some(Ok(details)) = wait_stream.next().await {
            exit_code = Some(details.status_code);
        }

        let _ = output_task.await;

        let _ = docker
            .remove_container(
                &created.id,
                Some(
                    bollard::query_parameters::RemoveContainerOptionsBuilder::default()
                        .force(true)
                        .build(),
                ),
            )
            .await;

        match exit_code.unwrap_or_default() {
            0 => Ok(()),
            code => Err(eyre!("lighthouse import exited with status code {code}")),
        }
    }

    let theme = ColorfulTheme::default();
    let password = Password::with_theme(&theme)
        .with_prompt("Enter the keystore password to import validators")
        .validate_with(|value: &String| validate_password(value).map_err(|error| error.to_string()))
        .interact()?;
    let password = Zeroizing::new(password);
    let handle = Handle::current();
    handle.block_on(import_with_bollard(
        &lighthouse_mount,
        &keys_mount,
        metadata_mount.as_deref(),
        network,
        ephemery.is_some(),
        &password,
    ))
}

fn lighthouse_root() -> Result<PathBuf> {
    Ok(api::kittynode_path()?.join(".lighthouse"))
}

fn canonicalize_path(path: &Path) -> PathBuf {
    // Use dunce to get a platform-friendly canonical path (no \\\\?\\ prefix on Windows).
    dunce::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
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

/// Removes the validator container if present so other commands can safely reconfigure Ethereum state.
pub async fn remove_validator_container_if_present() {
    if let Ok(docker) = kittynode_core::api::get_docker().await {
        let _ = docker
            .remove_container(
                VALIDATOR_CONTAINER_NAME,
                Some(bollard::query_parameters::RemoveContainerOptions {
                    force: true,
                    link: false,
                    v: false,
                }),
            )
            .await;
    }
}
