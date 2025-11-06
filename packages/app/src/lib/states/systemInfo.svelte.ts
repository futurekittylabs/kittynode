import { coreClient } from "$lib/client";
import type { SystemInfo } from "$lib/types/system_info";

let systemInfo = $state<SystemInfo>();

export const systemInfoState = {
  get systemInfo() {
    return systemInfo;
  },
  async fetchSystemInfo() {
    try {
      systemInfo = undefined; // invalidate previous data
      systemInfo = await coreClient.getSystemInfo();
      console.info("Successfully fetched system info.");
    } catch (e) {
      console.error(`Failed to fetch system info: ${e}.`);
    }
  },
};
