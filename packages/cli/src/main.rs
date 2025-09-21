mod commands;

use clap::{Parser, Subcommand};
use eyre::Result;

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
    }

    Ok(())
}
