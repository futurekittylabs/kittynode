export interface Package {
  containers: Container[];
  defaultConfig: PackageConfig;
  description: string;
  name: string;
  networkName: string;
}

export interface PackageConfig {
  values: Record<string, string>;
}

export interface Container {
  cmd: string[];
  fileBindings: Binding[];
  image: string;
  name: string;
  portBindings: Record<string, PortBinding[]>;
  volumeBindings: Binding[];
}

export interface PortBinding {
  hostIp?: string | null;
  hostPort?: string | null;
}

export interface Binding {
  destination: string;
  options?: string;
  source: string;
}

export type InstallStatus = "notInstalled" | "partiallyInstalled" | "installed";
export type RuntimeStatus = "notRunning" | "partiallyRunning" | "running";

export interface PackageState {
  configPresent: boolean;
  install: InstallStatus;
  missingContainers: string[];
  runtime: RuntimeStatus;
}
