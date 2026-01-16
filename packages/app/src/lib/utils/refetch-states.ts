import { systemInfoState } from "$lib/states/system-info.svelte";
import { operationalState } from "$lib/states/operational.svelte";

export function refetchStates() {
  systemInfoState.fetchSystemInfo();
  void operationalState.refresh();
}
