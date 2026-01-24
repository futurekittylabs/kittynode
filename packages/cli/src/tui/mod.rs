use std::{
    collections::{HashMap, HashSet},
    io::stdout,
    path::PathBuf,
    time::{Duration, Instant},
};

use crossterm::{
    cursor::Show,
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use eyre::Result;
use kittynode_core::api;
use kittynode_core::api::types::{
    Config, OperationalMode, OperationalState, Package, PackageState, RuntimeStatus, SystemInfo,
    WebServiceStatus,
};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, Borders, Cell, Clear, List, ListItem, Paragraph, Row, Table, TableState, Wrap,
    },
};
use tokio::runtime::Handle;
use tracing::{error, info};

const TICK_RATE: Duration = Duration::from_millis(160);
const REFRESH_INTERVAL: Duration = Duration::from_secs(5);
const MIN_WIDTH: u16 = 84;
const MIN_HEIGHT: u16 = 24;

pub async fn run() -> Result<()> {
    tokio::task::block_in_place(run_blocking)
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

fn run_blocking() -> Result<()> {
    let handle = Handle::current();
    let mut stdout = stdout();
    enable_raw_mode()?;
    let _raw_terminal_guard = RawTerminalGuard;
    let _mute_logs = crate::log_control::mute_guard();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;

    let mut app = App::new();
    app.refresh(&handle);

    let mut last_tick = Instant::now();
    let mut last_refresh = Instant::now();

    loop {
        terminal.draw(|frame| render(frame, &mut app))?;

        if app.should_quit {
            break;
        }

        let timeout = TICK_RATE.saturating_sub(last_tick.elapsed());
        if event::poll(timeout)? {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    app.handle_key(key, &handle, &mut terminal)?;
                }
                Event::Resize(_, _) => {}
                _ => {}
            }
        }

        if last_refresh.elapsed() >= REFRESH_INTERVAL {
            app.refresh(&handle);
            last_refresh = Instant::now();
        }

        last_tick = Instant::now();
    }

    Ok(())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Section {
    Overview,
    Packages,
    Catalog,
    Web,
    Docker,
    Config,
    System,
    Help,
}

