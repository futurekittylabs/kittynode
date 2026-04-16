mod capability;
mod config;
mod container;
mod docker;
mod package;
mod server;
mod system;
mod update;
mod update_checker;
mod validator;
use clap::{CommandFactory, Parser, Subcommand};
use eyre::{Result, eyre};
use std::sync::atomic::{AtomicBool, Ordering};
use tracing_subscriber::fmt::MakeWriter;

#[derive(Parser)]
#[command(
    version,
    about = "Manage your Kittynode installation from the terminal"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Manage packages available to Kittynode")]
    Package {
        #[command(subcommand)]
        command: package::PackageCommands,
    },
    #[command(about = "Inspect or update Kittynode configuration")]
    Config {
        #[command(subcommand)]
        command: config::ConfigCommands,
    },
    #[command(about = "Manage capability flags on this Kittynode")]
    Capability {
        #[command(subcommand)]
        command: capability::CapabilityCommands,
    },
    #[command(about = "Inspect system diagnostics and environment")]
    System {
        #[command(subcommand)]
        command: system::SystemCommands,
    },
    #[command(about = "Control Docker services used by Kittynode")]
    Docker {
        #[command(subcommand)]
        command: docker::DockerCommands,
    },
    #[command(about = "Inspect managed containers")]
    Container {
        #[command(subcommand)]
        command: container::ContainerCommands,
    },
    #[command(about = "Manage validator key material")]
    Validator {
        #[command(subcommand)]
        command: validator::ValidatorCommands,
    },
    #[command(about = "Control the Kittynode server")]
    Server {
        #[command(subcommand)]
        command: server::ServerCommands,
    },
    #[command(about = "Update Kittynode to the latest release")]
    Update,
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
            Commands::Server { command } => command.execute().await,
            Commands::Update => update::run(),
        }
    }
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

    match cli.command {
        Some(command) => command.execute().await,
        None => {
            let mut command = Cli::command();
            command
                .print_help()
                .map_err(|err| eyre!("Failed to print help: {err}"))?;
            println!();
            Ok(())
        }
    }
}
