mod commands;

use clap::{Parser, Subcommand};
use eyre::Result;
use kittynode_core::api::CreateDepositDataParams;
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    version,
    about = "Manage your Kittynode installation from the terminal",
    long_about = "Use kittynode commands to install packages, inspect configuration, and work with validator tooling."
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
    #[command(about = "Validator tooling for key management and deposits")]
    Validator {
        #[command(subcommand)]
        command: ValidatorCommands,
    },
    #[command(about = "Control the Kittynode web service")]
    Web {
        #[command(subcommand)]
        command: WebCommands,
    },
}

#[derive(Subcommand)]
enum PackageCommands {
    #[command(name = "list", about = "List packages available to install")]
    List,
    #[command(
        name = "installed",
        about = "Show packages currently installed on this Kittynode"
    )]
    Installed,
    #[command(about = "Install a package from the Kittynode registry")]
    Install {
        #[arg(value_name = "PACKAGE_NAME", help = "Name of the package to install")]
        name: String,
    },
    #[command(about = "Uninstall a package and optionally remove its Docker images")]
    Uninstall {
        #[arg(value_name = "PACKAGE_NAME", help = "Name of the package to uninstall")]
        name: String,
        #[arg(long = "include-images", help = "Remove associated Docker images")]
        include_images: bool,
    },
    #[command(about = "Stop all containers that belong to a package")]
    Stop {
        #[arg(value_name = "PACKAGE_NAME", help = "Name of the package to stop")]
        name: String,
    },
    #[command(about = "Resume containers for a previously stopped package")]
    Resume {
        #[arg(value_name = "PACKAGE_NAME", help = "Name of the package to resume")]
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
    #[command(
        name = "generate-keys",
        about = "Create a new validator keypair on disk"
    )]
    GenerateKeys {
        #[arg(
            long = "output-dir",
            value_name = "PATH",
            help = "Folder where key files should be written"
        )]
        output_dir: PathBuf,
        #[arg(
            long = "entropy",
            value_name = "STRING",
            help = "Entropy string used to derive the keys"
        )]
        entropy: String,
        #[arg(
            long = "file-name",
            value_name = "NAME",
            help = "Optional custom base name for the key files"
        )]
        file_name: Option<String>,
        #[arg(
            long = "overwrite",
            help = "Replace existing key files if they already exist"
        )]
        overwrite: bool,
    },
    #[command(
        name = "create-deposit-data",
        about = "Build deposit data for an existing validator key"
    )]
    CreateDepositData {
        #[arg(
            long = "key",
            value_name = "PATH",
            help = "Path to the validator key file"
        )]
        key_path: PathBuf,
        #[arg(
            long = "output",
            value_name = "PATH",
            help = "Destination file for the deposit data JSON"
        )]
        output_path: PathBuf,
        #[arg(
            long = "withdrawal-credentials",
            value_name = "HEX",
            help = "Hex-encoded withdrawal credentials"
        )]
        withdrawal_credentials: String,
        #[arg(
            long = "amount-gwei",
            default_value_t = 32_000_000_000,
            help = "Deposit amount in gwei (defaults to 32 ETH)"
        )]
        amount_gwei: u64,
        #[arg(
            long = "fork-version",
            default_value = "00000000",
            help = "Fork version for the target network in hex"
        )]
        fork_version: String,
        #[arg(
            long = "genesis-root",
            value_name = "HEX",
            help = "Genesis validators root for the target network"
        )]
        genesis_root: String,
        #[arg(
            long = "network",
            value_name = "NAME",
            help = "Optional network name written to the deposit data file"
        )]
        network: Option<String>,
        #[arg(
            long = "overwrite",
            help = "Replace the output file if it already exists"
        )]
        overwrite: bool,
    },
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

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .init();
    let cli = Cli::parse();

    match cli.command {
        Commands::Package { command } => match command {
            PackageCommands::List => commands::get_packages().await?,
            PackageCommands::Installed => commands::get_installed_packages().await?,
            PackageCommands::Install { name } => commands::install_package(name).await?,
            PackageCommands::Uninstall {
                name,
                include_images,
            } => commands::delete_package(name, include_images).await?,
            PackageCommands::Stop { name } => commands::stop_package(name).await?,
            PackageCommands::Resume { name } => commands::resume_package(name).await?,
            PackageCommands::Config { command } => match command {
                PackageConfigCommands::Show { name } => commands::get_package_config(name).await?,
                PackageConfigCommands::Set { name, values } => {
                    commands::update_package_config(name, values).await?
                }
            },
        },
        Commands::Config { command } => match command {
            ConfigCommands::Show => commands::get_config()?,
            ConfigCommands::Init => commands::init_kittynode()?,
            ConfigCommands::Delete => commands::delete_kittynode()?,
        },
        Commands::Capability { command } => match command {
            CapabilityCommands::List => commands::get_capabilities()?,
            CapabilityCommands::Add { name } => commands::add_capability(name)?,
            CapabilityCommands::Remove { name } => commands::remove_capability(name)?,
        },
        Commands::System { command } => match command {
            SystemCommands::Info => commands::system_info().await?,
            SystemCommands::State => commands::get_operational_state().await?,
        },
        Commands::Docker { command } => match command {
            DockerCommands::Status => commands::is_docker_running().await?,
            DockerCommands::Start => commands::start_docker_if_needed().await?,
        },
        Commands::Container { command } => match command {
            ContainerCommands::Logs { container, tail } => {
                commands::get_container_logs(container, tail).await?
            }
        },
        Commands::Validator { command } => match command {
            ValidatorCommands::GenerateKeys {
                output_dir,
                entropy,
                file_name,
                overwrite,
            } => commands::validator_generate_keys(output_dir, file_name, entropy, overwrite)?,
            ValidatorCommands::CreateDepositData {
                key_path,
                output_path,
                withdrawal_credentials,
                amount_gwei,
                fork_version,
                genesis_root,
                network,
                overwrite,
            } => {
                let params = CreateDepositDataParams::from_hex_inputs(
                    key_path,
                    output_path,
                    withdrawal_credentials,
                    amount_gwei,
                    &fork_version,
                    &genesis_root,
                    overwrite,
                )?
                .with_network_name(network);

                commands::validator_create_deposit_data(params)?
            }
        },
        Commands::Web { command } => match command {
            WebCommands::Start { port } => commands::start_web_service(port)?,
            WebCommands::Stop => commands::stop_web_service()?,
            WebCommands::Status => commands::web_status()?,
            WebCommands::Logs { follow, tail } => commands::web_logs(follow, tail)?,
            WebCommands::RunInternal {
                port,
                service_token,
            } => commands::run_web_service(port, service_token).await?,
        },
    }

    Ok(())
}
