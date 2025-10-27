<script lang="ts">
import { onMount, onDestroy } from "svelte";
import { Button } from "$lib/components/ui/button";
import * as Card from "$lib/components/ui/card";
import { platform } from "@tauri-apps/plugin-os";
import { systemInfoStore } from "$lib/states/systemInfo.svelte";
import { packagesStore } from "$lib/states/packages.svelte";
import { appConfigStore } from "$lib/states/appConfig.svelte";
import { serverUrlStore } from "$lib/states/serverUrl.svelte";
import { operationalStateStore } from "$lib/states/operationalState.svelte";
import DockerStatusCard from "$lib/components/DockerStatusCard.svelte";
import { goto } from "$app/navigation";
import { usePackageInstaller } from "$lib/composables/usePackageInstaller.svelte";
import {
  defaultEthereumNetwork,
  ethereumNetworks,
} from "$lib/constants/ethereumNetworks";
import * as Select from "$lib/components/ui/select";
import { runtimeOverviewStore } from "$lib/states/runtimeOverview.svelte";
import { coreClient } from "$lib/client";
import {
  Package2,
  Settings2,
  Info,
  Activity,
  HardDrive,
  Cpu,
  ArrowRight,
  Download,
  CirclePause,
  LoaderCircle,
  CircleAlert,
} from "@lucide/svelte";
import { formatPackageName } from "$lib/utils";
import { packageConfigStore } from "$lib/states/packageConfig.svelte";

const { isInstalling, installPackage } = usePackageInstaller();

let selectedEthereumNetwork = $state(defaultEthereumNetwork);

const defaultEthereumNetworkLabel =
  ethereumNetworks.find((option) => option.value === defaultEthereumNetwork)
    ?.label ?? defaultEthereumNetwork;

const selectedEthereumNetworkLabel = $derived(
  ethereumNetworks.find((option) => option.value === selectedEthereumNetwork)
    ?.label || defaultEthereumNetworkLabel,
);

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

const installedPackagesList = $derived(
  installedState.status === "ready" ? packagesStore.installedPackages : [],
);
let ethereumNetworkLabel = $state<string | null>(null);
let ethereumValidatorInstalled = $state<boolean | null>(null);

$effect(() => {
  if (installedState.status !== "ready") {
    ethereumNetworkLabel = null;
    return;
  }
  const hasEth = installedPackagesList.some((p) => p.name === "ethereum");
  if (!hasEth) {
    ethereumNetworkLabel = null;
    ethereumValidatorInstalled = null;
    return;
  }
  (async () => {
    try {
      const cfg = await packageConfigStore.getConfig("ethereum");
      const network = cfg.values.network;
      if (network) {
        if (network === "hoodi") ethereumNetworkLabel = "Hoodi";
        else if (network === "mainnet") ethereumNetworkLabel = "Mainnet";
        else if (network === "sepolia") ethereumNetworkLabel = "Sepolia";
        else if (network === "ephemery") ethereumNetworkLabel = "Ephemery";
        else ethereumNetworkLabel = network;
      } else {
        ethereumNetworkLabel = null;
      }
    } catch (e) {
      ethereumNetworkLabel = null;
    }

    // Fetch validator installed status once when we can manage
    try {
      if (operationalStateStore.canManage) {
        ethereumValidatorInstalled = await coreClient.isValidatorInstalled();
      } else {
        ethereumValidatorInstalled = null;
      }
    } catch (e) {
      ethereumValidatorInstalled = null;
    }
  })();
});

const runtimeStatuses = $derived(runtimeOverviewStore.statuses);
const runtimeStatusesLoading = $derived(runtimeOverviewStore.loading);

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
  } catch (error) {
    console.error(`Failed to load Kittynode config: ${error}`);
  }

  await operationalStateStore.refresh();

  if (isLocalDesktop() && appConfigStore.autoStartDocker) {
    console.info("Attempting Docker auto-start based on user preference");
    const result = await operationalStateStore.startDockerIfNeeded();
    if (result.status === "error") {
      console.error(
        `Docker auto-start failed: ${result.error}. Continuing without auto-start.`,
      );
    }
  }

  const pollingInterval = operationalStateStore.isStarting ? 2000 : 5000;
  operationalStateStore.startPolling(pollingInterval);

  if (!isMobileAndLocal()) {
    await packagesStore.loadPackages();
    await packagesStore.syncInstalledPackages();
  }
});

