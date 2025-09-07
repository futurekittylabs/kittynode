<script lang="ts">
import { onMount } from "svelte";
import { remoteAccessStore } from "$stores/remoteAccess.svelte";
import { systemInfoStore } from "$stores/systemInfo.svelte";
import { dockerStatus } from "$stores/dockerStatus.svelte";
import { Skeleton } from "$lib/components/ui/skeleton";
import { Progress } from "$lib/components/ui/progress";
import * as Card from "$lib/components/ui/card";
import { Button } from "$lib/components/ui/button";
import {
  Cpu,
  HardDrive,
  Activity,
  Server,
  Wifi,
  WifiOff,
  RefreshCw,
  CheckCircle2,
  AlertCircle,
  Globe,
} from "@lucide/svelte";

function calculateUsagePercentage(used: number, total: number): number {
  return Math.round((used / total) * 100);
}

function fetchSystemInfo() {
  systemInfoStore.fetchSystemInfo();
}

const getUsageColor = (percentage: number) => {
  if (percentage < 50) return "text-green-500";
  if (percentage < 80) return "text-yellow-500";
  return "text-red-500";
};

onMount(() => {
  if (!systemInfoStore.systemInfo) {
    fetchSystemInfo();
  }
  dockerStatus.startPolling();

  return () => {
    dockerStatus.stopPolling();
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
    <Button 
      size="sm" 
      variant="outline"
      onclick={fetchSystemInfo}
    >
      <RefreshCw class="h-4 w-4 mr-1" />
      Refresh
    </Button>
  </div>

  <!-- Status Cards -->
  <div class="grid gap-4 md:grid-cols-3">
    <Card.Root>
      <Card.Header class="pb-3">
        <Card.Title class="text-sm font-medium">Docker Status</Card.Title>
      </Card.Header>
      <Card.Content>
        <div class="flex items-center space-x-2">
          {#if dockerStatus.isRunning}
            <CheckCircle2 class="h-4 w-4 text-green-500" />
            <span class="text-sm font-medium">Running</span>
          {:else}
            <AlertCircle class="h-4 w-4 text-yellow-500" />
            <span class="text-sm font-medium">Not Running</span>
          {/if}
        </div>
      </Card.Content>
    </Card.Root>

    <Card.Root>
      <Card.Header class="pb-3">
        <Card.Title class="text-sm font-medium">Remote Access</Card.Title>
      </Card.Header>
      <Card.Content>
        <div class="flex items-center space-x-2">
          {#if remoteAccessStore.remoteAccess}
            <Globe class="h-4 w-4 text-green-500" />
            <span class="text-sm font-medium">Enabled</span>
          {:else}
            <WifiOff class="h-4 w-4 text-muted-foreground" />
            <span class="text-sm font-medium">Disabled</span>
          {/if}
        </div>
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

  {#if systemInfoStore.systemInfo}
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
            <p class="text-sm font-medium">{systemInfoStore.systemInfo.processor.name}</p>
          </div>
          <div class="grid grid-cols-2 gap-4 text-sm">
            <div>
              <p class="text-muted-foreground">Cores</p>
              <p class="font-medium">{systemInfoStore.systemInfo.processor.cores}</p>
            </div>
            <div>
              <p class="text-muted-foreground">Frequency</p>
              <p class="font-medium">{systemInfoStore.systemInfo.processor.frequency_ghz.toFixed(2)} GHz</p>
            </div>
          </div>
          <div>
            <p class="text-sm text-muted-foreground">Architecture</p>
            <p class="text-sm font-medium">{systemInfoStore.systemInfo.processor.architecture}</p>
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
            <p class="text-2xl font-bold">{systemInfoStore.systemInfo.memory.total_display}</p>
          </div>
          <div class="pt-2">
            <p class="text-xs text-muted-foreground">
              Memory is managed automatically by the system
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
        {#each systemInfoStore.systemInfo.storage.disks as disk}
          {@const usagePercent = calculateUsagePercentage(
            disk.total_bytes - disk.available_bytes,
            disk.total_bytes
          )}
          <div class="space-y-2">
            <div class="flex items-center justify-between">
              <div>
                <p class="text-sm font-medium">{disk.name}</p>
                <p class="text-xs text-muted-foreground">{disk.mount_point} • {disk.disk_type}</p>
              </div>
              <span class="text-sm font-medium {getUsageColor(usagePercent)}">
                {usagePercent}%
              </span>
            </div>
            <Progress value={usagePercent} max={100} />
            <div class="flex justify-between text-xs text-muted-foreground">
              <span>{disk.available_display} free</span>
              <span>{disk.total_display} total</span>
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