impl Section {
    fn title(self) -> &'static str {
        match self {
            Section::Overview => "Overview",
            Section::Packages => "Packages",
            Section::Catalog => "Catalog",
            Section::Web => "Web",
            Section::Docker => "Docker",
            Section::Config => "Config",
            Section::System => "System",
            Section::Help => "Help",
        }
    }

    fn hint(self) -> &'static str {
        match self {
            Section::Overview => "r refresh | tab switch sections",
            Section::Packages => "enter start/stop | d delete | D delete+images | r refresh",
            Section::Catalog => "enter install | r refresh",
            Section::Web => "s start/stop | R restart | r refresh",
            Section::Docker => "s start docker | r refresh",
            Section::Config => "r refresh",
            Section::System => "r refresh",
            Section::Help => "tab switch sections",
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum StatusLevel {
    Info,
    Success,
    Warn,
    Error,
}

struct StatusMessage {
    text: String,
    level: StatusLevel,
}

struct AppData {
    operational_state: Option<OperationalState>,
    config: Option<Config>,
    system_info: Option<SystemInfo>,
    web_status: Option<WebServiceStatus>,
    web_log_path: Option<PathBuf>,
    packages: Vec<Package>,
    package_states: HashMap<String, PackageState>,
    catalog: Vec<Package>,
}

impl AppData {
    fn new() -> Self {
        Self {
            operational_state: None,
            config: None,
            system_info: None,
            web_status: None,
            web_log_path: None,
            packages: Vec::new(),
            package_states: HashMap::new(),
            catalog: Vec::new(),
        }
    }
}

struct App {
    sections: Vec<Section>,
    selected_section: usize,
    data: AppData,
    status: StatusMessage,
    package_state: TableState,
    catalog_state: TableState,
    modal: Option<Modal>,
    last_refresh: Option<Instant>,
    should_quit: bool,
}

impl App {
    fn new() -> Self {
        let sections = vec![
            Section::Overview,
            Section::Packages,
            Section::Catalog,
            Section::Web,
            Section::Docker,
            Section::Config,
            Section::System,
            Section::Help,
        ];
        let mut package_state = TableState::default();
        package_state.select(Some(0));
        let mut catalog_state = TableState::default();
        catalog_state.select(Some(0));
        Self {
            sections,
            selected_section: 0,
            data: AppData::new(),
            status: StatusMessage {
                text: "Ready".to_string(),
                level: StatusLevel::Info,
            },
            package_state,
            catalog_state,
            modal: None,
            last_refresh: None,
            should_quit: false,
        }
    }

    fn current_section(&self) -> Section {
        self.sections
            .get(self.selected_section)
            .copied()
            .unwrap_or(Section::Overview)
    }

    fn next_section(&mut self) {
        self.selected_section = (self.selected_section + 1) % self.sections.len();
    }

    fn previous_section(&mut self) {
        if self.selected_section == 0 {
            self.selected_section = self.sections.len() - 1;
        } else {
            self.selected_section -= 1;
        }
    }

    fn set_status(&mut self, text: impl Into<String>, level: StatusLevel) {
        self.status = StatusMessage {
            text: text.into(),
            level,
        };
    }

    fn refresh(&mut self, handle: &Handle) {
        self.last_refresh = Some(Instant::now());

        match api::get_config() {
            Ok(config) => self.data.config = Some(config),
            Err(err) => {
                self.set_status(format!("Config error: {err}"), StatusLevel::Error);
                error!(error = %err, "Failed to load config");
            }
        }

        match handle.block_on(api::get_operational_state()) {
            Ok(state) => self.data.operational_state = Some(state),
            Err(err) => {
                self.set_status(
                    format!("Operational state error: {err}"),
                    StatusLevel::Error,
                );
                error!(error = %err, "Failed to load operational state");
            }
        }

        match api::get_system_info() {
            Ok(info) => self.data.system_info = Some(info),
            Err(err) => {
                self.set_status(format!("System info error: {err}"), StatusLevel::Error);
                error!(error = %err, "Failed to load system info");
            }
        }

        match api::get_web_service_status() {
            Ok(status) => {
                self.data.web_status = Some(status);
                match api::get_web_service_log_path() {
                    Ok(path) => self.data.web_log_path = Some(path),
                    Err(_) => self.data.web_log_path = None,
                }
            }
            Err(err) => {
                self.set_status(format!("Web status error: {err}"), StatusLevel::Error);
                error!(error = %err, "Failed to load web status");
            }
        }

        match handle.block_on(api::get_installed_packages()) {
            Ok(packages) => {
                self.data.packages = packages;
                self.data.packages.sort_by(|a, b| a.name().cmp(b.name()));
                let mut names: Vec<String> = self
                    .data
                    .packages
                    .iter()
                    .map(|pkg| pkg.name().to_string())
                    .collect();
                names.sort();
                let name_refs: Vec<&str> = names.iter().map(|name| name.as_str()).collect();
                if name_refs.is_empty() {
                    self.data.package_states.clear();
                } else {
                    match handle.block_on(api::get_packages(&name_refs)) {
                        Ok(states) => self.data.package_states = states,
                        Err(err) => {
                            self.set_status(
                                format!("Package state error: {err}"),
                                StatusLevel::Error,
                            );
                            error!(error = %err, "Failed to load package states");
                        }
                    }
                }
                self.sync_package_selection();
            }
            Err(err) => {
                self.set_status(format!("Package list error: {err}"), StatusLevel::Error);
                error!(error = %err, "Failed to load packages");
            }
        }

        match api::get_package_catalog() {
            Ok(catalog) => {
                let mut entries: Vec<Package> = catalog.into_values().collect();
                entries.sort_by(|a, b| a.name().cmp(b.name()));
                self.data.catalog = entries;
                self.sync_catalog_selection();
            }
            Err(err) => {
                self.set_status(format!("Catalog error: {err}"), StatusLevel::Error);
                error!(error = %err, "Failed to load catalog");
            }
        }
    }

    fn sync_package_selection(&mut self) {
        if self.data.packages.is_empty() {
            self.package_state.select(None);
            return;
        }
        let selected = self.package_state.selected().unwrap_or(0);
        let next = selected.min(self.data.packages.len() - 1);
        self.package_state.select(Some(next));
    }

    fn sync_catalog_selection(&mut self) {
        if self.data.catalog.is_empty() {
            self.catalog_state.select(None);
            return;
        }
        let selected = self.catalog_state.selected().unwrap_or(0);
        let next = selected.min(self.data.catalog.len() - 1);
        self.catalog_state.select(Some(next));
    }

    fn handle_key(
        &mut self,
        key: KeyEvent,
        handle: &Handle,
        terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    ) -> Result<()> {
        if let Some(modal) = self.modal.take() {
            self.modal = modal.handle_key(self, key, handle, terminal)?;
            return Ok(());
        }

        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.should_quit = true;
            }
            KeyCode::Tab => self.next_section(),
            KeyCode::BackTab => self.previous_section(),
            KeyCode::Char('r') => self.refresh(handle),
            KeyCode::Char('?') => {
                self.selected_section = self
                    .sections
                    .iter()
                    .position(|section| *section == Section::Help)
                    .unwrap_or(self.selected_section);
            }
            KeyCode::Up => self.handle_up(),
            KeyCode::Down => self.handle_down(),
            KeyCode::Enter => self.handle_enter(handle, terminal)?,
            KeyCode::Char('d') => self.handle_delete(false),
            KeyCode::Char('D') => self.handle_delete(true),
            KeyCode::Char('s') => self.handle_start_stop(handle, terminal)?,
            KeyCode::Char('R') => self.handle_restart(handle, terminal)?,
            _ => {}
        }

        Ok(())
    }

    fn handle_up(&mut self) {
        match self.current_section() {
            Section::Packages => move_selection_up(&mut self.package_state),
            Section::Catalog => move_selection_up(&mut self.catalog_state),
            _ => {}
        }
    }

    fn handle_down(&mut self) {
        match self.current_section() {
            Section::Packages => {
                move_selection_down(&mut self.package_state, self.data.packages.len())
            }
            Section::Catalog => {
                move_selection_down(&mut self.catalog_state, self.data.catalog.len())
            }
            _ => {}
        }
    }

    fn handle_enter(
        &mut self,
        handle: &Handle,
        terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    ) -> Result<()> {
        match self.current_section() {
            Section::Packages => self.toggle_package(handle, terminal),
            Section::Catalog => self.install_selected(handle, terminal),
            _ => Ok(()),
        }
    }

    fn handle_delete(&mut self, include_images: bool) {
        if self.current_section() != Section::Packages {
            return;
        }
        let Some(index) = self.package_state.selected() else {
            self.set_status("No package selected", StatusLevel::Warn);
            return;
        };
        let Some(package) = self.data.packages.get(index) else {
            self.set_status("No package selected", StatusLevel::Warn);
            return;
        };
        self.modal = Some(Modal::ConfirmDelete {
            package_name: package.name().to_string(),
            include_images,
        });
    }

    fn handle_start_stop(
        &mut self,
        handle: &Handle,
        terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    ) -> Result<()> {
        match self.current_section() {
            Section::Web => self.toggle_web(handle, terminal),
            Section::Docker => self.start_docker(handle, terminal),
            _ => Ok(()),
        }
    }

    fn handle_restart(
        &mut self,
        handle: &Handle,
        terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    ) -> Result<()> {
        if self.current_section() != Section::Web {
            return Ok(());
        }
        self.restart_web(handle, terminal)
    }

    fn toggle_package(
        &mut self,
        handle: &Handle,
        terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    ) -> Result<()> {
        let Some(index) = self.package_state.selected() else {
            self.set_status("No package selected", StatusLevel::Warn);
            return Ok(());
        };
        let Some(package) = self.data.packages.get(index) else {
            self.set_status("No package selected", StatusLevel::Warn);
            return Ok(());
        };
        let name = package.name().to_string();
        let status = self
            .data
            .package_states
            .get(&name)
            .map(|state| state.runtime);

        let should_stop = matches!(
            status,
            Some(RuntimeStatus::Running | RuntimeStatus::PartiallyRunning)
        );
        let action = if should_stop { "Stopping" } else { "Starting" };
        self.set_status(format!("{action} {name}..."), StatusLevel::Info);
        terminal.draw(|frame| render(frame, self))?;

        let result = if should_stop {
            handle.block_on(api::stop_package(&name))
        } else {
            handle.block_on(api::start_package(&name))
        };

        match result {
            Ok(()) => {
                info!(package = %name, "Package state updated");
                self.set_status(format!("{action} {name} complete"), StatusLevel::Success);
                self.refresh(handle);
            }
            Err(err) => {
                error!(error = %err, package = %name, "Failed to update package");
                self.set_status(format!("{action} {name} failed: {err}"), StatusLevel::Error);
            }
        }
        Ok(())
    }

    fn install_selected(
        &mut self,
        handle: &Handle,
        terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    ) -> Result<()> {
        let Some(index) = self.catalog_state.selected() else {
            self.set_status("No package selected", StatusLevel::Warn);
            return Ok(());
        };
        let Some(package) = self.data.catalog.get(index) else {
            self.set_status("No package selected", StatusLevel::Warn);
            return Ok(());
        };
        let name = package.name().to_string();
        if self
            .data
            .packages
            .iter()
            .any(|installed| installed.name() == name)
        {
            self.set_status(format!("{name} is already installed"), StatusLevel::Warn);
            return Ok(());
        }

        if name == "ethereum" {
            let networks = api::ethereum_supported_networks_display("|");
            let options: Vec<String> = networks
                .split('|')
                .map(|entry| entry.trim().to_string())
                .filter(|entry| !entry.is_empty())
                .collect();
            if options.is_empty() {
                self.set_status(
                    "No supported Ethereum networks available",
                    StatusLevel::Error,
                );
                return Ok(());
            }
            self.modal = Some(Modal::SelectNetwork {
                package_name: name,
                networks: options,
                selected: 0,
            });
            return Ok(());
        }

        self.install_package(handle, terminal, name, None)
    }

    fn install_package(
        &mut self,
        handle: &Handle,
        terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
        name: String,
        network: Option<String>,
    ) -> Result<()> {
        let network_display = network.as_deref().unwrap_or("default");
        self.set_status(
            format!("Installing {name} ({network_display})..."),
            StatusLevel::Info,
        );
        terminal.draw(|frame| render(frame, self))?;

        let result = handle.block_on(api::install_package_with_network(&name, network.as_deref()));
        match result {
            Ok(()) => {
                info!(package = %name, network = ?network, "Package installed");
                self.set_status(format!("{name} installed"), StatusLevel::Success);
                self.refresh(handle);
            }
            Err(err) => {
                error!(error = %err, package = %name, "Failed to install package");
                self.set_status(format!("Install failed: {err}"), StatusLevel::Error);
            }
        }
        Ok(())
    }

    fn toggle_web(
        &mut self,
        _handle: &Handle,
        terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    ) -> Result<()> {
        let status = self
            .data
            .web_status
            .clone()
            .unwrap_or(WebServiceStatus::NotRunning);
        match status {
            WebServiceStatus::Started { .. } | WebServiceStatus::AlreadyRunning { .. } => {
                self.stop_web(terminal)
            }
            _ => self.start_web(terminal),
        }
    }

    fn start_web(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    ) -> Result<()> {
        self.set_status("Starting web service...", StatusLevel::Info);
        terminal.draw(|frame| render(frame, self))?;

        let binary = match std::env::current_exe() {
            Ok(path) => path,
            Err(err) => {
                self.set_status(
                    format!("Failed to locate kittynode binary: {err}"),
                    StatusLevel::Error,
                );
                error!(error = %err, "Failed to locate kittynode binary");
                return Ok(());
            }
        };
        let status = api::start_web_service(
            None,
            &binary,
            &["web", crate::commands::WEB_INTERNAL_SUBCOMMAND],
        );

        match status {
            Ok(state) => {
                info!("Web service started");
                self.data.web_status = Some(state);
                self.set_status("Web service started", StatusLevel::Success);
                self.refresh(&Handle::current());
            }
            Err(err) => {
                error!(error = %err, "Failed to start web service");
                self.set_status(format!("Web start failed: {err}"), StatusLevel::Error);
            }
        }
        Ok(())
    }

    fn stop_web(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    ) -> Result<()> {
        self.set_status("Stopping web service...", StatusLevel::Info);
        terminal.draw(|frame| render(frame, self))?;

        match api::stop_web_service() {
            Ok(state) => {
                info!("Web service stopped");
                self.data.web_status = Some(state);
                self.set_status("Web service stopped", StatusLevel::Success);
                self.refresh(&Handle::current());
            }
            Err(err) => {
                error!(error = %err, "Failed to stop web service");
                self.set_status(format!("Web stop failed: {err}"), StatusLevel::Error);
            }
        }
        Ok(())
    }

    fn restart_web(
        &mut self,
        handle: &Handle,
        terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    ) -> Result<()> {
        self.set_status("Restarting web service...", StatusLevel::Info);
        terminal.draw(|frame| render(frame, self))?;

        let port = restart_port_from_status(self.data.web_status.as_ref());

        match api::stop_web_service() {
            Ok(_) => {}
            Err(err) => {
                error!(error = %err, "Failed to stop web service during restart");
                self.set_status(format!("Restart failed: {err}"), StatusLevel::Error);
                return Ok(());
            }
        }

        match api::start_web_service(
            port,
            &match std::env::current_exe() {
                Ok(path) => path,
                Err(err) => {
                    self.set_status(
                        format!("Failed to locate kittynode binary: {err}"),
                        StatusLevel::Error,
                    );
                    error!(error = %err, "Failed to locate kittynode binary");
                    return Ok(());
                }
            },
            &["web", crate::commands::WEB_INTERNAL_SUBCOMMAND],
        ) {
            Ok(state) => {
                info!("Web service restarted");
                self.data.web_status = Some(state);
                self.set_status("Web service restarted", StatusLevel::Success);
                self.refresh(handle);
            }
            Err(err) => {
                error!(error = %err, "Failed to start web service during restart");
                self.set_status(format!("Restart failed: {err}"), StatusLevel::Error);
            }
        }
        Ok(())
    }

    fn start_docker(
        &mut self,
        handle: &Handle,
        terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    ) -> Result<()> {
        self.set_status("Starting Docker...", StatusLevel::Info);
        terminal.draw(|frame| render(frame, self))?;

        match handle.block_on(api::start_docker_if_needed()) {
            Ok(status) => {
                info!(status = ?status, "Docker start requested");
                self.set_status(
                    format!("Docker status: {}", status.as_str()),
                    StatusLevel::Success,
                );
                self.refresh(handle);
            }
            Err(err) => {
                error!(error = %err, "Failed to start Docker");
                self.set_status(format!("Docker start failed: {err}"), StatusLevel::Error);
            }
        }
        Ok(())
    }
}

