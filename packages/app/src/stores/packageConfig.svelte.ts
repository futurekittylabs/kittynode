import { invoke } from "@tauri-apps/api/core";

interface PackageConfig {
  values: Record<string, string>;
}

export const packageConfigStore = {
  async getConfig(packageName: string): Promise<PackageConfig> {
    return await invoke("get_package_config", { name: packageName });
  },

  async updateConfig(
    packageName: string,
    config: PackageConfig,
  ): Promise<void> {
    await invoke("update_package_config", { name: packageName, config });
  },
};
