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

export type InstallStatus = "notInstalled" | "partiallyInstalled" | "installed";
export type RuntimeStatus = "notRunning" | "partiallyRunning" | "running";

export interface PackageState {
  install: InstallStatus;
  runtime: RuntimeStatus;
  configPresent: boolean;
  missingContainers: string[];
}