fn restart_port_from_status(status: Option<&WebServiceStatus>) -> Option<u16> {
    match status {
        Some(WebServiceStatus::Started { port, .. })
        | Some(WebServiceStatus::AlreadyRunning { port, .. })
        | Some(WebServiceStatus::Stopped { port, .. }) => Some(*port),
        _ => None,
    }
}

enum Modal {
    ConfirmDelete {
        package_name: String,
        include_images: bool,
    },
    SelectNetwork {
        package_name: String,
        networks: Vec<String>,
        selected: usize,
    },
}

impl Modal {
    fn handle_key(
        mut self,
        app: &mut App,
        key: KeyEvent,
        handle: &Handle,
        terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    ) -> Result<Option<Modal>> {
        match &mut self {
            Modal::ConfirmDelete {
                package_name,
                include_images,
            } => match key.code {
                KeyCode::Char('y') | KeyCode::Char('Y') => {
                    let name = package_name.clone();
                    let include_images = *include_images;
                    app.set_status(format!("Deleting {name}..."), StatusLevel::Info);
                    terminal.draw(|frame| render(frame, app))?;
                    let result = handle.block_on(api::delete_package(&name, include_images));
                    match result {
                        Ok(()) => {
                            info!(package = %name, "Package deleted");
                            app.set_status(format!("Deleted {name}"), StatusLevel::Success);
                            app.refresh(handle);
                        }
                        Err(err) => {
                            error!(error = %err, package = %name, "Failed to delete package");
                            app.set_status(format!("Delete failed: {err}"), StatusLevel::Error);
                        }
                    }
                    return Ok(None);
                }
                KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                    app.set_status("Delete cancelled", StatusLevel::Info);
                    return Ok(None);
                }
                _ => {}
            },
            Modal::SelectNetwork {
                package_name,
                networks,
                selected,
            } => match key.code {
                KeyCode::Up => {
                    if *selected > 0 {
                        *selected -= 1;
                    }
                }
                KeyCode::Down => {
                    if *selected + 1 < networks.len() {
                        *selected += 1;
                    }
                }
                KeyCode::Enter => {
                    let name = package_name.clone();
                    let network = networks.get(*selected).cloned().unwrap_or_default();
                    app.install_package(handle, terminal, name, Some(network))?;
                    return Ok(None);
                }
                KeyCode::Esc => {
                    app.set_status("Install cancelled", StatusLevel::Info);
                    return Ok(None);
                }
                _ => {}
            },
        }

