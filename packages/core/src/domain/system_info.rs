use serde::{Deserialize, Serialize};

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn processor_info_serializes_with_camel_case_fields() {
        let processor = ProcessorInfo {
            name: "Apple M1".to_string(),
            cores: 8,
            frequency_ghz: 3.2,
            architecture: "arm64".to_string(),
        };

        let json = serde_json::to_value(&processor).unwrap();

        assert_eq!(json["name"], "Apple M1");
        assert_eq!(json["cores"], 8);
        assert_eq!(json["frequencyGhz"], 3.2);
        assert_eq!(json["architecture"], "arm64");
    }

    #[test]
    fn memory_info_serializes_with_camel_case_fields() {
        let memory = MemoryInfo {
            total_bytes: 17179869184,
            total_display: "16 GB".to_string(),
        };

        let json = serde_json::to_value(&memory).unwrap();

        assert_eq!(json["totalBytes"], 17179869184_u64);
        assert_eq!(json["totalDisplay"], "16 GB");
    }

    #[test]
    fn disk_info_serializes_with_camel_case_fields() {
        let disk = DiskInfo {
            name: "Macintosh HD".to_string(),
            mount_point: "/".to_string(),
            total_bytes: 500000000000,
            available_bytes: 200000000000,
            total_display: "500 GB".to_string(),
            used_display: "300 GB".to_string(),
            available_display: "200 GB".to_string(),
            disk_type: "SSD".to_string(),
        };

        let json = serde_json::to_value(&disk).unwrap();

        assert_eq!(json["name"], "Macintosh HD");
        assert_eq!(json["mountPoint"], "/");
        assert_eq!(json["totalBytes"], 500000000000_u64);
        assert_eq!(json["availableBytes"], 200000000000_u64);
        assert_eq!(json["totalDisplay"], "500 GB");
        assert_eq!(json["usedDisplay"], "300 GB");
        assert_eq!(json["availableDisplay"], "200 GB");
        assert_eq!(json["diskType"], "SSD");
    }

    #[test]
    fn system_info_roundtrips_through_json() {
        let original = SystemInfo {
            processor: ProcessorInfo {
                name: "Intel Core i7".to_string(),
                cores: 4,
                frequency_ghz: 2.8,
                architecture: "x86_64".to_string(),
            },
            memory: MemoryInfo {
                total_bytes: 8589934592,
                total_display: "8 GB".to_string(),
            },
            storage: StorageInfo {
                disks: vec![DiskInfo {
                    name: "Main".to_string(),
                    mount_point: "/".to_string(),
                    total_bytes: 1000000000000,
                    available_bytes: 500000000000,
                    total_display: "1 TB".to_string(),
                    used_display: "500 GB".to_string(),
                    available_display: "500 GB".to_string(),
                    disk_type: "HDD".to_string(),
                }],
            },
        };

        let json = serde_json::to_string(&original).unwrap();
        let deserialized: SystemInfo = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.processor.name, original.processor.name);
        assert_eq!(deserialized.processor.cores, original.processor.cores);
        assert_eq!(
            deserialized.processor.frequency_ghz,
            original.processor.frequency_ghz
        );
        assert_eq!(deserialized.memory.total_bytes, original.memory.total_bytes);
        assert_eq!(deserialized.storage.disks.len(), 1);
        assert_eq!(
            deserialized.storage.disks[0].name,
            original.storage.disks[0].name
        );
    }

    #[test]
    fn storage_info_handles_multiple_disks() {
        let storage = StorageInfo {
            disks: vec![
                DiskInfo {
                    name: "Disk 1".to_string(),
                    mount_point: "/".to_string(),
                    total_bytes: 1000,
                    available_bytes: 500,
                    total_display: "1 KB".to_string(),
                    used_display: "500 B".to_string(),
                    available_display: "500 B".to_string(),
                    disk_type: "SSD".to_string(),
                },
                DiskInfo {
                    name: "Disk 2".to_string(),
                    mount_point: "/mnt".to_string(),
                    total_bytes: 2000,
                    available_bytes: 1000,
                    total_display: "2 KB".to_string(),
                    used_display: "1 KB".to_string(),
                    available_display: "1 KB".to_string(),
                    disk_type: "HDD".to_string(),
                },
            ],
        };

        let json = serde_json::to_value(&storage).unwrap();

        assert_eq!(json["disks"].as_array().unwrap().len(), 2);
        assert_eq!(json["disks"][0]["name"], "Disk 1");
        assert_eq!(json["disks"][1]["name"], "Disk 2");
    }
}
