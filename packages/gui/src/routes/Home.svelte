<script lang="ts">
import { onMount, onDestroy } from "svelte";
import type { Package } from "$lib/types";
import { Button } from "$lib/components/ui/button";
import * as Card from "$lib/components/ui/card";
import { platform } from "@tauri-apps/plugin-os";
import { serverUrlStore } from "$stores/serverUrl.svelte";
import { systemInfoStore } from "$stores/systemInfo.svelte";
import { packagesStore } from "$stores/packages.svelte";
import { dockerStatus } from "$stores/dockerStatus.svelte";
import { goto } from "$app/navigation";
import {
  Package2,
  Play,
  Square,
  Settings2,
  CheckCircle2,
  AlertCircle,
  Info,
  Activity,
  Server,
  HardDrive,
  Cpu,
  ArrowRight,
  Download,
} from "@lucide/svelte";

let installingPackages = $state<Set<string>>(new Set());

function managePackage(packageName: string) {
  goto(`/node/${packageName}`);
}

async function installPackage(packageName: string) {
  installingPackages.add(packageName);
  try {
    await packagesStore.installPackage(packageName);
  } catch (error) {
    console.error("Failed to install package:", error);
  } finally {
    installingPackages.delete(packageName);
  }
}

function isMobileAndLocal() {
  return (
    ["ios", "android"].includes(platform()) && serverUrlStore.serverUrl === ""
  );
}

const formatBytes = (bytes: number): string => {
  const sizes = ["B", "KB", "MB", "GB", "TB"];
  if (bytes === 0) return "0 B";
  const i = Math.floor(Math.log(bytes) / Math.log(1024));
  return `${Math.round((bytes / 1024 ** i) * 100) / 100} ${sizes[i]}`;
};

onMount(async () => {
  if (!systemInfoStore.systemInfo) systemInfoStore.fetchSystemInfo();
  dockerStatus.startPolling();

  if (!isMobileAndLocal()) {
    await packagesStore.loadPackages();
    await packagesStore.loadInstalledPackages();
  }
});

onDestroy(() => {
  dockerStatus.stopPolling();
});
</script>

