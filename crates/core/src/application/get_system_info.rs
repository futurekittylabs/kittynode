use crate::domain::system_info::{DiskInfo, MemoryInfo, ProcessorInfo, StorageInfo, SystemInfo};
use eyre::Result;
use std::collections::HashSet;
use sysinfo::{Disks, System};

// Formats bytes using decimal multiples (B, KB, MB, GB, TB)
fn format_bytes_decimal(bytes: u64) -> String {
    let units = ["B", "KB", "MB", "GB", "TB"];
    let mut value = bytes as f64;
    let mut unit_index = 0;

    while value >= 1000.0 && unit_index < units.len() - 1 {
        value /= 1000.0;
        unit_index += 1;
    }

    format!("{:.2} {}", value, units[unit_index])
}

// Formats memory preferring whole-number GB. We snap to common marketed
// capacities (8, 16, 32, 64, ... GB) when the reported bytes are closely below
// those tiers to avoid showing "31 GB" on a 32 GB machine. When memory is below
// 1 GiB we fall back to MB granularity for accuracy on smaller systems.
fn format_memory_gb(bytes: u64) -> String {
    const MIB: u64 = 1024 * 1024;
    const GIB: u64 = 1024 * 1024 * 1024;
    const MARKETING_LEVELS: &[u64] = &[
        4, 6, 8, 12, 16, 24, 32, 48, 64, 96, 128, 192, 256, 384, 512, 768, 1024, 1536, 2048,
    ];
    const MARKETING_TOLERANCE: f64 = 0.07; // 7% below a tier still rounds up.

    if bytes >= GIB {
        let actual_gb = bytes as f64 / GIB as f64;
        let fallback_gb = actual_gb.round().max(1.0) as u64;

        let snapped_gb = MARKETING_LEVELS
            .iter()
            .copied()
            .find(|&tier| {
                let tier_f = tier as f64;
                tier_f >= actual_gb && (tier_f - actual_gb) / tier_f <= MARKETING_TOLERANCE
            })
            .unwrap_or(fallback_gb);

        format!("{} GB", snapped_gb)
    } else if bytes >= MIB {
        let mb = bytes.div_ceil(MIB);
        format!("{} MB", mb)
    } else {
        format!("{} B", bytes)
    }
}

pub fn get_system_info() -> Result<SystemInfo> {
    let mut system = System::new_all();
    system.refresh_all();

    let processor = get_processor_info(&system)?;
    let memory = get_memory_info(&system);
    let storage = get_storage_info()?;

    Ok(SystemInfo {
        processor,
        memory,
        storage,
    })
}

fn get_processor_info(system: &System) -> Result<ProcessorInfo> {
    let cpu = system
        .cpus()
        .first()
        .ok_or_else(|| eyre::eyre!("No CPU found"))?;

    Ok(ProcessorInfo {
        name: if cpu.brand().is_empty() {
            "Unknown CPU".to_string()
        } else {
            cpu.brand().to_string()
        },
        cores: sysinfo::System::physical_core_count().unwrap_or(1) as u32,
        frequency_ghz: cpu.frequency() as f64 / 1000.0,
        architecture: std::env::consts::ARCH.to_string(),
    })
}

fn get_memory_info(system: &System) -> MemoryInfo {
    let total = system.total_memory();
    MemoryInfo {
        total_bytes: total,
        // Show whole-number GB for user-friendly RAM display
        total_display: format_memory_gb(total),
    }
}

fn get_storage_info() -> Result<StorageInfo> {
    let disks = Disks::new_with_refreshed_list();

    let snapshots: Vec<DiskSnapshot> = disks
        .list()
        .iter()
        .filter_map(|disk| {
            // On macOS, sysinfo exposes multiple APFS system volumes (e.g.,
            // "/" and "/System/Volumes/Data") that represent the same
            // physical drive. These should not be shown separately in the UI.
            // Filter out internal system mount points to avoid duplicates like
            // "Macintosh HD" appearing twice.
            #[cfg(target_os = "macos")]
            {
                let mp = match disk.mount_point().to_str() {
                    Some(s) => s,
                    None => return None,
                };
                if mp.starts_with("/System/Volumes") || mp == "/private/var/vm" {
                    return None;
                }
            }

            let mount_point = disk.mount_point().to_str()?.to_string();
            let name = disk.name().to_str()?.to_string();
            let file_system = disk.file_system().to_str()?.to_string();

            Some(DiskSnapshot {
                name,
                mount_point,
                total_bytes: disk.total_space(),
                available_bytes: disk.available_space(),
                file_system,
            })
        })
        .collect();

    build_storage_info(snapshots)
}

#[derive(Debug, Clone)]
struct DiskSnapshot {
    name: String,
    mount_point: String,
    total_bytes: u64,
    available_bytes: u64,
    file_system: String,
}

