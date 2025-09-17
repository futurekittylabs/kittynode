export interface Package {
  name: string;
  description: string;
  network_name: string;
  containers: Container[];
}

export interface InstalledPackage {
  package: Package;
  isRunning: boolean;
}

export interface InstalledPackageApi {
  package: Package;
  is_running: boolean;
}

export interface Container {
  name: string;
  image: string;
  cmd: string[];
  port_bindings: Record<string, { host_ip: string; host_port: string }[]>;
  volume_bindings: Binding[];
  file_bindings: Binding[];
}

export interface Binding {
  source: string;
  destination: string;
  options?: string;
}
