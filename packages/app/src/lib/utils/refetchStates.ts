import { systemInfoStore } from "$lib/states/systemInfo.svelte";
import { operationalStateStore } from "$lib/states/operationalState.svelte";

export function refetchStates() {
  systemInfoStore.fetchSystemInfo();
  void operationalStateStore.refresh();
}
