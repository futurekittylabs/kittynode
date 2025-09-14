import { invoke } from "@tauri-apps/api/core";
import { serverUrlStore } from "./serverUrl.svelte";

interface PackageConfig {
  values: Record<string, string>;
}

export const packageConfigStore = {
  async getConfig(packageName: string): Promise<PackageConfig> {
    return await invoke("get_package_config", {
      name: packageName,
      serverUrl: serverUrlStore.serverUrl,
    });
  },

  async updateConfig(
    packageName: string,
    config: PackageConfig,
  ): Promise<void> {
    await invoke("update_package_config", {
      name: packageName,
      config,
      serverUrl: serverUrlStore.serverUrl,
    });
  },
};
