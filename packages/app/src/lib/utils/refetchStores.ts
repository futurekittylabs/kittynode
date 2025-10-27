import { systemInfoStore } from "$lib/states/systemInfo.svelte";
import { operationalStateStore } from "$lib/states/operationalState.svelte";

export function refetchStores() {
  systemInfoStore.fetchSystemInfo();
  void operationalStateStore.refresh();
}
