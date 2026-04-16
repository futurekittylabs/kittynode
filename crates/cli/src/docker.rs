use clap::Subcommand;
use eyre::Result;

#[derive(Subcommand)]
pub enum DockerCommands {
    #[command(
        name = "status",
        about = "Check whether Docker is reachable from Kittynode"
    )]
    Status,
    #[command(name = "start", about = "Start Docker if it is not already running")]
    Start,
}

impl DockerCommands {
    pub async fn execute(self) -> Result<()> {
        match self {
            Self::Status => status().await,
            Self::Start => start().await,
        }
    }
}

async fn status() -> Result<()> {
    let running = kittynode_core::docker::is_docker_running().await;
    println!(
        "{}",
        if running {
            "Docker is running"
        } else {
            "Docker is not running"
        }
    );
    Ok(())
}

async fn start() -> Result<()> {
    let status = kittynode_core::node::start_docker_if_needed().await?;
    println!("{}", status.as_str());
    Ok(())
}
