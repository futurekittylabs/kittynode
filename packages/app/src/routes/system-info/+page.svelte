<script lang="ts">
import { onMount } from "svelte";
import { serverUrlState } from "$lib/states/serverUrl.svelte";
import { systemInfoState } from "$lib/states/systemInfo.svelte";
import { operationalState } from "$lib/states/operational.svelte";
import { Skeleton } from "$lib/components/ui/skeleton";
import { Progress } from "$lib/components/ui/progress";
import * as Card from "$lib/components/ui/card";
import { Button } from "$lib/components/ui/button";
import DockerStatusCard from "$lib/components/DockerStatusCard.svelte";
import {
  Cpu,
  HardDrive,
  Activity,
  Server,
  RefreshCw,
  Globe,
  WifiOff,
} from "@lucide/svelte";

function calculateUsagePercentage(used: number, total: number): number {
  return Math.round((used / total) * 100);
}

function fetchSystemInfo() {
  systemInfoState.fetchSystemInfo();
}

onMount(() => {
  if (!systemInfoState.systemInfo) {
    fetchSystemInfo();
  }
  operationalState.startPolling();

  return () => {
    operationalState.stopPolling();
  };
});
</script>

<div class="space-y-6">
  <div class="flex items-center justify-between">
    <div>
      <h2 class="text-3xl font-bold tracking-tight">System Information</h2>
      <p class="text-muted-foreground">
        Monitor your system resources and capabilities
      </p>
    </div>
    <Button size="sm" variant="outline" onclick={fetchSystemInfo}>
      <RefreshCw class="h-4 w-4 mr-1" />
      Refresh
    </Button>
  </div>

  <!-- Status Cards -->
  <div class="grid gap-4 md:grid-cols-3">
    <DockerStatusCard />

    <Card.Root>
      <Card.Header class="pb-3">
        <Card.Title class="text-sm font-medium">Remote Server</Card.Title>
      </Card.Header>
      <Card.Content class="space-y-2">
        <div class="flex items-center space-x-2">
          {#if serverUrlState.serverUrl}
            <Globe class="h-4 w-4 text-green-500" />
            <span class="text-sm font-medium">Connected</span>
          {:else}
            <WifiOff class="h-4 w-4 text-muted-foreground" />
            <span class="text-sm font-medium">Not connected</span>
          {/if}
        </div>
        {#if serverUrlState.serverUrl}
          <p class="text-xs text-muted-foreground break-all">
            {serverUrlState.serverUrl}
          </p>
        {/if}
      </Card.Content>
    </Card.Root>

    <Card.Root>
      <Card.Header class="pb-3">
        <Card.Title class="text-sm font-medium">System Health</Card.Title>
      </Card.Header>
      <Card.Content>
        <div class="flex items-center space-x-2">
          <Activity class="h-4 w-4 text-green-500" />
          <span class="text-sm font-medium">Healthy</span>
        </div>
      </Card.Content>
    </Card.Root>
  </div>

  {#if systemInfoState.systemInfo}
    <!-- Hardware Information -->
    <div class="grid gap-4 md:grid-cols-2">
      <Card.Root>
        <Card.Header>
          <Card.Title class="flex items-center gap-2">
            <Cpu class="h-5 w-5" />
            Processor
          </Card.Title>
        </Card.Header>
        <Card.Content class="space-y-2">
          <div>
            <p class="text-sm font-medium">
              {systemInfoState.systemInfo.processor.name}
            </p>
          </div>
          <div class="grid grid-cols-2 gap-4 text-sm">
            <div>
              <p class="text-muted-foreground">Cores</p>
              <p class="font-medium">
                {systemInfoState.systemInfo.processor.cores}
              </p>
            </div>
            <div>
              <p class="text-muted-foreground">Frequency</p>
              <p class="font-medium">
                {systemInfoState.systemInfo.processor.frequencyGhz.toFixed(2)} GHz
              </p>
            </div>
          </div>
          <div>
            <p class="text-sm text-muted-foreground">Architecture</p>
            <p class="text-sm font-medium">
              {systemInfoState.systemInfo.processor.architecture}
            </p>
          </div>
        </Card.Content>
      </Card.Root>

      <Card.Root>
        <Card.Header>
          <Card.Title class="flex items-center gap-2">
            <Server class="h-5 w-5" />
            Memory
          </Card.Title>
        </Card.Header>
        <Card.Content class="space-y-2">
          <div>
            <p class="text-sm text-muted-foreground">Total System Memory</p>
            <p class="text-2xl font-bold">
              {systemInfoState.systemInfo.memory.totalDisplay}
            </p>
          </div>
        </Card.Content>
      </Card.Root>
    </div>

    <!-- Storage Information -->
    <Card.Root>
      <Card.Header>
        <Card.Title class="flex items-center gap-2">
          <HardDrive class="h-5 w-5" />
          Storage Devices
        </Card.Title>
        <Card.Description>
          Available storage across all mounted drives
        </Card.Description>
      </Card.Header>
      <Card.Content class="space-y-4">
        {#each systemInfoState.systemInfo.storage.disks as disk}
          {@const usagePercent = calculateUsagePercentage(
            disk.totalBytes - disk.availableBytes,
            disk.totalBytes,
          )}
          <div class="space-y-2">
            <div class="flex items-center justify-between">
              <div>
                <p class="text-sm font-medium">{disk.name}</p>
              </div>
            </div>
            <Progress value={usagePercent} max={100} />
            <div class="flex justify-between text-xs text-muted-foreground">
              <span>{disk.usedDisplay} of {disk.totalDisplay} used</span>
            </div>
          </div>
        {/each}
      </Card.Content>
    </Card.Root>
  {:else}
    <Card.Root>
      <Card.Content class="pt-6">
        <div class="space-y-2">
          <Skeleton class="h-4 w-full" />
          <Skeleton class="h-4 w-3/4" />
          <Skeleton class="h-4 w-1/2" />
        </div>
      </Card.Content>
    </Card.Root>
  {/if}
</div>
