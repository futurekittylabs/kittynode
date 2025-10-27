import { systemInfoStore } from "$states/systemInfo.svelte";
import { operationalStateStore } from "$states/operationalState.svelte";

export function refetchStores() {
  systemInfoStore.fetchSystemInfo();
  void operationalStateStore.refresh();
}
