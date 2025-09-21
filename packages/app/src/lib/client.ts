import { invoke } from "@tauri-apps/api/core";

import type {
  KittynodeConfig,
  OperationalState,
  Package,
  PackageConfig,
  PortBinding,
  SystemInfo,
} from "$lib/types";

type RawKittynodeConfig = {
  capabilities?: string[];
  server_url?: string;
  onboarding_completed?: boolean;
  auto_start_docker?: boolean;
};

type RawOperationalState = {
  mode: OperationalState["mode"];
  docker_running: boolean;
  can_install: boolean;
  can_manage: boolean;
  diagnostics?: string[];
};

type RawPackage = {
  name: string;
  description: string;
  network_name: string;
  default_config: PackageConfig;
  containers: RawContainer[];
};

type RawContainer = {
  name: string;
  image: string;
  cmd: string[];
  port_bindings: Record<string, RawPortBinding[]>;
  volume_bindings: RawBinding[];
  file_bindings: RawBinding[];
};

type RawBinding = {
  source: string;
  destination: string;
  options?: string;
};

type RawPortBinding = {
  host_ip?: string | null;
  host_port?: string | null;
};

type RawPackagesResponse = Record<string, RawPackage>;

type RawSystemInfo = {
  processor: {
    name: string;
    cores: number;
    frequency_ghz: number;
    architecture: string;
  };
  memory: {
    total_bytes: number;
    total_display: string;
  };
  storage: {
    disks: RawDiskInfo[];
  };
};

type RawDiskInfo = {
  name: string;
  mount_point: string;
  total_bytes: number;
  available_bytes: number;
  total_display: string;
  used_display: string;
  available_display: string;
  disk_type: string;
};

function transformConfig(
  raw: RawKittynodeConfig | null | undefined,
): KittynodeConfig {
  return {
    capabilities: raw?.capabilities ?? [],
    serverUrl: raw?.server_url ?? "",
    onboardingCompleted: raw?.onboarding_completed ?? false,
    autoStartDocker: raw?.auto_start_docker ?? false,
  };
}

function transformOperationalState(raw: RawOperationalState): OperationalState {
  return {
    mode: raw.mode,
    dockerRunning: raw.docker_running,
    canInstall: raw.can_install,
    canManage: raw.can_manage,
    diagnostics: raw.diagnostics ?? [],
  };
}

function transformPackages(raw: RawPackagesResponse): Record<string, Package> {
  return Object.fromEntries(
    Object.entries(raw).map(([name, pkg]) => [name, transformPackage(pkg)]),
  );
}

function transformPackage(raw: RawPackage): Package {
  return {
    name: raw.name,
    description: raw.description,
    networkName: raw.network_name,
    defaultConfig: raw.default_config,
    containers: raw.containers.map(transformContainer),
  };
}

function transformContainer(raw: RawContainer): Package["containers"][number] {
  return {
    name: raw.name,
    image: raw.image,
    cmd: [...raw.cmd],
    portBindings: transformPortBindings(raw.port_bindings),
    volumeBindings: raw.volume_bindings.map(transformBinding),
    fileBindings: raw.file_bindings.map(transformBinding),
  };
}

function transformBinding(
  raw: RawBinding,
): Package["containers"][number]["volumeBindings"][number] {
  return {
    source: raw.source,
    destination: raw.destination,
    options: raw.options,
  };
}

function transformPortBindings(
  raw: Record<string, RawPortBinding[]>,
): Record<string, PortBinding[]> {
  return Object.fromEntries(
    Object.entries(raw).map(([key, bindings]) => [
      key,
      bindings.map(transformPortBinding),
    ]),
  );
}

function transformPortBinding(raw: RawPortBinding): PortBinding {
  return {
    hostIp: raw.host_ip ?? null,
    hostPort: raw.host_port ?? null,
  };
}

function transformSystemInfo(raw: RawSystemInfo): SystemInfo {
  return {
    processor: {
      name: raw.processor.name,
      cores: raw.processor.cores,
      frequencyGhz: raw.processor.frequency_ghz,
      architecture: raw.processor.architecture,
    },
    memory: {
      totalBytes: raw.memory.total_bytes,
      totalDisplay: raw.memory.total_display,
    },
    storage: {
      disks: raw.storage.disks.map(transformDiskInfo),
    },
  };
}

function transformDiskInfo(
  raw: RawDiskInfo,
): SystemInfo["storage"]["disks"][number] {
  return {
    name: raw.name,
    mountPoint: raw.mount_point,
    totalBytes: raw.total_bytes,
    availableBytes: raw.available_bytes,
    totalDisplay: raw.total_display,
    usedDisplay: raw.used_display,
    availableDisplay: raw.available_display,
    diskType: raw.disk_type,
  };
}

export type DockerStartStatus =
  | "running"
  | "disabled"
  | "already_started"
  | "starting";

export interface LatestManifest {
  version: string;
}

export const coreClient = {
  getPackages(): Promise<Record<string, Package>> {
    return invoke<RawPackagesResponse>("get_packages").then(transformPackages);
  },

  getInstalledPackages(): Promise<Package[]> {
    return invoke<RawPackage[]>("get_installed_packages").then((packages) =>
      packages.map(transformPackage),
    );
  },

  installPackage(name: string): Promise<void> {
    return invoke<void>("install_package", { name });
  },

  deletePackage(name: string, includeImages: boolean): Promise<void> {
    return invoke<void>("delete_package", { name, includeImages });
  },

  getContainerLogs(
    containerName: string,
    tailLines: number | null,
  ): Promise<string[]> {
    return invoke<string[]>("get_container_logs", { containerName, tailLines });
  },

  getOperationalState(): Promise<OperationalState> {
    return invoke<RawOperationalState>("get_operational_state").then(
      transformOperationalState,
    );
  },

  startDockerIfNeeded(): Promise<DockerStartStatus> {
    return invoke<DockerStartStatus>("start_docker_if_needed");
  },

  initKittynode(): Promise<void> {
    return invoke<void>("init_kittynode");
  },

  setOnboardingCompleted(completed: boolean): Promise<void> {
    return invoke<void>("set_onboarding_completed", { completed });
  },

  getOnboardingCompleted(): Promise<boolean> {
    return invoke<boolean>("get_onboarding_completed");
  },

  getConfig(): Promise<KittynodeConfig> {
    return invoke<RawKittynodeConfig>("get_config").then(transformConfig);
  },

  setAutoStartDocker(enabled: boolean): Promise<void> {
    return invoke<void>("set_auto_start_docker", { enabled });
  },

  setServerUrl(endpoint: string): Promise<void> {
    return invoke<void>("set_server_url", { endpoint });
  },

  getSystemInfo(): Promise<SystemInfo> {
    return invoke<RawSystemInfo>("system_info").then(transformSystemInfo);
  },

  getPackageConfig(name: string): Promise<PackageConfig> {
    return invoke<PackageConfig>("get_package_config", { name });
  },

  updatePackageConfig(name: string, config: PackageConfig): Promise<void> {
    return invoke<void>("update_package_config", { name, config });
  },

  deleteKittynode(): Promise<void> {
    return invoke<void>("delete_kittynode");
  },

  restartApp(): Promise<void> {
    return invoke<void>("restart_app");
  },

  fetchLatestManifest(url: string): Promise<LatestManifest> {
    return invoke<LatestManifest>("fetch_latest_manifest", { url });
  },
};
