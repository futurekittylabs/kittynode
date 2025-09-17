<script lang="ts">
import { page } from "$app/state";
import { Button } from "$lib/components/ui/button";
import * as Card from "$lib/components/ui/card";
import { packagesStore } from "$stores/packages.svelte";
import { onDestroy, onMount } from "svelte";
import DockerLogs from "../../package/DockerLogs.svelte";
import { dockerStatus } from "$stores/dockerStatus.svelte";
import { packageConfigStore } from "$stores/packageConfig.svelte";
import { usePackageDeleter } from "$lib/composables/usePackageDeleter.svelte";
import * as Select from "$lib/components/ui/select";
import * as Alert from "$lib/components/ui/alert";
import {
  Terminal,
  CircleCheck,
  Trash2,
  Play,
  Square,
  Activity,
  Settings,
  FileText,
  CircleAlert,
  Wifi,
  WifiOff,
} from "@lucide/svelte";
import { notifyError, notifySuccess } from "$utils/notify";

const { isDeleting, deletePackage } = usePackageDeleter();

const packageName = $derived(page.params.name || "");
const pkg = $derived(
  packageName ? packagesStore.packages[packageName] : undefined,
);

const installedState = $derived(packagesStore.installedState);
const packageStatus = $derived(
  pkg ? packagesStore.installationStatus(pkg.name) : "unknown",
);

let activeLogType = $state<null | "execution" | "consensus">("execution");
let configLoading = $state(false);
let selectedNetwork = $state("holesky");
let currentNetwork = $state("holesky");

const networks = [
  { value: "mainnet", label: "Mainnet" },
  { value: "holesky", label: "Holesky" },
];

const logSources = {
  execution: {
    description: "Execution client logs",
    containerName: "reth-node",
  },
  consensus: {
    description: "Consensus client logs",
    containerName: "lighthouse-node",
  },
} as const;

const activeLogSource = $derived(
  activeLogType ? logSources[activeLogType] : null,
);

const networkTriggerContent = $derived(
  networks.find((n) => n.value === selectedNetwork)?.label || "Holesky",
);

const currentNetworkDisplay = $derived(
  networks.find((n) => n.value === currentNetwork)?.label || "Holesky",
);

const installedStatus = $derived(installedState.status);
const isInstalled = $derived(packageStatus === "installed");
const isDeletingPackage = $derived(pkg ? isDeleting(pkg.name) : false);

async function handleDeletePackage(name: string) {
  await deletePackage(name, { redirectToDashboard: true });
}

function toggleLogs(logType: "execution" | "consensus") {
  activeLogType = activeLogType === logType ? null : logType;
}

async function loadConfig() {
  if (!packageName) return;
  try {
    const config = await packageConfigStore.getConfig(packageName);
    const network = config.values.network || "holesky";
    currentNetwork = selectedNetwork = network;
  } catch (e) {
    notifyError("Failed to get package config", e);
  }
}

async function updateConfig() {
  if (!packageName) return;

  configLoading = true;
  try {
    await packageConfigStore.updateConfig(packageName, {
      values: {
        network: selectedNetwork,
      },
    });
    currentNetwork = selectedNetwork;
    notifySuccess("Configuration updated successfully");
  } catch (e) {
    notifyError("Failed to update package config", e);
  } finally {
    configLoading = false;
  }
}

$effect(() => {
  if (isInstalled && packageName) {
    loadConfig();
  }
});

onMount(async () => {
  dockerStatus.startPolling();
  await packagesStore.loadInstalledPackages({ force: true });
  if (isInstalled) {
    await loadConfig();
  }
});

onDestroy(() => {
  dockerStatus.stopPolling();
});
</script>

