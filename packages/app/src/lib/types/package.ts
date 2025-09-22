export interface Package {
  name: string;
  description: string;
  networkName: string;
  defaultConfig: PackageConfig;
  containers: Container[];
}

export interface PackageConfig {
  values: Record<string, string>;
}

export interface Container {
  name: string;
  image: string;
  cmd: string[];
  portBindings: Record<string, PortBinding[]>;
  volumeBindings: Binding[];
  fileBindings: Binding[];
}

export interface PortBinding {
  hostIp?: string | null;
  hostPort?: string | null;
}

export interface Binding {
  source: string;
  destination: string;
  options?: string;
}

export interface PackageRuntimeState {
  running: boolean;
}
