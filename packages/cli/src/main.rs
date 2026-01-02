mod commands;
mod update_checker;

use clap::{Parser, Subcommand, ValueEnum};
use eyre::Result;
use std::sync::atomic::{AtomicBool, Ordering};
use tracing_subscriber::fmt::MakeWriter;

#[derive(Parser)]
#[command(
    version,
    about = "Manage your Kittynode installation from the terminal"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Manage packages available to Kittynode")]
    Package {
        #[command(subcommand)]
        command: PackageCommands,
    },
    #[command(about = "Inspect or update Kittynode configuration")]
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },
    #[command(about = "Manage capability flags on this Kittynode")]
    Capability {
        #[command(subcommand)]
        command: CapabilityCommands,
    },
    #[command(about = "Inspect system diagnostics and environment")]
    System {
        #[command(subcommand)]
        command: SystemCommands,
    },
    #[command(about = "Control Docker services used by Kittynode")]
    Docker {
        #[command(subcommand)]
        command: DockerCommands,
    },
    #[command(about = "Inspect managed containers")]
    Container {
        #[command(subcommand)]
        command: ContainerCommands,
    },
    #[command(about = "Manage validator key material")]
    Validator {
        #[command(subcommand)]
        command: ValidatorCommands,
    },
    #[command(about = "Control the Kittynode web service")]
    Web {
        #[command(subcommand)]
        command: WebCommands,
    },
    #[command(about = "Update Kittynode to the latest release")]
    Update,
}

#[derive(Copy, Clone, Debug, ValueEnum)]
enum EthereumNetwork {
    Mainnet,
    Hoodi,
    Sepolia,
    Ephemery,
}

impl EthereumNetwork {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Mainnet => "mainnet",
            Self::Hoodi => "hoodi",
            Self::Sepolia => "sepolia",
            Self::Ephemery => "ephemery",
        }
    }
}

#[derive(Subcommand)]
enum PackageCommands {
    #[command(name = "catalog", about = "List packages from the catalog")]
    Catalog,
    #[command(name = "list", about = "List packages currently installed")]
    List,
    #[command(about = "Install a package from the catalog")]
    Install {
        #[arg(value_name = "PACKAGE_NAME", help = "Name of the package to install")]
        name: String,
        #[arg(
            long = "network",
            value_name = "NETWORK",
            help = "Select the network for supported packages"
        )]
        network: Option<EthereumNetwork>,
    },
    #[command(about = "Delete a package and optionally remove its Docker images")]
    Delete {
        #[arg(value_name = "PACKAGE_NAME", help = "Name of the package to delete")]
        name: String,
        #[arg(long = "include-images", help = "Remove associated Docker images")]
        include_images: bool,
    },
    #[command(about = "Stop all containers that belong to a package")]
    Stop {
        #[arg(value_name = "PACKAGE_NAME", help = "Name of the package to stop")]
        name: String,
    },
    #[command(about = "Start containers for a previously stopped package")]
    Start {
        #[arg(value_name = "PACKAGE_NAME", help = "Name of the package to start")]
        name: String,
    },
    #[command(about = "Manage package-specific configuration overrides")]
    Config {
        #[command(subcommand)]
        command: PackageConfigCommands,
    },
}

#[derive(Subcommand)]
enum PackageConfigCommands {
    #[command(
        name = "show",
        about = "Show configuration overrides applied to a package"
    )]
    Show {
        #[arg(
            value_name = "PACKAGE_NAME",
            help = "Package whose overrides should be shown"
        )]
        name: String,
    },
    #[command(name = "set", about = "Set configuration overrides for a package")]
    Set {
        #[arg(
            value_name = "PACKAGE_NAME",
            help = "Package whose overrides should be updated"
        )]
        name: String,
        #[arg(
            long = "value",
            value_name = "KEY=VALUE",
            value_parser = parse_key_val,
            num_args = 0..
        )]
        values: Vec<(String, String)>,
    },
}

#[derive(Subcommand)]
enum ConfigCommands {
    #[command(name = "show", about = "Print global Kittynode configuration values")]
    Show,
    #[command(
        name = "init",
        about = "Initialize Kittynode data directories and defaults"
    )]
    Init,
    #[command(
        name = "delete",
        about = "Delete local Kittynode data and configuration"
    )]
    Delete,
}

#[derive(Subcommand)]
enum CapabilityCommands {
    #[command(name = "list", about = "List capabilities enabled on this Kittynode")]
    List,
    #[command(about = "Enable a capability in the local Kittynode config")]
    Add {
        #[arg(value_name = "CAPABILITY", help = "Capability identifier to enable")]
        name: String,
    },
    #[command(about = "Disable a capability in the local Kittynode config")]
    Remove {
        #[arg(value_name = "CAPABILITY", help = "Capability identifier to disable")]
        name: String,
    },
}

