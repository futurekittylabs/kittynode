use clap::Subcommand;
use eyre::Result;

#[derive(Subcommand)]
pub enum ContainerCommands {
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

impl ContainerCommands {
    pub async fn execute(self) -> Result<()> {
        match self {
            Self::Logs { container, tail } => logs(container, tail).await,
        }
    }
}

async fn logs(container: String, tail: Option<usize>) -> Result<()> {
    let logs = kittynode_core::docker::get_container_logs(&container, tail).await?;
    for line in logs {
        println!("{line}");
    }
    Ok(())
}
