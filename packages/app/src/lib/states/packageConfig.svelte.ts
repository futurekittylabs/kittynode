import { coreClient } from "$lib/client";
import type { PackageConfig } from "$lib/types";
import { ethereumNetworkState } from "$lib/states/ethereumNetwork.svelte";

export const packageConfigState = {
  async getConfig(packageName: string): Promise<PackageConfig> {
    return await coreClient.getPackageConfig(packageName);
  },

  async updateConfig(
    packageName: string,
    config: PackageConfig,
  ): Promise<void> {
    await coreClient.updatePackageConfig(packageName, config);
    ethereumNetworkState.handleConfigUpdated(packageName);
  },
};