fn build_storage_info(disks: Vec<DiskSnapshot>) -> Result<StorageInfo> {
    const MIN_DISK_SIZE: u64 = 10 * 1024 * 1024 * 1024; // 10 GiB

    let mut seen_signatures = HashSet::new();
    let disk_infos: Vec<DiskInfo> = disks
        .into_iter()
        .filter_map(|disk| {
            if disk.total_bytes < MIN_DISK_SIZE || disk.total_bytes == 0 {
                return None;
            }

            let storage_signature = (disk.total_bytes, disk.available_bytes);
            if !seen_signatures.insert(storage_signature) {
                return None;
            }

            Some(DiskInfo {
                name: disk.name,
                mount_point: disk.mount_point,
                total_bytes: disk.total_bytes,
                available_bytes: disk.available_bytes,
                // For disks, users expect decimal-based sizes like Finder/Windows
                total_display: format_bytes_decimal(disk.total_bytes),
                used_display: format_bytes_decimal(disk.total_bytes - disk.available_bytes),
                available_display: format_bytes_decimal(disk.available_bytes),
                disk_type: disk.file_system,
            })
        })
        .collect();

    if disk_infos.is_empty() {
        return Err(eyre::eyre!("No valid disks found"));
    }

    Ok(StorageInfo { disks: disk_infos })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memory_binary_formats_expected() {
        // 32 GiB in bytes
        let bytes_32_gib = 32u64 * 1024 * 1024 * 1024;
        let s = format_memory_gb(bytes_32_gib);
        assert_eq!(s, "32 GB", "got {s}");
    }

    #[test]
    fn memory_ceil_prevents_under_reporting() {
        const GIB: u64 = 1024 * 1024 * 1024;
        const MIB: u64 = 1024 * 1024;

        // 32 GiB minus 512 MiB should still display as 32 GB for user expectations
        let bytes = (32 * GIB) - (512 * MIB);
        let s = format_memory_gb(bytes);
        assert_eq!(s, "32 GB", "got {s}");
    }

    #[test]
    fn memory_snaps_to_marketing_tier_when_exactly_under() {
        const GIB: u64 = 1024 * 1024 * 1024;
        // Exactly 31 GiB should report 32 GB marketed size
        let bytes = 31 * GIB;
        let s = format_memory_gb(bytes);
        assert_eq!(s, "32 GB", "got {s}");
    }

    #[test]
    fn memory_uses_mb_for_sub_gib_values() {
        const MIB: u64 = 1024 * 1024;
        let bytes = 512 * MIB;
        let s = format_memory_gb(bytes);
        assert_eq!(s, "512 MB", "got {s}");
    }

    #[test]
    fn memory_falls_back_when_far_from_marketing_tier() {
        const GIB: u64 = 1024 * 1024 * 1024;
        let bytes = 20 * GIB;
        let s = format_memory_gb(bytes);
        assert_eq!(s, "20 GB", "got {s}");
    }

    #[test]
    fn memory_rounds_nearest_when_not_marketing_tier() {
        const GIB: u64 = 1024 * 1024 * 1024;
        let bytes = (20 * GIB) + (200 * 1024 * 1024); // ~20.2 GiB
        let s = format_memory_gb(bytes);
        assert_eq!(s, "20 GB", "got {s}");
    }

    #[test]
    fn decimal_formats_expected() {
        // 32 GiB in bytes should be ~34.36 GB in decimal
        let bytes_32_gib = 32u64 * 1024 * 1024 * 1024;
        let s = format_bytes_decimal(bytes_32_gib);
        assert!(s.starts_with("34.36 GB"), "got {s}");
    }

    #[test]
    fn build_storage_info_filters_small_and_duplicate_disks() {
        let disks = vec![
            DiskSnapshot {
                name: "primary".into(),
                mount_point: "/".into(),
                total_bytes: 512u64 * 1024 * 1024 * 1024, // 512 GiB
                available_bytes: 200u64 * 1024 * 1024 * 1024,
                file_system: "apfs".into(),
            },
            DiskSnapshot {
                // Duplicate signature should be ignored
                name: "duplicate".into(),
                mount_point: "/System/Volumes/Data".into(),
                total_bytes: 512u64 * 1024 * 1024 * 1024,
                available_bytes: 200u64 * 1024 * 1024 * 1024,
                file_system: "apfs".into(),
            },
            DiskSnapshot {
                // Too small, should be ignored
                name: "tiny".into(),
                mount_point: "/tiny".into(),
                total_bytes: 2u64 * 1024 * 1024 * 1024,
                available_bytes: 1024u64 * 1024 * 1024,
                file_system: "ext4".into(),
            },
        ];

        let storage = build_storage_info(disks).expect("expected valid storage info");
        assert_eq!(storage.disks.len(), 1);
        assert_eq!(storage.disks[0].name, "primary");
        assert_eq!(
            storage.disks[0].available_bytes,
            200u64 * 1024 * 1024 * 1024
        );
    }

    #[test]
    fn build_storage_info_errors_when_no_valid_disks() {
        let disks = vec![DiskSnapshot {
            name: "tiny".into(),
            mount_point: "/tiny".into(),
            total_bytes: 0,
            available_bytes: 0,
            file_system: "ext4".into(),
        }];

        let err = build_storage_info(disks).expect_err("expected validation failure");
        assert!(err.to_string().contains("No valid disks"));
    }

    #[test]
    fn build_storage_info_formats_display_strings() {
        let total = 512u64 * 1024 * 1024 * 1024; // 512 GiB
        let available = 200u64 * 1024 * 1024 * 1024; // 200 GiB

        let storage = build_storage_info(vec![DiskSnapshot {
            name: "primary".into(),
            mount_point: "/".into(),
            total_bytes: total,
            available_bytes: available,
            file_system: "apfs".into(),
        }])
        .expect("expected valid storage info");

        let disk = &storage.disks[0];
        assert_eq!(disk.total_display, format_bytes_decimal(total));
        assert_eq!(disk.available_display, format_bytes_decimal(available));
        assert_eq!(disk.used_display, format_bytes_decimal(total - available));
    }
}
