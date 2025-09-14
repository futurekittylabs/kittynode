import { systemInfoStore } from "$stores/systemInfo.svelte";

export function refetchStores() {
  systemInfoStore.fetchSystemInfo();
}
