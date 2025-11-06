import { systemInfoState } from "$lib/states/systemInfo.svelte";
import { operationalState } from "$lib/states/operational.svelte";

export function refetchStates() {
  systemInfoState.fetchSystemInfo();
  void operationalState.refresh();
}