#[derive(Subcommand)]
enum SystemCommands {
    #[command(
        name = "info",
        about = "Display hardware and OS details used by Kittynode"
    )]
    Info,
    #[command(
        name = "state",
        about = "Show overall operational status and readiness flags"
    )]
    State,
}

#[derive(Subcommand)]
enum DockerCommands {
    #[command(
        name = "status",
        about = "Check whether Docker is reachable from Kittynode"
    )]
    Status,
    #[command(name = "start", about = "Start Docker if it is not already running")]
    Start,
}

#[derive(Subcommand)]
enum ContainerCommands {
    #[command(name = "logs", about = "Show recent logs from a managed container")]
    Logs {
        #[arg(value_name = "CONTAINER_NAME", help = "Managed container to inspect")]
        container: String,
        #[arg(
            long = "tail",
            value_name = "LINES",
            help = "Number of log lines to fetch"
        )]
        tail: Option<usize>,
    },
}

#[derive(Subcommand)]
enum ValidatorCommands {
    #[command(name = "keygen", about = "Generate Ethereum validator keys")]
    Keygen,
    #[command(name = "init", about = "Initialize the validator setup workflow")]
    Init,
}

#[derive(Subcommand)]
enum WebCommands {
    #[command(name = "start", about = "Start the Kittynode web service")]
    Start {
        #[arg(
            long = "port",
            value_name = "PORT",
            help = "Port to bind the Kittynode web service"
        )]
        port: Option<u16>,
    },
    #[command(name = "restart", about = "Restart the Kittynode web service")]
    Restart {
        #[arg(
            long = "port",
            value_name = "PORT",
            help = "Port to bind the Kittynode web service"
        )]
        port: Option<u16>,
    },
    #[command(name = "stop", about = "Stop the Kittynode web service")]
    Stop,
    #[command(name = "status", about = "Show Kittynode web service status")]
    Status,
    #[command(name = "logs", about = "Stream logs from the Kittynode web service")]
    Logs {
        #[arg(
            long = "follow",
            short = 'f',
            help = "Follow log output until interrupted"
        )]
        follow: bool,
        #[arg(
            long = "tail",
            value_name = "LINES",
            help = "Number of lines to show from the end of the log"
        )]
        tail: Option<usize>,
    },
    #[command(name = "__internal-run", hide = true)]
    RunInternal {
        #[arg(
            long = "port",
            value_name = "PORT",
            help = "Port to bind the Kittynode web service"
        )]
        port: Option<u16>,
        #[arg(
            long = "service-token",
            value_name = "TOKEN",
            hide = true,
            help = "Internal token used to bind the web host to the parent process"
        )]
        service_token: Option<String>,
    },
}

impl Commands {
    async fn execute(self) -> Result<()> {
        match self {
            Commands::Package { command } => command.execute().await,
            Commands::Config { command } => command.execute(),
            Commands::Capability { command } => command.execute(),
            Commands::System { command } => command.execute().await,
            Commands::Docker { command } => command.execute().await,
            Commands::Container { command } => command.execute().await,
            Commands::Validator { command } => command.execute().await,
            Commands::Web { command } => command.execute().await,
            Commands::Update => commands::run_updater(),
        }
    }
}

impl PackageCommands {
    async fn execute(self) -> Result<()> {
        match self {
            PackageCommands::Catalog => commands::get_package_catalog().await,
            PackageCommands::List => commands::get_installed_packages().await,
            PackageCommands::Install { name, network } => {
                commands::install_package(name, network.map(EthereumNetwork::as_str)).await
            }
            PackageCommands::Delete {
                name,
                include_images,
            } => commands::delete_package(name, include_images).await,
            PackageCommands::Stop { name } => commands::stop_package(name).await,
            PackageCommands::Start { name } => commands::start_package(name).await,
            PackageCommands::Config { command } => command.execute().await,
        }
    }
}

impl PackageConfigCommands {
    async fn execute(self) -> Result<()> {
        match self {
            PackageConfigCommands::Show { name } => commands::get_package_config(name).await,
            PackageConfigCommands::Set { name, values } => {
                commands::update_package_config(name, values).await
            }
        }
    }
}

impl ConfigCommands {
    fn execute(self) -> Result<()> {
        match self {
            ConfigCommands::Show => commands::get_config(),
            ConfigCommands::Init => commands::init_kittynode(),
            ConfigCommands::Delete => commands::delete_kittynode(),
        }
    }
}

