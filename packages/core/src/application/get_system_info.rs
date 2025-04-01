use crate::domain::system_info::{DiskInfo, MemoryInfo, ProcessorInfo, StorageInfo, SystemInfo};
use eyre::Result;
use sysinfo::{Disks, System};

fn format_bytes(bytes: u64) -> String {
    let units = ["B", "KB", "MB", "GB", "TB"];
    let mut value = bytes as f64;
    let mut unit_index = 0;

    let binary_to_decimal = |bytes: f64, power: i32| -> f64 {
        bytes * (1024.0_f64.powi(power) / 1000.0_f64.powi(power))
    };

    while value >= 1024.0 && unit_index < units.len() - 1 {
        value = binary_to_decimal(value, 1);
        value /= 1000.0;
        unit_index += 1;
    }

    format!("{:.2} {}", value, units[unit_index])
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
        total_display: format_bytes(total),
    }
}

fn get_storage_info() -> Result<StorageInfo> {
    const MIN_DISK_SIZE: u64 = 10 * 1024 * 1024 * 1024; // 10 GiB

    let disks = Disks::new_with_refreshed_list();

    let mut seen_signatures = std::collections::HashSet::new();
    let disk_infos: Vec<DiskInfo> = disks
        .list()
        .iter()
        .filter_map(|disk| {
            if disk.total_space() < MIN_DISK_SIZE || disk.total_space() == 0 {
                return None;
            }

            let storage_signature = (disk.total_space(), disk.available_space());
            if !seen_signatures.insert(storage_signature) {
                return None;
            }

            Some(DiskInfo {
                name: disk.name().to_str()?.to_string(),
                mount_point: disk.mount_point().to_str()?.to_string(),
                total_bytes: disk.total_space(),
                available_bytes: disk.available_space(),
                total_display: format_bytes(disk.total_space()),
                available_display: format_bytes(disk.available_space()),
                disk_type: disk.file_system().to_str()?.to_string(),
            })
        })
        .collect();

    if disk_infos.is_empty() {
        return Err(eyre::eyre!("No valid disks found"));
    }

    Ok(StorageInfo { disks: disk_infos })
}
