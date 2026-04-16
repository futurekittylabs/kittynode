use clap::{Subcommand, ValueEnum};
use eyre::{Result, WrapErr, eyre};
use kittynode_core::packages::{Package, PackageConfig, RuntimeStatus};
use std::collections::HashMap;

#[derive(Subcommand)]
pub enum PackageCommands {
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

#[derive(Copy, Clone, Debug, ValueEnum)]
pub(crate) enum EthereumNetwork {
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
pub(crate) enum PackageConfigCommands {
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

impl PackageCommands {
    pub async fn execute(self) -> Result<()> {
        match self {
            Self::Catalog => get_package_catalog().await,
            Self::List => get_installed_packages().await,
            Self::Install { name, network } => {
                install_package(name, network.map(EthereumNetwork::as_str)).await
            }
            Self::Delete {
                name,
                include_images,
            } => delete_package(name, include_images).await,
            Self::Stop { name } => stop_package(name).await,
            Self::Start { name } => start_package(name).await,
            Self::Config { command } => command.execute().await,
        }
    }
}

impl PackageConfigCommands {
    async fn execute(self) -> Result<()> {
        match self {
            Self::Show { name } => get_package_config(name).await,
            Self::Set { name, values } => update_package_config(name, values).await,
        }
    }
}

async fn get_package_catalog() -> Result<()> {
    let packages = kittynode_core::packages::get_package_catalog()?;
    let mut entries: Vec<(&String, &Package)> = packages.iter().collect();
    entries.sort_by_key(|(name, _)| *name);
    for (_, package) in entries {
        println!("{package}");
    }
    Ok(())
}

async fn get_installed_packages() -> Result<()> {
    let packages = kittynode_core::packages::get_installed_packages().await?;
    let names: Vec<String> = packages.iter().map(|pkg| pkg.name().to_string()).collect();
    let name_refs: Vec<&str> = names.iter().map(|name| name.as_str()).collect();
    let runtime_states = match kittynode_core::packages::get_packages(&name_refs).await {
        Ok(map) => map,
        Err(error) => {
            tracing::warn!(%error, "failed to retrieve runtime state information");
            HashMap::new()
        }
    };

    if packages.is_empty() {
        println!("No packages are currently installed");
        return Ok(());
    }

    for package in &packages {
        let state = runtime_states
            .get(package.name())
            .map(|state| {
                if state.runtime == RuntimeStatus::Running {
                    "running"
                } else {
                    "stopped"
                }
            })
            .unwrap_or("unknown");

        println!("{} [status: {state}]", package.name());
        println!("  {}", package.description());
        println!("  Network: {}", package.network_name());
        println!();
    }
    Ok(())
}

async fn install_package(name: String, network: Option<&str>) -> Result<()> {
    if name == "ethereum" && network.is_none() {
        let supported = kittynode_core::ethereum::supported_networks_display("|");
        return Err(eyre!(
            "Network must be provided when installing ethereum. Use --network <{supported}>"
        ));
    }

    kittynode_core::packages::install_package_with_network(&name, network)
        .await
        .wrap_err_with(|| format!("Failed to install {name}"))?;
    tracing::info!("installed {name}");
    Ok(())
}

async fn delete_package(name: String, include_images: bool) -> Result<()> {
    let packages = kittynode_core::packages::get_installed_packages()
        .await
        .wrap_err("Failed to list installed packages")?;
    let Some(package) = packages.iter().find(|pkg| pkg.name() == name) else {
        println!("Package {name} is not installed");
        return Ok(());
    };
    let resolved_name = package.name();

    kittynode_core::packages::delete_package(resolved_name, include_images)
        .await
        .wrap_err_with(|| format!("Failed to delete {resolved_name}"))?;
    tracing::info!("deleted {resolved_name}");
    Ok(())
}

async fn stop_package(name: String) -> Result<()> {
    kittynode_core::packages::stop_package(&name)
        .await
        .wrap_err_with(|| format!("Failed to stop {name}"))?;
    tracing::info!("stopped {name}");
    Ok(())
}

async fn start_package(name: String) -> Result<()> {
    kittynode_core::packages::start_package(&name)
        .await
        .wrap_err_with(|| format!("Failed to start {name}"))?;
    tracing::info!("started {name}");
    Ok(())
}

async fn get_package_config(name: String) -> Result<()> {
    let config = kittynode_core::packages::get_package_config(&name).await?;
    if config.values.is_empty() {
        println!("No overrides set for {name}");
    } else {
        println!("Overrides for {name}:");
        for (key, value) in &config.values {
            println!("  {key}={value}");
        }
    }
    Ok(())
}

async fn update_package_config(name: String, values: Vec<(String, String)>) -> Result<()> {
    let config = PackageConfig {
        values: values.into_iter().collect(),
    };
    kittynode_core::packages::update_package_config(&name, config)
        .await
        .wrap_err_with(|| format!("Failed to update config for {name}"))?;
    tracing::info!("updated config for {name}");
    Ok(())
}

fn parse_key_val(input: &str) -> Result<(String, String), String> {
    let position = input
        .find('=')
        .ok_or_else(|| "expected KEY=VALUE".to_string())?;
    let key = input[..position].trim();
    let value = input[position + 1..].trim();
    if key.is_empty() {
        return Err("key cannot be empty".to_string());
    }
    Ok((key.to_string(), value.to_string()))
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