        Ok(Some(self))
    }
}

struct Theme {
    accent: Color,
    accent_soft: Color,
    text: Color,
    muted: Color,
    success: Color,
    warning: Color,
    danger: Color,
}

impl Theme {
    fn new() -> Self {
        Self {
            accent: Color::Rgb(115, 215, 255),
            accent_soft: Color::Rgb(180, 235, 255),
            text: Color::Rgb(235, 235, 235),
            muted: Color::Rgb(140, 150, 160),
            success: Color::Rgb(126, 214, 165),
            warning: Color::Rgb(245, 209, 122),
            danger: Color::Rgb(255, 138, 128),
        }
    }
}

fn render(frame: &mut Frame, app: &mut App) {
    let theme = Theme::new();
    let size = frame.area();
    if size.width < MIN_WIDTH || size.height < MIN_HEIGHT {
        render_too_small(frame, size, &theme);
        return;
    }

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(2),
        ])
        .split(size);

    render_header(frame, layout[0], app, &theme);
    render_body(frame, layout[1], app, &theme);
    render_footer(frame, layout[2], app, &theme);

    if let Some(modal) = &app.modal {
        render_modal(frame, modal, &theme);
    }
}

fn render_too_small(frame: &mut Frame, area: Rect, theme: &Theme) {
    let block = Block::new().borders(Borders::ALL).title("Kittynode");
    let message = Paragraph::new("Resize the terminal to continue.")
        .style(Style::default().fg(theme.text))
        .alignment(Alignment::Center)
        .block(block);
    frame.render_widget(message, area);
}

fn render_header(frame: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let version = env!("CARGO_PKG_VERSION");
    let server = app
        .data
        .config
        .as_ref()
        .map(|config| {
            if config.server_url.trim().is_empty() {
                "(local)".to_string()
            } else {
                config.server_url.clone()
            }
        })
        .unwrap_or_else(|| "loading...".to_string());

    let refresh = app
        .last_refresh
        .map(|instant| format!("refresh {} ago", format_duration(instant.elapsed())))
        .unwrap_or_else(|| "refresh pending".to_string());

    let title = Line::from(vec![
        Span::styled(
            "Kittynode",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" "),
        Span::styled(
            "Control",
            Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" "),
        Span::styled(format!("v{version}"), Style::default().fg(theme.muted)),
    ]);

    let subtitle = Line::from(vec![
        Span::styled("Server", Style::default().fg(theme.muted)),
        Span::raw(": "),
        Span::styled(server, Style::default().fg(theme.text)),
        Span::raw("  |  "),
        Span::styled(refresh, Style::default().fg(theme.muted)),
    ]);

    let header = Paragraph::new(vec![title, subtitle])
        .block(Block::new().borders(Borders::BOTTOM))
        .alignment(Alignment::Left);
    frame.render_widget(header, area);
}

