export interface SystemInfo {
  processor: ProcessorInfo;
  memory: MemoryInfo;
  storage: StorageInfo;
}

export interface ProcessorInfo {
  name: string;
  cores: number;
  frequencyGhz: number;
  architecture: string;
}

export interface MemoryInfo {
  totalBytes: number;
  totalDisplay: string;
}

export interface StorageInfo {
  disks: DiskInfo[];
}

export interface DiskInfo {
  name: string;
  mountPoint: string;
  totalBytes: number;
  availableBytes: number;
  totalDisplay: string;
  usedDisplay: string;
  availableDisplay: string;
  diskType: string;
}
