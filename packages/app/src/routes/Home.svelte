<script lang="ts">
import { onMount, onDestroy } from "svelte";
import { Button } from "$lib/components/ui/button";
import * as Card from "$lib/components/ui/card";
import { platform } from "@tauri-apps/plugin-os";
import { serverUrlStore } from "$stores/serverUrl.svelte";
import { systemInfoStore } from "$stores/systemInfo.svelte";
import { packagesStore } from "$stores/packages.svelte";
import { appConfigStore } from "$stores/appConfig.svelte";
import { dockerStatus } from "$stores/dockerStatus.svelte";
import DockerStatusCard from "$lib/components/DockerStatusCard.svelte";
import { goto } from "$app/navigation";
import { usePackageInstaller } from "$lib/composables/usePackageInstaller.svelte";
import {
  Package2,
  Settings2,
  Info,
  Activity,
  HardDrive,
  Cpu,
  ArrowRight,
  Download,
} from "@lucide/svelte";

const { isInstalling, installPackage } = usePackageInstaller();

const catalogState = $derived(packagesStore.catalogState);
const installedState = $derived(packagesStore.installedState);

const totalPackageCount = $derived(
  catalogState.status === "ready"
    ? Object.keys(packagesStore.packages).length
    : null,
);

const installedPackageCount = $derived(
  installedState.status === "ready"
    ? packagesStore.installedPackages.length
    : null,
);

const runningNodes = $derived(
  installedState.status === "ready" ? packagesStore.installedPackages : [],
);

const featuredAvailablePackages = $derived(
  catalogState.status !== "ready" || installedState.status !== "ready"
    ? null
    : Object.entries(catalogState.packages)
        .filter(([name]) => !Object.hasOwn(installedState.packages, name))
        .slice(0, 3),
);

function managePackage(packageName: string) {
  goto(`/node/${packageName}`);
}

function isMobileAndLocal() {
  return (
    ["ios", "android"].includes(platform()) && serverUrlStore.serverUrl === ""
  );
}

function isLocalDesktop() {
  return (
    !["ios", "android"].includes(platform()) && serverUrlStore.serverUrl === ""
  );
}

