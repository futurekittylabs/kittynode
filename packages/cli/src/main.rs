mod commands;
mod output;

use atty::Stream;
use clap::{Parser, Subcommand};
use eyre::Result;
use output::OutputFormat;

#[derive(Parser)]
#[command(about, version)]
struct Cli {
    #[arg(long, value_enum)]
    format: Option<OutputFormat>,
    #[arg(
        long,
        conflicts_with = "format",
        help = "Output in JSON format (alias for --format json)"
    )]
    json: bool,
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
    let format = determine_format(cli.format, cli.json);

    match cli.command {
        Commands::GetPackages => commands::get_packages(format).await?,
        Commands::GetInstalledPackages => commands::get_installed_packages(format).await?,
        Commands::InstallPackage { name } => commands::install_package(name).await?,
        Commands::DeletePackage {
            name,
            include_images,
        } => commands::delete_package(name, include_images).await?,
        Commands::SystemInfo => commands::system_info(format).await?,
        Commands::GetContainerLogs { container, tail } => {
            commands::get_container_logs(format, container, tail).await?
        }
        Commands::GetConfig => commands::get_config(format)?,
        Commands::GetPackageConfig { name } => commands::get_package_config(format, name).await?,
        Commands::UpdatePackageConfig { name, values } => {
            commands::update_package_config(name, values).await?
        }
        Commands::GetCapabilities => commands::get_capabilities(format)?,
        Commands::AddCapability { name } => commands::add_capability(name)?,
        Commands::RemoveCapability { name } => commands::remove_capability(name)?,
        Commands::InitKittynode => commands::init_kittynode()?,
        Commands::DeleteKittynode => commands::delete_kittynode()?,
        Commands::IsDockerRunning => commands::is_docker_running(format).await?,
        Commands::StartDockerIfNeeded => commands::start_docker_if_needed(format).await?,
        Commands::GetOperationalState => commands::get_operational_state(format).await?,
    }

    Ok(())
}

fn determine_format(explicit: Option<OutputFormat>, json: bool) -> OutputFormat {
    if let Some(format) = explicit {
        return format;
    }

    if json {
        return OutputFormat::Json;
    }

    if atty::is(Stream::Stdout) {
        OutputFormat::Text
    } else {
        OutputFormat::Json
    }
}