{#if pkg}
  <div class="mx-auto flex w-full max-w-6xl flex-col gap-6">
    <!-- Header -->
    <div class="flex flex-wrap items-start justify-between gap-4">
      <div>
        <h2 class="text-3xl font-bold tracking-tight">{pkg.name}</h2>
        <p class="text-muted-foreground">{pkg.description}</p>
      </div>
      {#if isInstalled}
        <div class="flex items-center space-x-2">
          <div class="flex items-center space-x-1 rounded-full bg-green-500/10 px-3 py-1.5">
            <Activity class="h-4 w-4 text-green-500 animate-pulse" />
            <span class="text-sm font-medium text-green-700 dark:text-green-400">Running</span>
          </div>
        </div>
      {/if}
    </div>

    {#if !dockerStatus.isRunning}
      <Alert.Root>
        <Terminal class="size-4" />
        <Alert.Title>Docker is not running</Alert.Title>
        <Alert.Description>
          Start Docker Desktop to manage this node.
        </Alert.Description>
      </Alert.Root>
    {:else if installedStatus === "error"}
      <Card.Root>
        <Card.Content class="flex items-center justify-between">
          <p class="text-sm text-muted-foreground">
            Failed to load node status.
          </p>
          <Button size="sm" variant="outline" onclick={() => packagesStore.loadInstalledPackages({ force: true })}>
            Retry
          </Button>
        </Card.Content>
      </Card.Root>
    {:else if installedStatus === "unavailable"}
      <Alert.Root>
        <Terminal class="size-4" />
        <Alert.Title>Docker is not available</Alert.Title>
        <Alert.Description>
          Start Docker Desktop to manage this node.
        </Alert.Description>
      </Alert.Root>
    {:else if packageStatus === "unknown"}
      <Card.Root>
        <Card.Content>
          <p class="text-sm text-muted-foreground">Checking node status...</p>
        </Card.Content>
      </Card.Root>
    {:else if !isInstalled}
      <Card.Root>
        <Card.Header>
          <Card.Title>Node Not Installed</Card.Title>
          <Card.Description>
            This node package is not currently installed. Visit the Package Store to install it.
          </Card.Description>
        </Card.Header>
        <Card.Footer>
          <Button href="/packages" variant="default">
            Go to Package Store
          </Button>
        </Card.Footer>
      </Card.Root>
    {:else}
      <!-- Quick Actions -->
      <div class="grid gap-4 sm:grid-cols-2 xl:grid-cols-3">
        <Card.Root>
          <Card.Header class="pb-3">
            <Card.Title class="text-sm font-medium">Node Status</Card.Title>
          </Card.Header>
          <Card.Content>
            <div class="flex items-center space-x-2">
              <Wifi class="h-4 w-4 text-green-500" />
              <span class="text-sm font-medium">Connected</span>
            </div>
          </Card.Content>
        </Card.Root>

        <Card.Root>
          <Card.Header class="pb-3">
            <Card.Title class="text-sm font-medium">Network</Card.Title>
          </Card.Header>
          <Card.Content>
            <div class="flex items-center space-x-2">
              <Activity class="h-4 w-4 text-muted-foreground" />
              <span class="text-sm font-medium">{currentNetworkDisplay}</span>
            </div>
          </Card.Content>
        </Card.Root>

        <Card.Root>
          <Card.Header class="pb-3">
            <Card.Title class="text-sm font-medium">Actions</Card.Title>
          </Card.Header>
          <Card.Content>
            <Button
              size="sm"
              variant="destructive"
              onclick={() => handleDeletePackage(pkg.name)}
              disabled={isDeletingPackage}
              class="w-full"
            >
              {#if isDeletingPackage}
                <div class="h-4 w-4 mr-1 animate-spin rounded-full border-2 border-current border-t-transparent"></div>
                Deleting...
              {:else}
                <Trash2 class="h-4 w-4 mr-1" />
                Delete Node
              {/if}
            </Button>
          </Card.Content>
        </Card.Root>
      </div>

      <!-- Configuration -->
      <Card.Root>
        <Card.Header>
          <Card.Title class="flex items-center gap-2">
            <Settings class="h-5 w-5" />
            Configuration
          </Card.Title>
          <Card.Description>
            Adjust settings for your {pkg.name} node
          </Card.Description>
        </Card.Header>
        <Card.Content>
          <form class="space-y-4" onsubmit={(e) => { e.preventDefault(); updateConfig(); }}>
            <div class="space-y-2">
              <label for="network" class="text-sm font-medium">Network</label>
              <Select.Root type="single" name="network" bind:value={selectedNetwork}>
                <Select.Trigger class="w-full sm:w-[220px] md:w-[240px]">
                  {networkTriggerContent}
                </Select.Trigger>
                <Select.Content>
                  <Select.Group>
                    {#each networks as network}
                      <Select.Item value={network.value} label={network.label}>
                        {network.label}
                      </Select.Item>
                    {/each}
                  </Select.Group>
                </Select.Content>
              </Select.Root>
            </div>
            <Button
              type="submit"
              disabled={configLoading || selectedNetwork === currentNetwork}
              size="sm"
            >
              {configLoading ? "Updating..." : "Update Configuration"}
            </Button>
          </form>
        </Card.Content>
      </Card.Root>

      <!-- Logs -->
      <Card.Root class="min-w-0">
        <Card.Header>
          <Card.Title class="flex items-center gap-2">
            <FileText class="h-5 w-5" />
            Logs
          </Card.Title>
          <Card.Description>
            View real-time logs from your node
          </Card.Description>
        </Card.Header>
        <Card.Content class="space-y-4">
          <div class="flex flex-wrap gap-2">
            <Button
              size="sm"
              variant={activeLogType === 'execution' ? 'default' : 'outline'}
              onclick={() => toggleLogs('execution')}
            >
              {activeLogType === 'execution' ? 'Hide' : 'Show'} Execution Logs
            </Button>
            <Button
              size="sm"
              variant={activeLogType === 'consensus' ? 'default' : 'outline'}
              onclick={() => toggleLogs('consensus')}
            >
              {activeLogType === 'consensus' ? 'Hide' : 'Show'} Consensus Logs
            </Button>
          </div>

          {#if activeLogSource}
            <div class="space-y-2">
              <div class="text-sm text-muted-foreground">
                {activeLogSource.description}:
              </div>
              <DockerLogs containerName={activeLogSource.containerName} tailLines={1000} />
            </div>
          {:else}
            <div class="rounded-lg border border-dashed bg-muted/30 py-10 text-center text-muted-foreground">
              Select a log type to view real-time logs
            </div>
          {/if}
        </Card.Content>
      </Card.Root>
    {/if}
  </div>
{:else}
  <div class="flex items-center justify-center min-h-[400px]">
    <Card.Root class="max-w-md">
      <Card.Header>
        <Card.Title class="flex items-center gap-2">
          <CircleAlert class="h-5 w-5" />
          Package Not Found
        </Card.Title>
      </Card.Header>
      <Card.Content>
        <p class="text-muted-foreground">
          The package "{packageName}" could not be found.
        </p>
      </Card.Content>
      <Card.Footer>
        <Button href="/packages" variant="default">
          Browse Available Packages
        </Button>
      </Card.Footer>
    </Card.Root>
  </div>
{/if}