impl CapabilityCommands {
    fn execute(self) -> Result<()> {
        match self {
            CapabilityCommands::List => commands::get_capabilities(),
            CapabilityCommands::Add { name } => commands::add_capability(name),
            CapabilityCommands::Remove { name } => commands::remove_capability(name),
        }
    }
}

impl SystemCommands {
    async fn execute(self) -> Result<()> {
        match self {
            SystemCommands::Info => commands::system_info().await,
            SystemCommands::State => commands::get_operational_state().await,
        }
    }
}

impl DockerCommands {
    async fn execute(self) -> Result<()> {
        match self {
            DockerCommands::Status => commands::is_docker_running().await,
            DockerCommands::Start => commands::start_docker_if_needed().await,
        }
    }
}

impl ContainerCommands {
    async fn execute(self) -> Result<()> {
        match self {
            ContainerCommands::Logs { container, tail } => {
                commands::get_container_logs(container, tail).await
            }
        }
    }
}

impl ValidatorCommands {
    async fn execute(self) -> Result<()> {
        match self {
            ValidatorCommands::Keygen => commands::validator::keygen(None).map(|_| ()),
            ValidatorCommands::Init => commands::validator::init().await,
        }
    }
}

impl WebCommands {
    async fn execute(self) -> Result<()> {
        match self {
            WebCommands::Start { port } => commands::start_web_service(port),
            WebCommands::Restart { port } => commands::restart_web_service(port),
            WebCommands::Stop => commands::stop_web_service(),
            WebCommands::Status => commands::web_status(),
            WebCommands::Logs { follow, tail } => commands::web_logs(follow, tail),
            WebCommands::RunInternal {
                port,
                service_token,
            } => commands::run_web_service(port, service_token).await,
        }
    }
}

fn parse_key_val(s: &str) -> Result<(String, String), String> {
    let position = s
        .find('=')
        .ok_or_else(|| "expected KEY=VALUE".to_string())?;
    let key = s[..position].trim();
    let value = s[position + 1..].trim();
    if key.is_empty() {
        return Err("key cannot be empty".to_string());
    }
    Ok((key.to_string(), value.to_string()))
}

mod log_control {
    use super::*;

    static QUIET: AtomicBool = AtomicBool::new(false);

    pub struct LogGuard {
        prev: bool,
    }

    impl Drop for LogGuard {
        fn drop(&mut self) {
            QUIET.store(self.prev, Ordering::SeqCst);
        }
    }

    struct ToggleWriter;
    impl<'a> MakeWriter<'a> for ToggleWriter {
        type Writer = Box<dyn std::io::Write + Send>;
        fn make_writer(&'a self) -> Self::Writer {
            if QUIET.load(Ordering::SeqCst) {
                Box::new(std::io::sink())
            } else {
                Box::new(std::io::stderr())
            }
        }
    }

    pub fn init_logging() {
        tracing_subscriber::fmt().with_writer(ToggleWriter).init();
    }

    pub fn mute_guard() -> LogGuard {
        let prev = QUIET.swap(true, Ordering::SeqCst);
        LogGuard { prev }
    }

    pub fn enable_guard() -> LogGuard {
        let prev = QUIET.swap(false, Ordering::SeqCst);
        LogGuard { prev }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    log_control::init_logging();

    // Skip the update check when the CLI itself is being updated.
    let skip_update_check = std::env::var_os("KITTYNODE_SKIP_UPDATE_CHECK").is_some();
    if !skip_update_check && std::env::args().nth(1).as_deref() != Some("update") {
        update_checker::check_and_print_update().await;
    }

    let cli = Cli::parse();

    cli.command.execute().await
}

#[cfg(test)]
mod tests {
    use super::parse_key_val;

    #[test]
    fn parse_key_val_returns_trimmed_pair() {
        let result = parse_key_val("FOO = bar").expect("expected key=val to parse");
        assert_eq!(result, ("FOO".to_string(), "bar".to_string()));
    }

    #[test]
    fn parse_key_val_handles_values_with_equals() {
        let result = parse_key_val("TOKEN=abc=123").expect("expected parser to keep tail");
        assert_eq!(result, ("TOKEN".to_string(), "abc=123".to_string()));
    }

    #[test]
    fn parse_key_val_missing_delimiter_errors() {
        let error = parse_key_val("NOVALUE").expect_err("missing '=' should error");
        assert_eq!(error, "expected KEY=VALUE");
    }

    #[test]
    fn parse_key_val_rejects_empty_key() {
        let error = parse_key_val(" =value").expect_err("empty key should error");
        assert_eq!(error, "key cannot be empty");
    }
}
