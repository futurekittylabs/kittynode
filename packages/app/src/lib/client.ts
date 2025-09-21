import { invoke } from "@tauri-apps/api/core";

import type {
  KittynodeConfig,
  OperationalState,
  Package,
  PackageConfig,
  SystemInfo,
} from "$lib/types";

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
    return invoke("get_packages");
  },

  getInstalledPackages(): Promise<Package[]> {
    return invoke("get_installed_packages");
  },

  installPackage(name: string): Promise<void> {
    return invoke("install_package", { name });
  },

  deletePackage(name: string, includeImages: boolean): Promise<void> {
    return invoke("delete_package", { name, includeImages });
  },

  getContainerLogs(
    containerName: string,
    tailLines: number | null,
  ): Promise<string[]> {
    return invoke("get_container_logs", { containerName, tailLines });
  },

  getOperationalState(): Promise<OperationalState> {
    return invoke("get_operational_state");
  },

  startDockerIfNeeded(): Promise<DockerStartStatus> {
    return invoke("start_docker_if_needed");
  },

  initKittynode(): Promise<void> {
    return invoke("init_kittynode");
  },

  setOnboardingCompleted(completed: boolean): Promise<void> {
    return invoke("set_onboarding_completed", { completed });
  },

  getOnboardingCompleted(): Promise<boolean> {
    return invoke("get_onboarding_completed");
  },

  getConfig(): Promise<KittynodeConfig> {
    return invoke("get_config");
  },

  setAutoStartDocker(enabled: boolean): Promise<void> {
    return invoke("set_auto_start_docker", { enabled });
  },

  setServerUrl(endpoint: string): Promise<void> {
    return invoke("set_server_url", { endpoint });
  },

  getSystemInfo(): Promise<SystemInfo> {
    return invoke("system_info");
  },

  getPackageConfig(name: string): Promise<PackageConfig> {
    return invoke("get_package_config", { name });
  },

  updatePackageConfig(name: string, config: PackageConfig): Promise<void> {
    return invoke("update_package_config", { name, config });
  },

  deleteKittynode(): Promise<void> {
    return invoke("delete_kittynode");
  },

  restartApp(): Promise<void> {
    return invoke("restart_app");
  },

  fetchLatestManifest(url: string): Promise<LatestManifest> {
    return invoke("fetch_latest_manifest", { url });
  },
};
