import { coreClient } from "$lib/client";
import type { PackageConfig } from "$lib/types";

export const packageConfigStore = {
  async getConfig(packageName: string): Promise<PackageConfig> {
    return await coreClient.getPackageConfig(packageName);
  },

  async updateConfig(
    packageName: string,
    config: PackageConfig,
  ): Promise<void> {
    await coreClient.updatePackageConfig(packageName, config);
  },
};
