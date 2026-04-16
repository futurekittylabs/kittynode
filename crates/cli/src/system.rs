use clap::Subcommand;
use eyre::Result;
use kittynode_core::node::{OperationalMode, OperationalState};
use kittynode_core::system::SystemInfo;
use std::fmt::Write;

#[derive(Subcommand)]
pub enum SystemCommands {
    #[command(
        name = "info",
        about = "Display hardware and OS details used by Kittynode"
    )]
    Info,
    #[command(
        name = "state",
        about = "Show overall operational status and readiness flags"
    )]
    State,
}

impl SystemCommands {
    pub async fn execute(self) -> Result<()> {
        match self {
            Self::Info => system_info().await,
            Self::State => operational_state().await,
        }
    }
}

async fn system_info() -> Result<()> {
    let info = kittynode_core::system::get_system_info()?;
    print!("{}", render_system_info(&info));
    Ok(())
}

async fn operational_state() -> Result<()> {
    let state = kittynode_core::node::get_operational_state().await?;
    print!("{}", render_operational_state(&state));
    Ok(())
}

fn render_system_info(info: &SystemInfo) -> String {
    let mut output = String::new();
    writeln!(
        output,
        "Processor: {} ({} cores, {:.2} GHz)",
        info.processor.name, info.processor.cores, info.processor.frequency_ghz
    )
    .expect("writing to string cannot fail");
    writeln!(output, "Memory: {}", info.memory.total_display)
        .expect("writing to string cannot fail");
    writeln!(output, "Storage:").expect("writing to string cannot fail");
    for disk in &info.storage.disks {
        writeln!(output, "  {} mounted on {}", disk.name, disk.mount_point)
            .expect("writing to string cannot fail");
        writeln!(output, "    Total: {}", disk.total_display)
            .expect("writing to string cannot fail");
        writeln!(output, "    Available: {}", disk.available_display)
            .expect("writing to string cannot fail");
    }
    output
}

fn render_operational_state(state: &OperationalState) -> String {
    let mut output = String::new();
    let mode = match state.mode {
        OperationalMode::Local => "local",
        OperationalMode::Remote => "remote",
    };
    writeln!(output, "Mode: {mode}").expect("writing to string cannot fail");
    let docker_running = if state.docker_running { "yes" } else { "no" };
    writeln!(output, "Docker running: {docker_running}").expect("writing to string cannot fail");
    let can_install = if state.can_install { "yes" } else { "no" };
    writeln!(output, "Can install: {can_install}").expect("writing to string cannot fail");
    let can_manage = if state.can_manage { "yes" } else { "no" };
    writeln!(output, "Can manage: {can_manage}").expect("writing to string cannot fail");
    if !state.diagnostics.is_empty() {
        writeln!(output, "Diagnostics:").expect("writing to string cannot fail");
        for entry in &state.diagnostics {
            writeln!(output, "  - {entry}").expect("writing to string cannot fail");
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::{render_operational_state, render_system_info};
    use kittynode_core::node::{OperationalMode, OperationalState};
    use kittynode_core::system::SystemInfo;
    use serde_json::json;

    #[test]
    fn render_operational_state_includes_diagnostics() {
        let state = OperationalState {
            mode: OperationalMode::Remote,
            docker_running: true,
            can_install: false,
            can_manage: true,
            diagnostics: vec!["restart docker".into(), "check firewall".into()],
        };

        let rendered = render_operational_state(&state);
        let expected = "Mode: remote\nDocker running: yes\nCan install: no\nCan manage: yes\nDiagnostics:\n  - restart docker\n  - check firewall\n";
        assert_eq!(rendered, expected);
    }

    #[test]
    fn render_operational_state_without_diagnostics_omits_section() {
        let state = OperationalState {
            mode: OperationalMode::Local,
            docker_running: false,
            can_install: true,
            can_manage: false,
            diagnostics: vec![],
        };

        let rendered = render_operational_state(&state);
        assert!(
            !rendered.contains("Diagnostics"),
            "expected diagnostics section to be omitted, got {rendered}",
        );
        let expected_prefix = "Mode: local\nDocker running: no\nCan install: yes\nCan manage: no\n";
        assert!(
            rendered.starts_with(expected_prefix),
            "expected output to start with {expected_prefix}, got {rendered}",
        );
    }

    #[test]
    fn render_system_info_lists_disks() {
        let info: SystemInfo = serde_json::from_value(json!({
            "processor": {
                "name": "Test CPU",
                "cores": 8,
                "frequencyGhz": 3.5,
                "architecture": "x86_64"
            },
            "memory": {
                "totalBytes": 34359738368u64,
                "totalDisplay": "32 GB"
            },
            "storage": {
                "disks": [
                    {
                        "name": "disk1",
                        "mountPoint": "/",
                        "totalBytes": 512000000000u64,
                        "availableBytes": 256000000000u64,
                        "totalDisplay": "512 GB",
                        "usedDisplay": "256 GB",
                        "availableDisplay": "256 GB",
                        "diskType": "apfs"
                    },
                    {
                        "name": "disk2",
                        "mountPoint": "/data",
                        "totalBytes": 1000000000000u64,
                        "availableBytes": 750000000000u64,
                        "totalDisplay": "1.00 TB",
                        "usedDisplay": "250.00 GB",
                        "availableDisplay": "750.00 GB",
                        "diskType": "ext4"
                    }
                ]
            }
        }))
        .expect("json literal is valid system info");

        let rendered = render_system_info(&info);
        let expected = "Processor: Test CPU (8 cores, 3.50 GHz)\nMemory: 32 GB\nStorage:\n  disk1 mounted on /\n    Total: 512 GB\n    Available: 256 GB\n  disk2 mounted on /data\n    Total: 1.00 TB\n    Available: 750.00 GB\n";
        assert_eq!(rendered, expected);
    }

    #[test]
    fn render_system_info_without_disks_still_lists_storage_section() {
        let info: SystemInfo = serde_json::from_value(json!({
            "processor": {
                "name": "Test CPU",
                "cores": 4,
                "frequencyGhz": 2.25,
                "architecture": "x86_64"
            },
            "memory": {
                "totalBytes": 17179869184u64,
                "totalDisplay": "16 GB"
            },
            "storage": {
                "disks": []
            }
        }))
        .expect("json literal is valid system info");

        let rendered = render_system_info(&info);
        let expected = "Processor: Test CPU (4 cores, 2.25 GHz)\nMemory: 16 GB\nStorage:\n";
        assert!(
            rendered.starts_with(expected),
            "expected output to start with {expected}, got {rendered}",
        );
        assert_eq!(rendered.lines().count(), 3);
    }
}
