export interface SystemInfo {
  memory: MemoryInfo;
  processor: ProcessorInfo;
  storage: StorageInfo;
}

export interface ProcessorInfo {
  architecture: string;
  cores: number;
  frequencyGhz: number;
  name: string;
}

export interface MemoryInfo {
  totalBytes: number;
  totalDisplay: string;
}

export interface StorageInfo {
  disks: DiskInfo[];
}

export interface DiskInfo {
  availableBytes: number;
  availableDisplay: string;
  diskType: string;
  mountPoint: string;
  name: string;
  totalBytes: number;
  totalDisplay: string;
  usedDisplay: string;
}
