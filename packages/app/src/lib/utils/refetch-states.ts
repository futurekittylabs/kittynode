import { operationalState } from "$lib/states/operational.svelte";
import { systemInfoState } from "$lib/states/system-info.svelte";

export function refetchStates() {
  systemInfoState.fetchSystemInfo();
  void operationalState.refresh();
}