onMount(async () => {
  if (!systemInfoStore.systemInfo) systemInfoStore.fetchSystemInfo();

  try {
    await appConfigStore.load();
  } catch (e) {
    console.error(`Failed to load Kittynode config: ${e}`);
  }

  // Start Docker if needed (only on first app startup)
  if (isLocalDesktop() && appConfigStore.autoStartDocker) {
    console.info("Attempting Docker auto-start based on user preference");
    await dockerStatus.startDockerIfNeeded();
  }

  const pollingInterval = dockerStatus.isStarting ? 2000 : 5000;
  dockerStatus.startPolling(pollingInterval);

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
      Manage your node infrastructure
    </p>
  </div>

  <div class="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
    <DockerStatusCard showServerIcon={true} />

    <Card.Root>
      <Card.Header class="pb-3">
        <Card.Title class="text-sm font-medium flex items-center justify-between">
          Active Packages
          <Package2 class="h-4 w-4 text-muted-foreground" />
        </Card.Title>
      </Card.Header>
        <Card.Content>
        <div class="text-2xl font-bold">
          {installedPackageCount ?? "--"}
        </div>
        <p class="text-xs text-muted-foreground">
          of {totalPackageCount ?? "--"} available
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

  <div class="space-y-4">
    <h3 class="text-xl font-semibold">Running Nodes</h3>

    {#if installedState.status === "loading" || installedState.status === "idle"}
      <Card.Root>
        <Card.Content>
          <p class="text-sm text-muted-foreground">Checking installed packages...</p>
        </Card.Content>
      </Card.Root>
    {:else if installedState.status === "unavailable"}
      <Card.Root>
        <Card.Content>
          <p class="text-sm text-muted-foreground">
            Docker needs to be running to manage installed nodes. Start Docker Desktop to continue.
          </p>
        </Card.Content>
      </Card.Root>
    {:else if installedState.status === "error"}
      <Card.Root>
        <Card.Content class="flex items-center justify-between">
          <p class="text-sm text-muted-foreground">
            Failed to load installed packages.
          </p>
          <Button size="sm" variant="outline" onclick={() => packagesStore.loadInstalledPackages({ force: true })}>
            Retry
          </Button>
        </Card.Content>
      </Card.Root>
    {:else if runningNodes.length > 0}
      <div class="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
        {#each runningNodes as pkg}
          <Card.Root>
            <Card.Header>
              <div class="flex items-start justify-between">
                <div class="flex items-start gap-3">
                  <div class="shrink-0">
                    <Activity class="w-5 h-5 text-green-500 mt-0.5" />
                  </div>
                  <div class="min-w-0">
                    <Card.Title class="text-base">{pkg.name}</Card.Title>
                    <Card.Description class="mt-1">
                      {pkg.description}
                    </Card.Description>
                  </div>
                </div>
                <div class="flex items-center space-x-1 rounded-full bg-green-500/10 px-2 py-1 shrink-0">
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
                Manage
              </Button>
            </Card.Footer>
          </Card.Root>
        {/each}
      </div>
    {:else}
      <Card.Root>
        <Card.Content>
          <p class="text-sm text-muted-foreground">No nodes installed yet.</p>
        </Card.Content>
      </Card.Root>
    {/if}
  </div>

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

    {#if catalogState.status === "error"}
      <Card.Root>
        <Card.Content class="flex items-center justify-between">
          <p class="text-sm text-muted-foreground">Failed to load available packages.</p>
          <Button size="sm" variant="outline" onclick={() => packagesStore.loadPackages({ force: true })}>
            Retry
          </Button>
        </Card.Content>
      </Card.Root>
    {:else if installedState.status === "error"}
      <Card.Root>
        <Card.Content class="flex items-center justify-between">
          <p class="text-sm text-muted-foreground">Failed to confirm installed packages.</p>
          <Button size="sm" variant="outline" onclick={() => packagesStore.loadInstalledPackages({ force: true })}>
            Retry
          </Button>
        </Card.Content>
      </Card.Root>
    {:else if installedState.status === "unavailable"}
      <Card.Root>
        <Card.Header>
          <Card.Title class="flex items-center space-x-2">
            <Info class="h-5 w-5" />
            <span>Docker Required</span>
          </Card.Title>
        </Card.Header>
        <Card.Content>
          <p class="text-sm text-muted-foreground">
            Docker needs to be running to view and manage packages. Please start Docker Desktop and return to this page.
          </p>
        </Card.Content>
      </Card.Root>
    {:else if catalogState.status !== "ready" || installedState.status !== "ready"}
      <Card.Root>
        <Card.Content>
          <p class="text-sm text-muted-foreground">Loading packages...</p>
        </Card.Content>
      </Card.Root>
    {:else if featuredAvailablePackages && featuredAvailablePackages.length > 0}
      <div class="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
        {#each featuredAvailablePackages as [name, pkg]}
          {@const status = packagesStore.installationStatus(name)}
          {@const isInstallingPackage = isInstalling(name)}
          {@const disabled =
            dockerStatus.isRunning !== true || status !== "available" || isInstallingPackage}

          <Card.Root>
            <Card.Header>
              <div class="flex items-start gap-3">
                <div class="shrink-0">
                  <Package2 class="w-5 h-5 text-muted-foreground mt-0.5" />
                </div>
                <div class="min-w-0 flex-1">
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
                onclick={async () => {
                  const installed = await installPackage(name);
                  if (installed) {
                    managePackage(name);
                  }
                }}
                disabled={disabled}
                class="w-full"
              >
                {#if isInstallingPackage}
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
        <Card.Content>
          <p class="text-center text-muted-foreground">All available packages are installed!</p>
        </Card.Content>
      </Card.Root>
    {/if}
  </div>
</div>
