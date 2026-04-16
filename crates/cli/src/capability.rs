use clap::Subcommand;
use eyre::{Result, WrapErr};

#[derive(Subcommand)]
pub enum CapabilityCommands {
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

impl CapabilityCommands {
    pub fn execute(self) -> Result<()> {
        match self {
            Self::List => list_capabilities(),
            Self::Add { name } => add_capability(name),
            Self::Remove { name } => remove_capability(name),
        }
    }
}

fn list_capabilities() -> Result<()> {
    let capabilities = kittynode_core::config::get_capabilities()?;
    if capabilities.is_empty() {
        println!("No capabilities configured");
    } else {
        for capability in &capabilities {
            println!("{capability}");
        }
    }
    Ok(())
}

fn add_capability(name: String) -> Result<()> {
    kittynode_core::config::add_capability(&name)
        .wrap_err_with(|| format!("Failed to add capability {name}"))?;
    tracing::info!("added capability {name}");
    Ok(())
}

fn remove_capability(name: String) -> Result<()> {
    kittynode_core::config::remove_capability(&name)
        .wrap_err_with(|| format!("Failed to remove capability {name}"))?;
    tracing::info!("removed capability {name}");
    Ok(())
}