fn render_body(frame: &mut Frame, area: Rect, app: &mut App, theme: &Theme) {
    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(22), Constraint::Min(0)])
        .split(area);
    render_nav(frame, layout[0], app, theme);
    render_section(frame, layout[1], app, theme);
}

fn render_nav(frame: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let items: Vec<ListItem> = app
        .sections
        .iter()
        .enumerate()
        .map(|(index, section)| {
            let selected = index == app.selected_section;
            let style = if selected {
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.text)
            };
            ListItem::new(Line::from(Span::styled(section.title(), style)))
        })
        .collect();

    let list = List::new(items)
        .block(Block::new().borders(Borders::ALL).title("Sections"))
        .highlight_style(Style::default().fg(theme.accent));
    frame.render_widget(list, area);
}

fn render_section(frame: &mut Frame, area: Rect, app: &mut App, theme: &Theme) {
    match app.current_section() {
        Section::Overview => render_overview(frame, area, app, theme),
        Section::Packages => render_packages(frame, area, app, theme),
        Section::Catalog => render_catalog(frame, area, app, theme),
        Section::Web => render_web(frame, area, app, theme),
        Section::Docker => render_docker(frame, area, app, theme),
        Section::Config => render_config(frame, area, app, theme),
        Section::System => render_system(frame, area, app, theme),
        Section::Help => render_help(frame, area, theme),
    }
}

fn render_overview(frame: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    let left = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(columns[0]);

    let right = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(columns[1]);

    let operational_lines = build_operational_lines(app.data.operational_state.as_ref(), theme);
    let operational = Paragraph::new(operational_lines)
        .wrap(Wrap { trim: true })
        .block(
            Block::new()
                .borders(Borders::ALL)
                .title("Operational State"),
        );
    frame.render_widget(operational, left[0]);

    let service_lines = build_service_lines(app.data.web_status.as_ref(), theme);
    let services = Paragraph::new(service_lines)
        .wrap(Wrap { trim: true })
        .block(Block::new().borders(Borders::ALL).title("Services"));
    frame.render_widget(services, left[1]);

    let package_lines = build_package_summary(app, theme);
    let packages = Paragraph::new(package_lines)
        .wrap(Wrap { trim: true })
        .block(Block::new().borders(Borders::ALL).title("Packages"));
    frame.render_widget(packages, right[0]);

    let config_lines = build_config_summary(app.data.config.as_ref(), theme);
    let config = Paragraph::new(config_lines)
        .wrap(Wrap { trim: true })
        .block(Block::new().borders(Borders::ALL).title("Configuration"));
    frame.render_widget(config, right[1]);
}

fn build_operational_lines(state: Option<&OperationalState>, theme: &Theme) -> Vec<Line<'static>> {
    let Some(state) = state else {
        return vec![Line::from(Span::styled(
            "Loading operational state...",
            Style::default().fg(theme.muted),
        ))];
    };

    let mode = match state.mode {
        OperationalMode::Local => "local",
        OperationalMode::Remote => "remote",
    };
    let docker_status = if state.docker_running {
        Span::styled("running", Style::default().fg(theme.success))
    } else {
        Span::styled("offline", Style::default().fg(theme.danger))
    };
    let can_install = if state.can_install { "yes" } else { "no" };
    let can_manage = if state.can_manage { "yes" } else { "no" };

    let mut lines = vec![
        Line::from(format!("Mode: {mode}")),
        Line::from(vec![Span::raw("Docker: "), docker_status]),
        Line::from(format!("Can install: {can_install}")),
        Line::from(format!("Can manage: {can_manage}")),
    ];

    if !state.diagnostics.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Diagnostics",
            Style::default().fg(theme.muted),
        )));
        for entry in &state.diagnostics {
            lines.push(Line::from(format!("- {entry}")));
        }
    }

    lines
}

fn build_service_lines(status: Option<&WebServiceStatus>, theme: &Theme) -> Vec<Line<'static>> {
    match status {
        Some(WebServiceStatus::Started { pid, port })
        | Some(WebServiceStatus::AlreadyRunning { pid, port }) => vec![
            Line::from(vec![
                Span::raw("Web: "),
                Span::styled("running", Style::default().fg(theme.success)),
            ]),
            Line::from(format!("Port: {port}")),
            Line::from(format!("PID: {pid}")),
        ],
        Some(WebServiceStatus::Stopped { pid, port }) => vec![
            Line::from(vec![
                Span::raw("Web: "),
                Span::styled("stopped", Style::default().fg(theme.warning)),
            ]),
            Line::from(format!("Last port: {port}")),
            Line::from(format!("Last PID: {pid}")),
        ],
        Some(WebServiceStatus::NotRunning) => vec![Line::from(vec![
            Span::raw("Web: "),
            Span::styled("not running", Style::default().fg(theme.muted)),
        ])],
        None => vec![Line::from(Span::styled(
            "Web status unavailable",
            Style::default().fg(theme.muted),
        ))],
    }
}

fn build_package_summary(app: &App, theme: &Theme) -> Vec<Line<'static>> {
    let total = app.data.packages.len();
    let mut running = 0;
    for pkg in &app.data.packages {
        if matches!(
            app.data
                .package_states
                .get(pkg.name())
                .map(|state| state.runtime),
            Some(RuntimeStatus::Running | RuntimeStatus::PartiallyRunning)
        ) {
            running += 1;
        }
    }
    vec![
        Line::from(format!("Installed: {total}")),
        Line::from(format!("Running: {running}")),
        Line::from(""),
        Line::from(Span::styled(
            "Open Packages to manage services.",
            Style::default().fg(theme.muted),
        )),
    ]
}

fn build_config_summary(config: Option<&Config>, theme: &Theme) -> Vec<Line<'static>> {
    let Some(config) = config else {
        return vec![Line::from(Span::styled(
            "Loading configuration...",
            Style::default().fg(theme.muted),
        ))];
    };
    let server = if config.server_url.trim().is_empty() {
        "(local)"
    } else {
        config.server_url.as_str()
    };
    let auto_start = if config.auto_start_docker {
        "enabled"
    } else {
        "disabled"
    };
    let onboarding = if config.onboarding_completed {
        "complete"
    } else {
        "incomplete"
    };
    vec![
        Line::from(format!("Server: {server}")),
        Line::from(format!(
            "Capabilities: {}",
            if config.capabilities.is_empty() {
                "none".to_string()
            } else {
                config.capabilities.join(", ")
            }
        )),
        Line::from(format!("Auto start Docker: {auto_start}")),
        Line::from(format!("Onboarding: {onboarding}")),
    ]
}

