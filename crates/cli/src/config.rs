use clap::Subcommand;
use eyre::{Result, WrapErr};
use kittynode_core::config::Config;
use std::fmt::Write;

#[derive(Subcommand)]
pub enum ConfigCommands {
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

impl ConfigCommands {
    pub fn execute(self) -> Result<()> {
        match self {
            Self::Show => show_config(),
            Self::Init => init_kittynode(),
            Self::Delete => delete_kittynode(),
        }
    }
}

fn show_config() -> Result<()> {
    let config = kittynode_core::config::get_config()?;
    print!("{}", render_config(&config));
    Ok(())
}

fn init_kittynode() -> Result<()> {
    kittynode_core::node::init_kittynode().wrap_err("Failed to initialize Kittynode")?;
    tracing::info!("initialized kittynode");
    Ok(())
}

fn delete_kittynode() -> Result<()> {
    kittynode_core::node::delete_kittynode().wrap_err("Failed to delete Kittynode data")?;
    tracing::info!("deleted kittynode data");
    Ok(())
}

fn render_config(config: &Config) -> String {
    let mut output = String::new();
    let server = if config.server_url.is_empty() {
        "(local)"
    } else {
        config.server_url.as_str()
    };
    writeln!(output, "Server URL: {server}").expect("writing to string cannot fail");
    writeln!(output, "Capabilities:").expect("writing to string cannot fail");
    for capability in &config.capabilities {
        writeln!(output, "  - {capability}").expect("writing to string cannot fail");
    }
    let onboarding = if config.onboarding_completed {
        "yes"
    } else {
        "no"
    };
    writeln!(output, "Onboarding completed: {onboarding}").expect("writing to string cannot fail");
    let auto_start = if config.auto_start_docker {
        "enabled"
    } else {
        "disabled"
    };
    writeln!(output, "Auto start Docker: {auto_start}").expect("writing to string cannot fail");
    output
}

#[cfg(test)]
mod tests {
    use super::render_config;
    use kittynode_core::config::Config;

    #[test]
    fn render_config_formats_remote_server_with_capabilities() {
        let config = Config {
            capabilities: vec!["ethereum".into(), "solana".into()],
            server_url: "https://rpc.example".into(),
            onboarding_completed: true,
            auto_start_docker: false,
            ..Default::default()
        };

        let rendered = render_config(&config);
        let expected = "Server URL: https://rpc.example\nCapabilities:\n  - ethereum\n  - solana\nOnboarding completed: yes\nAuto start Docker: disabled\n";
        assert_eq!(rendered, expected);
    }

    #[test]
    fn render_config_for_local_server_shows_placeholder() {
        let config = Config {
            server_url: String::new(),
            capabilities: vec![],
            onboarding_completed: false,
            auto_start_docker: true,
            ..Default::default()
        };

        let rendered = render_config(&config);
        let expected = "Server URL: (local)\nCapabilities:\nOnboarding completed: no\nAuto start Docker: enabled\n";
        assert_eq!(rendered, expected);
    }
}