$effect(() => {
  if (installedState.status === "ready") {
    const names = packagesStore.installedPackages.map((pkg) => pkg.name);
    runtimeOverviewStore.sync(names, {
      enabled: names.length > 0,
      pollInterval: operationalStateStore.isStarting ? 2000 : 5000,
    });
  } else {
    runtimeOverviewStore.stop();
  }
});

onDestroy(() => {
  operationalStateStore.stopPolling();
  runtimeOverviewStore.stop();
});
</script>

<div class="space-y-6">
  <div>
    <h2 class="text-3xl font-bold tracking-tight">Dashboard</h2>
    <p class="text-muted-foreground">Manage your node infrastructure</p>
  </div>

  <div class="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
    <DockerStatusCard showServerIcon={true} />

    <Card.Root>
      <Card.Header class="pb-3">
        <Card.Title
          class="text-sm font-medium flex items-center justify-between"
        >
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
          <Card.Title
            class="text-sm font-medium flex items-center justify-between"
          >
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
          <Card.Title
            class="text-sm font-medium flex items-center justify-between"
          >
            Memory
            <HardDrive class="h-4 w-4 text-muted-foreground" />
          </Card.Title>
        </Card.Header>
        <Card.Content>
          <div class="text-2xl font-bold">
            {systemInfoStore.systemInfo.memory.totalDisplay}
          </div>
          <p class="text-xs text-muted-foreground">Total System Memory</p>
        </Card.Content>
      </Card.Root>
    {/if}
  </div>

  <div class="space-y-4">
    <h3 class="text-xl font-semibold">Installed Nodes</h3>

    {#if installedState.status === "loading" || installedState.status === "idle"}
      <Card.Root>
        <Card.Content>
          <p class="text-sm text-muted-foreground">
            Checking installed packages...
          </p>
        </Card.Content>
      </Card.Root>
    {:else if installedState.status === "unavailable"}
      <Card.Root>
        <Card.Content>
          <p class="text-sm text-muted-foreground">
            Docker needs to be running to manage installed nodes. Start Docker
            Desktop to continue.
          </p>
        </Card.Content>
      </Card.Root>
    {:else if installedState.status === "error"}
      <Card.Root>
        <Card.Content class="flex items-center justify-between">
          <p class="text-sm text-muted-foreground">
            Failed to load installed packages.
          </p>
          <Button
            size="sm"
            variant="outline"
            onclick={() => packagesStore.loadInstalledPackages({ force: true })}
          >
            Retry
          </Button>
        </Card.Content>
      </Card.Root>
    {:else if installedPackagesList.length > 0}
      <div class="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
        {#each installedPackagesList as pkg}
          {@const runtimeStatus =
            runtimeStatuses[pkg.name] ??
            (runtimeStatusesLoading ? "checking" : "unknown")}
          <Card.Root>
            <Card.Header>
              <div class="flex items-start justify-between">
                <div class="flex items-start gap-3">
                  <div class="shrink-0">
                    {#if runtimeStatus === "running"}
                      <Activity class="w-5 h-5 text-green-500 mt-0.5" />
                    {:else if runtimeStatus === "stopped"}
                      <CirclePause
                        class="w-5 h-5 text-amber-500 dark:text-amber-200 mt-0.5"
                      />
                    {:else if runtimeStatus === "checking"}
                      <LoaderCircle
                        class="w-5 h-5 text-muted-foreground mt-0.5 animate-spin"
                      />
                    {:else}
                      <CircleAlert
                        class="w-5 h-5 text-muted-foreground mt-0.5"
                      />
                    {/if}
                  </div>
                  <div class="min-w-0">
                    <Card.Title class="text-base">
                      {pkg.name === "ethereum" && ethereumNetworkLabel
                        ? `${formatPackageName(pkg.name)} (${ethereumNetworkLabel})`
                        : formatPackageName(pkg.name)}
                    </Card.Title>
                    <Card.Description class="mt-1">
                      {#if pkg.name === "ethereum" && ethereumValidatorInstalled}
                        Manage your {formatPackageName(pkg.name)} validator.
                      {:else}
                        Manage your {formatPackageName(pkg.name)} node.
                      {/if}
                    </Card.Description>
                  </div>
                </div>
                {#if runtimeStatus === "running"}
                  <div
                    class="flex items-center space-x-1 rounded-full bg-green-500/10 px-2 py-1 shrink-0"
                  >
                    <div
                      class="h-2 w-2 rounded-full bg-green-500 animate-pulse"
                    ></div>
                    <span
                      class="text-xs font-medium text-green-700 dark:text-green-400"
                      >Running</span
                    >
                  </div>
                {:else if runtimeStatus === "stopped"}
                  <div
                    class="flex items-center space-x-1 rounded-full bg-amber-500/10 px-2 py-1 shrink-0"
                  >
                    <div
                      class="h-2 w-2 rounded-full bg-amber-500 dark:bg-amber-200"
                    ></div>
                    <span
                      class="text-xs font-medium text-amber-700 dark:text-amber-200"
                      >Stopped</span
                    >
                  </div>
                {:else if runtimeStatus === "checking"}
                  <div
                    class="flex items-center space-x-1 rounded-full bg-muted px-2 py-1 shrink-0"
                  >
                    <LoaderCircle
                      class="h-3 w-3 animate-spin text-muted-foreground"
                    />
                    <span class="text-xs font-medium text-muted-foreground"
                      >Checking…</span
                    >
                  </div>
                {:else}
                  <div
                    class="flex items-center space-x-1 rounded-full bg-muted px-2 py-1 shrink-0"
                  >
                    <div class="h-2 w-2 rounded-full bg-muted-foreground"></div>
                    <span class="text-xs font-medium text-muted-foreground"
                      >Status unknown</span
                    >
                  </div>
                {/if}
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
      <Button size="sm" variant="outline" onclick={() => goto("/packages")}>
        View All
        <ArrowRight class="h-4 w-4 ml-1" />
      </Button>
    </div>

    {#if catalogState.status === "error"}
      <Card.Root>
        <Card.Content class="flex items-center justify-between">
          <p class="text-sm text-muted-foreground">
            Failed to load available packages.
          </p>
          <Button
            size="sm"
            variant="outline"
            onclick={() => packagesStore.loadPackages({ force: true })}
          >
            Retry
          </Button>
        </Card.Content>
      </Card.Root>
    {:else if installedState.status === "error"}
      <Card.Root>
        <Card.Content class="flex items-center justify-between">
          <p class="text-sm text-muted-foreground">
            Failed to confirm installed packages.
          </p>
          <Button
            size="sm"
            variant="outline"
            onclick={() => packagesStore.loadInstalledPackages({ force: true })}
          >
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
            Docker needs to be running to view and manage packages. Please start
            Docker and return to this page.
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
            !operationalStateStore.canInstall ||
            status !== "available" ||
            isInstallingPackage}

          <Card.Root>
            <Card.Header>
              <div class="flex items-start gap-3">
                <div class="shrink-0">
                  <Package2 class="w-5 h-5 text-muted-foreground mt-0.5" />
                </div>
                <div class="min-w-0 flex-1">
                  <Card.Title class="text-base">{formatPackageName(name)}</Card.Title>
                  <Card.Description class="mt-1">
                    {pkg.description}
                  </Card.Description>
                </div>
              </div>
            </Card.Header>

            <Card.Footer class="flex flex-col gap-3">
              {#if name === "ethereum"}
                <div class="w-full space-y-1">
                  <Select.Root
                    type="single"
                    bind:value={selectedEthereumNetwork}
                    disabled={disabled}
                  >
                    <Select.Label class="text-xs font-medium text-muted-foreground">
                      Network
                    </Select.Label>
                    <Select.Trigger class="w-full justify-between">
                      <span class="text-sm">{selectedEthereumNetworkLabel}</span>
                    </Select.Trigger>
                    <Select.Content>
                      <Select.Group>
                        {#each ethereumNetworks as option}
                          <Select.Item
                            value={option.value}
                            label={option.label}
                          >
                            {option.label}
                          </Select.Item>
                        {/each}
                      </Select.Group>
                    </Select.Content>
                  </Select.Root>
                </div>
              {/if}
              <Button
                size="sm"
                variant="default"
                onclick={async () => {
                  const selectedNetwork =
                    name === "ethereum" ? selectedEthereumNetwork : undefined;
                  const installed = await installPackage(name, selectedNetwork);
                  if (installed) {
                    managePackage(name);
                  }
                }}
                {disabled}
                class="w-full"
              >
                {#if isInstallingPackage}
                  <div
                    class="h-4 w-4 mr-1 animate-spin rounded-full border-2 border-current border-t-transparent"
                  ></div>
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
          <p class="text-center text-muted-foreground">
            All available packages are installed!
          </p>
        </Card.Content>
      </Card.Root>
    {/if}
  </div>
</div>