fn render_packages(frame: &mut Frame, area: Rect, app: &mut App, theme: &Theme) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(8), Constraint::Length(7)])
        .split(area);

    if app.data.packages.is_empty() {
        let empty = Paragraph::new("No packages installed.")
            .style(Style::default().fg(theme.muted))
            .block(Block::new().borders(Borders::ALL).title("Packages"));
        frame.render_widget(empty, layout[0]);
    } else {
        let rows: Vec<Row> = app
            .data
            .packages
            .iter()
            .map(|pkg| {
                let status = app
                    .data
                    .package_states
                    .get(pkg.name())
                    .map(|state| state.runtime);
                let (label, color) = runtime_label(status, theme);
                Row::new(vec![
                    Cell::from(pkg.name().to_string()),
                    Cell::from(pkg.network_name().to_string()),
                    Cell::from(Span::styled(label, Style::default().fg(color))),
                    Cell::from(truncate(pkg.description(), 46)),
                ])
            })
            .collect();

        let table = Table::new(
            rows,
            [
                Constraint::Length(16),
                Constraint::Length(14),
                Constraint::Length(12),
                Constraint::Min(10),
            ],
        )
        .header(
            Row::new(vec!["Name", "Network", "Status", "Description"]).style(
                Style::default()
                    .fg(theme.accent_soft)
                    .add_modifier(Modifier::BOLD),
            ),
        )
        .block(
            Block::new()
                .borders(Borders::ALL)
                .title("Installed Packages"),
        )
        .row_highlight_style(Style::default().fg(theme.text).bg(Color::Rgb(40, 60, 80)));

        frame.render_stateful_widget(table, layout[0], &mut app.package_state);
    }

    let detail = build_package_detail(app, theme);
    let detail = Paragraph::new(detail)
        .wrap(Wrap { trim: true })
        .block(Block::new().borders(Borders::ALL).title("Details"));
    frame.render_widget(detail, layout[1]);
}

fn build_package_detail(app: &App, theme: &Theme) -> Vec<Line<'static>> {
    let Some(index) = app.package_state.selected() else {
        return vec![Line::from(Span::styled(
            "Select a package to see details.",
            Style::default().fg(theme.muted),
        ))];
    };
    let Some(package) = app.data.packages.get(index) else {
        return vec![Line::from(Span::styled(
            "Select a package to see details.",
            Style::default().fg(theme.muted),
        ))];
    };
    let state = app.data.package_states.get(package.name());
    let status = state.map(|state| state.runtime);
    let (label, color) = runtime_label(status, theme);
    let mut lines = vec![
        Line::from(vec![
            Span::raw("Status: "),
            Span::styled(label, Style::default().fg(color)),
        ]),
        Line::from(format!("Network: {}", package.network_name())),
        Line::from(format!("Description: {}", package.description())),
    ];
    if let Some(state) = state
        && !state.missing_containers.is_empty()
    {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Missing containers:",
            Style::default().fg(theme.warning),
        )));
        for entry in &state.missing_containers {
            lines.push(Line::from(format!("- {entry}")));
        }
    }
    lines
}

fn render_catalog(frame: &mut Frame, area: Rect, app: &mut App, theme: &Theme) {
    if app.data.catalog.is_empty() {
        let empty = Paragraph::new("Catalog unavailable.")
            .style(Style::default().fg(theme.muted))
            .block(Block::new().borders(Borders::ALL).title("Catalog"));
        frame.render_widget(empty, area);
        return;
    }

    let installed: HashSet<&str> = app.data.packages.iter().map(|pkg| pkg.name()).collect();

    let rows: Vec<Row> = app
        .data
        .catalog
        .iter()
        .map(|pkg| {
            let status = if installed.contains(pkg.name()) {
                Span::styled("installed", Style::default().fg(theme.success))
            } else {
                Span::styled("available", Style::default().fg(theme.accent))
            };
            Row::new(vec![
                Cell::from(pkg.name().to_string()),
                Cell::from(pkg.network_name().to_string()),
                Cell::from(status),
                Cell::from(truncate(pkg.description(), 50)),
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(16),
            Constraint::Length(14),
            Constraint::Length(12),
            Constraint::Min(10),
        ],
    )
    .header(
        Row::new(vec!["Name", "Network", "Status", "Description"]).style(
            Style::default()
                .fg(theme.accent_soft)
                .add_modifier(Modifier::BOLD),
        ),
    )
    .block(Block::new().borders(Borders::ALL).title("Catalog"))
    .row_highlight_style(Style::default().fg(theme.text).bg(Color::Rgb(40, 60, 80)));

    frame.render_stateful_widget(table, area, &mut app.catalog_state);
}

fn render_web(frame: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let lines = build_web_lines(app, theme);
    let paragraph = Paragraph::new(lines)
        .wrap(Wrap { trim: true })
        .block(Block::new().borders(Borders::ALL).title("Web Service"));
    frame.render_widget(paragraph, area);
}

fn build_web_lines(app: &App, theme: &Theme) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    match app.data.web_status.as_ref() {
        Some(WebServiceStatus::Started { pid, port })
        | Some(WebServiceStatus::AlreadyRunning { pid, port }) => {
            lines.push(Line::from(vec![
                Span::raw("Status: "),
                Span::styled("running", Style::default().fg(theme.success)),
            ]));
            lines.push(Line::from(format!("Port: {port}")));
            lines.push(Line::from(format!("PID: {pid}")));
        }
        Some(WebServiceStatus::Stopped { pid, port }) => {
            lines.push(Line::from(vec![
                Span::raw("Status: "),
                Span::styled("stopped", Style::default().fg(theme.warning)),
            ]));
            lines.push(Line::from(format!("Last port: {port}")));
            lines.push(Line::from(format!("Last PID: {pid}")));
        }
        Some(WebServiceStatus::NotRunning) => {
            lines.push(Line::from(vec![
                Span::raw("Status: "),
                Span::styled("not running", Style::default().fg(theme.muted)),
            ]));
        }
        None => {
            lines.push(Line::from(Span::styled(
                "Web service status unavailable.",
                Style::default().fg(theme.muted),
            )));
        }
    }

    if let Some(path) = &app.data.web_log_path {
        lines.push(Line::from(""));
        lines.push(Line::from(format!("Logs: {}", path.display())));
    }
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Use s to start/stop, R to restart.",
        Style::default().fg(theme.muted),
    )));
    lines
}

