import { systemInfoStore } from "$stores/systemInfo.svelte";
import { operationalStateStore } from "$stores/operationalState.svelte";

export function refetchStores() {
  systemInfoStore.fetchSystemInfo();
  void operationalStateStore.refresh();
}
