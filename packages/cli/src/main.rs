mod commands;

use clap::{Parser, Subcommand};
use eyre::Result;
use std::path::PathBuf;

#[derive(Parser)]
#[command(about, version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    GetPackages,
    GetInstalledPackages,
    InstallPackage {
        #[arg(value_name = "PACKAGE_NAME")]
        name: String,
    },
    DeletePackage {
        #[arg(value_name = "PACKAGE_NAME")]
        name: String,
        #[arg(long = "include-images", help = "Remove associated Docker images")]
        include_images: bool,
    },
    SystemInfo,
    GetContainerLogs {
        #[arg(value_name = "CONTAINER_NAME")]
        container: String,
        #[arg(
            long = "tail",
            value_name = "LINES",
            help = "Number of log lines to fetch"
        )]
        tail: Option<usize>,
    },
    GetConfig,
    GetPackageConfig {
        #[arg(value_name = "PACKAGE_NAME")]
        name: String,
    },
    UpdatePackageConfig {
        #[arg(value_name = "PACKAGE_NAME")]
        name: String,
        #[arg(
            long = "value",
            value_name = "KEY=VALUE",
            value_parser = parse_key_val,
            num_args = 0..
        )]
        values: Vec<(String, String)>,
    },
    GetCapabilities,
    AddCapability {
        #[arg(value_name = "CAPABILITY")]
        name: String,
    },
    RemoveCapability {
        #[arg(value_name = "CAPABILITY")]
        name: String,
    },
    InitKittynode,
    DeleteKittynode,
    IsDockerRunning,
    StartDockerIfNeeded,
    GetOperationalState,
    Validator {
        #[command(subcommand)]
        command: ValidatorCommands,
    },
}

#[derive(Subcommand)]
enum ValidatorCommands {
    #[command(name = "generate-keys")]
    GenerateKeys {
        #[arg(long = "output-dir", value_name = "PATH")]
        output_dir: PathBuf,
        #[arg(long = "entropy", value_name = "STRING")]
        entropy: String,
        #[arg(long = "file-name", value_name = "NAME")]
        file_name: Option<String>,
        #[arg(long = "overwrite")]
        overwrite: bool,
    },
    #[command(name = "create-deposit-data")]
    CreateDepositData {
        #[arg(long = "key", value_name = "PATH")]
        key_path: PathBuf,
        #[arg(long = "output", value_name = "PATH")]
        output_path: PathBuf,
        #[arg(long = "withdrawal-credentials", value_name = "HEX")]
        withdrawal_credentials: String,
        #[arg(long = "amount-gwei", default_value_t = 32_000_000_000)]
        amount_gwei: u64,
        #[arg(long = "fork-version", default_value = "00000000")]
        fork_version: String,
        #[arg(long = "genesis-root", value_name = "HEX")]
        genesis_root: String,
        #[arg(long = "overwrite")]
        overwrite: bool,
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
        Commands::GetPackages => commands::get_packages().await?,
        Commands::GetInstalledPackages => commands::get_installed_packages().await?,
        Commands::InstallPackage { name } => commands::install_package(name).await?,
        Commands::DeletePackage {
            name,
            include_images,
        } => commands::delete_package(name, include_images).await?,
        Commands::SystemInfo => commands::system_info().await?,
        Commands::GetContainerLogs { container, tail } => {
            commands::get_container_logs(container, tail).await?
        }
        Commands::GetConfig => commands::get_config()?,
        Commands::GetPackageConfig { name } => commands::get_package_config(name).await?,
        Commands::UpdatePackageConfig { name, values } => {
            commands::update_package_config(name, values).await?
        }
        Commands::GetCapabilities => commands::get_capabilities()?,
        Commands::AddCapability { name } => commands::add_capability(name)?,
        Commands::RemoveCapability { name } => commands::remove_capability(name)?,
        Commands::InitKittynode => commands::init_kittynode()?,
        Commands::DeleteKittynode => commands::delete_kittynode()?,
        Commands::IsDockerRunning => commands::is_docker_running().await?,
        Commands::StartDockerIfNeeded => commands::start_docker_if_needed().await?,
        Commands::GetOperationalState => commands::get_operational_state().await?,
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
                overwrite,
            } => commands::validator_create_deposit_data(
                key_path,
                output_path,
                withdrawal_credentials,
                amount_gwei,
                fork_version,
                genesis_root,
                overwrite,
            )?,
        },
    }

    Ok(())
}