fn render_docker(frame: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let mut lines = Vec::new();
    let docker_running = app
        .data
        .operational_state
        .as_ref()
        .map(|state| state.docker_running);
    match docker_running {
        Some(true) => lines.push(Line::from(vec![
            Span::raw("Docker: "),
            Span::styled("running", Style::default().fg(theme.success)),
        ])),
        Some(false) => lines.push(Line::from(vec![
            Span::raw("Docker: "),
            Span::styled("offline", Style::default().fg(theme.danger)),
        ])),
        None => lines.push(Line::from(Span::styled(
            "Docker status unavailable.",
            Style::default().fg(theme.muted),
        ))),
    };

    if let Some(config) = &app.data.config {
        let auto_start = if config.auto_start_docker {
            "enabled"
        } else {
            "disabled"
        };
        lines.push(Line::from(format!("Auto start: {auto_start}")));
    }
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Press s to start Docker if needed.",
        Style::default().fg(theme.muted),
    )));

    let paragraph = Paragraph::new(lines)
        .wrap(Wrap { trim: true })
        .block(Block::new().borders(Borders::ALL).title("Docker"));
    frame.render_widget(paragraph, area);
}

fn render_config(frame: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let lines = build_config_lines(app.data.config.as_ref(), theme);
    let paragraph = Paragraph::new(lines)
        .wrap(Wrap { trim: true })
        .block(Block::new().borders(Borders::ALL).title("Configuration"));
    frame.render_widget(paragraph, area);
}

fn build_config_lines(config: Option<&Config>, theme: &Theme) -> Vec<Line<'static>> {
    let Some(config) = config else {
        return vec![Line::from(Span::styled(
            "Configuration unavailable.",
            Style::default().fg(theme.muted),
        ))];
    };
    let server = if config.server_url.trim().is_empty() {
        "(local)"
    } else {
        config.server_url.as_str()
    };
    let mut lines = vec![
        Line::from(format!("Server URL: {server}")),
        Line::from(format!(
            "Onboarding completed: {}",
            if config.onboarding_completed {
                "yes"
            } else {
                "no"
            }
        )),
        Line::from(format!(
            "Auto start Docker: {}",
            if config.auto_start_docker {
                "enabled"
            } else {
                "disabled"
            }
        )),
    ];
    lines.push(Line::from(""));
    lines.push(Line::from("Capabilities:"));
    if config.capabilities.is_empty() {
        lines.push(Line::from(Span::styled(
            "None",
            Style::default().fg(theme.muted),
        )));
    } else {
        for capability in &config.capabilities {
            lines.push(Line::from(format!("- {capability}")));
        }
    }
    lines
}

fn render_system(frame: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let lines = build_system_lines(app.data.system_info.as_ref(), theme);
    let paragraph = Paragraph::new(lines)
        .wrap(Wrap { trim: true })
        .block(Block::new().borders(Borders::ALL).title("System"));
    frame.render_widget(paragraph, area);
}

fn build_system_lines(info: Option<&SystemInfo>, theme: &Theme) -> Vec<Line<'static>> {
    let Some(info) = info else {
        return vec![Line::from(Span::styled(
            "System info unavailable.",
            Style::default().fg(theme.muted),
        ))];
    };

    let mut lines = vec![
        Line::from(format!(
            "Processor: {} ({} cores, {:.2} GHz)",
            info.processor.name, info.processor.cores, info.processor.frequency_ghz
        )),
        Line::from(format!("Memory: {}", info.memory.total_display)),
        Line::from("Storage:"),
    ];
    if info.storage.disks.is_empty() {
        lines.push(Line::from(Span::styled(
            "No disks detected.",
            Style::default().fg(theme.muted),
        )));
    } else {
        for disk in &info.storage.disks {
            lines.push(Line::from(format!(
                "- {} mounted on {}",
                disk.name, disk.mount_point
            )));
            lines.push(Line::from(format!(
                "  Total: {} | Available: {}",
                disk.total_display, disk.available_display
            )));
        }
    }
    lines
}