<div class="space-y-6">
  <div>
    <h2 class="text-3xl font-bold tracking-tight">Dashboard</h2>
    <p class="text-muted-foreground">
      Manage your blockchain infrastructure
    </p>
  </div>

  <div class="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
    <Card.Root>
      <Card.Header class="pb-3">
        <Card.Title class="text-sm font-medium flex items-center justify-between">
          Docker Status
          <Server class="h-4 w-4 text-muted-foreground" />
        </Card.Title>
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
        <Card.Title class="text-sm font-medium flex items-center justify-between">
          Active Packages
          <Package2 class="h-4 w-4 text-muted-foreground" />
        </Card.Title>
      </Card.Header>
      <Card.Content>
        <div class="text-2xl font-bold">
          {packagesStore.installedPackages.length}
        </div>
        <p class="text-xs text-muted-foreground">
          of {Object.keys(packagesStore.packages).length} available
        </p>
      </Card.Content>
    </Card.Root>

    {#if systemInfoStore.systemInfo}
      <Card.Root>
        <Card.Header class="pb-3">
          <Card.Title class="text-sm font-medium flex items-center justify-between">
            Processor
            <Cpu class="h-4 w-4 text-muted-foreground" />
          </Card.Title>
        </Card.Header>
        <Card.Content>
          <div class="text-2xl font-bold">
            {systemInfoStore.systemInfo.processor.cores} cores
          </div>
          <p class="text-xs text-muted-foreground">
            {systemInfoStore.systemInfo.processor.name}
          </p>
        </Card.Content>
      </Card.Root>

      <Card.Root>
        <Card.Header class="pb-3">
          <Card.Title class="text-sm font-medium flex items-center justify-between">
            Memory
            <HardDrive class="h-4 w-4 text-muted-foreground" />
          </Card.Title>
        </Card.Header>
        <Card.Content>
          <div class="text-2xl font-bold">
            {systemInfoStore.systemInfo.memory.total_display}
          </div>
          <p class="text-xs text-muted-foreground">
            Total System Memory
          </p>
        </Card.Content>
      </Card.Root>
    {/if}
  </div>

  {#if packagesStore.installedPackages.length > 0}
    <div class="space-y-4">
      <h3 class="text-xl font-semibold">Running Nodes</h3>
      <div class="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
        {#each packagesStore.installedPackages as pkg}
          <Card.Root>
            <Card.Header>
              <div class="flex items-start justify-between">
                <div class="flex items-start space-x-3">
                  <Activity class="h-5 w-5 text-green-500 mt-0.5" />
                  <div>
                    <Card.Title class="text-base">{pkg.name}</Card.Title>
                    <Card.Description class="mt-1">
                      {pkg.description}
                    </Card.Description>
                  </div>
                </div>
                <div class="flex items-center space-x-1 rounded-full bg-green-500/10 px-2 py-1">
                  <div class="h-2 w-2 rounded-full bg-green-500 animate-pulse"></div>
                  <span class="text-xs font-medium text-green-700 dark:text-green-400">Running</span>
                </div>
              </div>
            </Card.Header>
            <Card.Footer>
              <Button 
                size="sm"
                variant="default"
                onclick={() => managePackage(pkg.name)}
                class="w-full"
              >
                <Settings2 class="h-4 w-4 mr-1" />
                Manage Node
              </Button>
            </Card.Footer>
          </Card.Root>
        {/each}
      </div>
    </div>
  {/if}

  <div class="space-y-4">
    <div class="flex items-center justify-between">
      <h3 class="text-xl font-semibold">Available Packages</h3>
      <Button
        size="sm"
        variant="outline"
        onclick={() => goto('/packages')}
      >
        View All
        <ArrowRight class="h-4 w-4 ml-1" />
      </Button>
    </div>
    
    {#if Object.keys(packagesStore.packages).length > 0}
      {@const availablePackages = Object.entries(packagesStore.packages)
        .filter(([name]) => !packagesStore.isInstalled(name))
        .slice(0, 3)}
      
      {#if availablePackages.length > 0}
        <div class="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
          {#each availablePackages as [name, pkg]}
            {@const isInstalling = installingPackages.has(name)}
            <Card.Root>
              <Card.Header>
                <div class="flex items-start space-x-3">
                  <Package2 class="h-5 w-5 text-muted-foreground mt-0.5" />
                  <div class="flex-1">
                    <Card.Title class="text-base">{name}</Card.Title>
                    <Card.Description class="mt-1">
                      {pkg.description}
                    </Card.Description>
                  </div>
                </div>
              </Card.Header>
              
              <Card.Footer>
                <Button 
                  size="sm"
                  variant="default"
                  onclick={() => installPackage(name)}
                  disabled={!dockerStatus.isRunning || isInstalling}
                  class="w-full"
                >
                  {#if isInstalling}
                    <div class="h-4 w-4 mr-1 animate-spin rounded-full border-2 border-current border-t-transparent"></div>
                    Installing...
                  {:else}
                    <Download class="h-4 w-4 mr-1" />
                    Install
                  {/if}
                </Button>
              </Card.Footer>
            </Card.Root>
          {/each}
        </div>
      {:else}
        <Card.Root>
          <Card.Content class="pt-6">
            <p class="text-center text-muted-foreground">All available packages are installed!</p>
          </Card.Content>
        </Card.Root>
      {/if}
    {:else if !dockerStatus.isRunning}
      <Card.Root>
        <Card.Header>
          <Card.Title class="flex items-center space-x-2">
            <Info class="h-5 w-5" />
            <span>Docker Required</span>
          </Card.Title>
        </Card.Header>
        <Card.Content>
          <p class="text-sm text-muted-foreground">
            Docker needs to be running to view and manage packages. Please start Docker Desktop and refresh this page.
          </p>
        </Card.Content>
      </Card.Root>
    {:else}
      <Card.Root>
        <Card.Content class="pt-6">
          <p class="text-center text-muted-foreground">No packages available.</p>
        </Card.Content>
      </Card.Root>
    {/if}
  </div>
</div>
