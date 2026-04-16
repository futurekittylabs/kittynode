use eyre::{Result, eyre};
use std::process::Command;

/// Launch the standalone updater installed by cargo-dist installers.
/// This expects a `kittynode-cli-update` binary to be on PATH.
pub fn run() -> Result<()> {
    if cfg!(windows) {
        println!("Update kittynode-cli by entering the command `kittynode-cli-update`!");
        return Ok(());
    }

    match Command::new("kittynode-cli-update").status() {
        Ok(status) => {
            if status.success() {
                Ok(())
            } else {
                Err(eyre!(
                    "updater exited with code {}",
                    status.code().unwrap_or(-1)
                ))
            }
        }
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Err(eyre!(
            "could not find 'kittynode-cli-update' in PATH; reinstall via the installer or ensure the updater is installed"
        )),
        Err(err) => Err(eyre!("failed to launch updater: {err}")),
    }
}
