use eyre::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use sysinfo::{Disks, System};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemInfo {
    pub processor: ProcessorInfo,
    pub memory: MemoryInfo,
    pub storage: StorageInfo,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessorInfo {
    pub name: String,
    pub cores: u32,
    pub frequency_ghz: f64,
    pub architecture: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryInfo {
    pub total_bytes: u64,
    pub total_display: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StorageInfo {
    pub disks: Vec<DiskInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiskInfo {
    pub name: String,
    pub mount_point: String,
    pub total_bytes: u64,
    pub available_bytes: u64,
    pub total_display: String,
    pub used_display: String,
    pub available_display: String,
    pub disk_type: String,
}

pub fn get_system_info() -> Result<SystemInfo> {
    let mut system = System::new_all();
    system.refresh_all();

    Ok(SystemInfo {
        processor: get_processor_info(&system)?,
        memory: get_memory_info(&system),
        storage: get_storage_info()?,
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
        total_display: format_memory_gb(total),
    }
}

fn get_storage_info() -> Result<StorageInfo> {
    let disks = Disks::new_with_refreshed_list();

    let snapshots: Vec<DiskSnapshot> = disks
        .list()
        .iter()
        .filter_map(|disk| {
            #[cfg(target_os = "macos")]
            {
                let mount_point = disk.mount_point().to_str()?;
                if mount_point.starts_with("/System/Volumes") || mount_point == "/private/var/vm" {
                    return None;
                }
            }

            Some(DiskSnapshot {
                name: disk.name().to_str()?.to_string(),
                mount_point: disk.mount_point().to_str()?.to_string(),
                total_bytes: disk.total_space(),
                available_bytes: disk.available_space(),
                file_system: disk.file_system().to_str()?.to_string(),
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
    const MIN_DISK_SIZE: u64 = 10 * 1024 * 1024 * 1024;

    let mut seen_signatures = HashSet::new();
    let disk_infos: Vec<DiskInfo> = disks
        .into_iter()
        .filter_map(|disk| {
            if disk.total_bytes < MIN_DISK_SIZE || disk.total_bytes == 0 {
                return None;
            }

            let signature = (disk.total_bytes, disk.available_bytes);
            if !seen_signatures.insert(signature) {
                return None;
            }

            Some(DiskInfo {
                name: disk.name,
                mount_point: disk.mount_point,
                total_bytes: disk.total_bytes,
                available_bytes: disk.available_bytes,
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

fn format_memory_gb(bytes: u64) -> String {
    const MIB: u64 = 1024 * 1024;
    const GIB: u64 = 1024 * 1024 * 1024;
    const MARKETING_LEVELS: &[u64] = &[
        4, 6, 8, 12, 16, 24, 32, 48, 64, 96, 128, 192, 256, 384, 512, 768, 1024, 1536, 2048,
    ];
    const MARKETING_TOLERANCE: f64 = 0.07;

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
        format!("{} MB", bytes.div_ceil(MIB))
    } else {
        format!("{} B", bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::format_memory_gb;

    #[test]
    fn memory_binary_formats_expected() {
        let bytes_32_gib = 32u64 * 1024 * 1024 * 1024;
        let formatted = format_memory_gb(bytes_32_gib);
        assert_eq!(formatted, "32 GB", "got {formatted}");
    }

    #[test]
    fn memory_ceil_prevents_under_reporting() {
        const GIB: u64 = 1024 * 1024 * 1024;
        const MIB: u64 = 1024 * 1024;

        let bytes = (32 * GIB) - (512 * MIB);
        let formatted = format_memory_gb(bytes);
        assert_eq!(formatted, "32 GB", "got {formatted}");
    }

    #[test]
    fn memory_snaps_to_marketing_tier_when_exactly_under() {
        const GIB: u64 = 1024 * 1024 * 1024;
        let bytes = 31 * GIB;
        let formatted = format_memory_gb(bytes);
        assert_eq!(formatted, "32 GB", "got {formatted}");
    }

    #[test]
    fn memory_uses_mb_for_sub_gib_values() {
        const MIB: u64 = 1024 * 1024;
        let bytes = 512 * MIB;
        let formatted = format_memory_gb(bytes);
        assert_eq!(formatted, "512 MB", "got {formatted}");
    }
}