fn render_help(frame: &mut Frame, area: Rect, theme: &Theme) {
    let lines = vec![
        Line::from(Span::styled(
            "Key bindings",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from("tab / shift+tab  -  switch sections"),
        Line::from("up / down        -  move selection"),
        Line::from("enter            -  primary action"),
        Line::from("r                -  refresh"),
        Line::from("q / esc          -  quit"),
        Line::from(""),
        Line::from("Packages: enter start/stop, d delete, D delete+images"),
        Line::from("Catalog: enter install, ethereum prompts for network"),
        Line::from("Web: s start/stop, R restart"),
        Line::from("Docker: s start"),
    ];
    let paragraph = Paragraph::new(lines)
        .block(Block::new().borders(Borders::ALL).title("Help"))
        .alignment(Alignment::Left);
    frame.render_widget(paragraph, area);
}

fn render_footer(frame: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let hint = app.current_section().hint();
    let hint_line = Line::from(vec![
        Span::styled(hint, Style::default().fg(theme.muted)),
        Span::raw("  |  "),
        Span::styled("q to quit", Style::default().fg(theme.muted)),
    ]);
    let status_style = match app.status.level {
        StatusLevel::Info => Style::default().fg(theme.text),
        StatusLevel::Success => Style::default().fg(theme.success),
        StatusLevel::Warn => Style::default().fg(theme.warning),
        StatusLevel::Error => Style::default().fg(theme.danger),
    };
    let status_line = Line::from(Span::styled(&app.status.text, status_style));
    let paragraph =
        Paragraph::new(vec![hint_line, status_line]).block(Block::new().borders(Borders::TOP));
    frame.render_widget(paragraph, area);
}

fn render_modal(frame: &mut Frame, modal: &Modal, theme: &Theme) {
    let area = centered_rect(60, 40, frame.area());
    frame.render_widget(Clear, area);

    match modal {
        Modal::ConfirmDelete {
            package_name,
            include_images,
        } => {
            let warning = if *include_images {
                "This will remove Docker images too."
            } else {
                "Images will be kept."
            };
            let lines = vec![
                Line::from(Span::styled(
                    "Confirm delete",
                    Style::default()
                        .fg(theme.warning)
                        .add_modifier(Modifier::BOLD),
                )),
                Line::from(""),
                Line::from(format!("Package: {package_name}")),
                Line::from(warning),
                Line::from(""),
                Line::from("Press y to delete, n to cancel."),
            ];
            let paragraph = Paragraph::new(lines)
                .block(Block::new().borders(Borders::ALL).title("Confirm"))
                .alignment(Alignment::Left);
            frame.render_widget(paragraph, area);
        }
        Modal::SelectNetwork {
            networks, selected, ..
        } => {
            let mut items = Vec::new();
            for (index, network) in networks.iter().enumerate() {
                let style = if index == *selected {
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(theme.text)
                };
                items.push(ListItem::new(Line::from(Span::styled(network, style))));
            }
            let list = List::new(items)
                .block(Block::new().borders(Borders::ALL).title("Select network"))
                .highlight_style(Style::default().fg(theme.accent));
            frame.render_widget(list, area);
        }
    }

    let subtitle = Paragraph::new("esc to cancel")
        .style(Style::default().fg(theme.muted))
        .alignment(Alignment::Right);
    let subtitle_area = Rect {
        x: area.x,
        y: area.y + area.height.saturating_sub(1),
        width: area.width,
        height: 1,
    };
    frame.render_widget(subtitle, subtitle_area);
}

fn centered_rect(percent_x: u16, percent_y: u16, rect: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(rect);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

fn runtime_label(status: Option<RuntimeStatus>, theme: &Theme) -> (&'static str, Color) {
    match status {
        Some(RuntimeStatus::Running) => ("running", theme.success),
        Some(RuntimeStatus::PartiallyRunning) => ("partial", theme.warning),
        Some(RuntimeStatus::NotRunning) => ("stopped", theme.danger),
        None => ("unknown", theme.muted),
    }
}

fn truncate(input: &str, max: usize) -> String {
    if input.len() <= max {
        return input.to_string();
    }
    if max <= 3 {
        return ".".repeat(max);
    }
    let mut out = input
        .chars()
        .take(max.saturating_sub(3))
        .collect::<String>();
    out.push_str("...");
    out
}

fn format_duration(duration: Duration) -> String {
    let secs = duration.as_secs();
    if secs < 60 {
        format!("{secs}s")
    } else {
        let minutes = secs / 60;
        let seconds = secs % 60;
        format!("{minutes}m {seconds}s")
    }
}

fn move_selection_up(state: &mut TableState) {
    if let Some(selected) = state.selected() {
        let next = selected.saturating_sub(1);
        state.select(Some(next));
    }
}

fn move_selection_down(state: &mut TableState, len: usize) {
    if len == 0 {
        state.select(None);
        return;
    }
    let next = match state.selected() {
        Some(selected) if selected + 1 < len => selected + 1,
        Some(selected) => selected,
        None => 0,
    };
    state.select(Some(next));
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;
    use serde_json::json;

    fn fake_package(name: &str) -> Package {
        serde_json::from_value(json!({
            "name": name,
            "description": "Test package",
            "networkName": "testnet",
            "containers": [],
            "defaultConfig": { "values": {} }
        }))
        .expect("package json should deserialize")
    }

    #[test]
    fn restart_port_from_status_prefers_existing_port() {
        let started = WebServiceStatus::Started {
            pid: 42,
            port: 8080,
        };
        let running = WebServiceStatus::AlreadyRunning {
            pid: 12,
            port: 9090,
        };
        let stopped = WebServiceStatus::Stopped { pid: 7, port: 7070 };
        let not_running = WebServiceStatus::NotRunning;

        assert_eq!(restart_port_from_status(Some(&started)), Some(8080));
        assert_eq!(restart_port_from_status(Some(&running)), Some(9090));
        assert_eq!(restart_port_from_status(Some(&stopped)), Some(7070));
        assert_eq!(restart_port_from_status(Some(&not_running)), None);
        assert_eq!(restart_port_from_status(None), None);
    }

    #[test]
    fn packages_table_persists_scroll_offset() -> Result<()> {
        let mut app = App::new();
        app.selected_section = app
            .sections
            .iter()
            .position(|section| *section == Section::Packages)
            .unwrap_or(0);
        app.data.packages = (0..30)
            .map(|idx| fake_package(&format!("pkg-{idx}")))
            .collect();
        app.package_state.select(Some(25));

        let backend = TestBackend::new(MIN_WIDTH, MIN_HEIGHT);
        let mut terminal = Terminal::new(backend)?;
        terminal.draw(|frame| render(frame, &mut app))?;

        assert!(
            app.package_state.offset() > 0,
            "expected table offset to advance when selection is off-screen"
        );

        Ok(())
    }

    #[test]
    fn catalog_table_persists_scroll_offset() -> Result<()> {
        let mut app = App::new();
        app.selected_section = app
            .sections
            .iter()
            .position(|section| *section == Section::Catalog)
            .unwrap_or(0);
        app.data.catalog = (0..30)
            .map(|idx| fake_package(&format!("cat-{idx}")))
            .collect();
        app.catalog_state.select(Some(25));

        let backend = TestBackend::new(MIN_WIDTH, MIN_HEIGHT);
        let mut terminal = Terminal::new(backend)?;
        terminal.draw(|frame| render(frame, &mut app))?;

        assert!(
            app.catalog_state.offset() > 0,
            "expected catalog offset to advance when selection is off-screen"
        );

        Ok(())
    }
}
