export interface SystemInfo {
  processor: ProcessorInfo;
  memory: MemoryInfo;
  storage: StorageInfo;
}

export interface ProcessorInfo {
  name: string;
  cores: number;
  frequency_ghz: number;
  architecture: string;
}

export interface MemoryInfo {
  total_bytes: number;
  total_display: string;
}

export interface StorageInfo {
  disks: DiskInfo[];
}

export interface DiskInfo {
  name: string;
  mount_point: string;
  total_bytes: number;
  available_bytes: number;
  total_display: string;
  used_display: string;
  available_display: string;
  disk_type: string;
}
