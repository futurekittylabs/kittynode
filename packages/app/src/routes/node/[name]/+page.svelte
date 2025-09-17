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
  Trash2,
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
const isRunning = $derived(packageStatus === "running");
const isStopped = $derived(packageStatus === "stopped");

let activeLogType = $state<null | "execution" | "consensus">(null);
let configLoading = $state(false);
let selectedNetwork = $state("holesky");
let currentNetwork = $state("holesky");

const networks = [
  { value: "mainnet", label: "Mainnet" },
  { value: "holesky", label: "Holesky" },
];

const networkTriggerContent = $derived(
  networks.find((n) => n.value === selectedNetwork)?.label || "Holesky",
);

const currentNetworkDisplay = $derived(
  networks.find((n) => n.value === currentNetwork)?.label || "Holesky",
);

const installedStatus = $derived(installedState.status);
const isInstalled = $derived(isRunning || isStopped);
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
  <div class="space-y-6">
    <!-- Header -->
    <div class="flex items-center justify-between">
      <div>
        <h2 class="text-3xl font-bold tracking-tight">{pkg.name}</h2>
        <p class="text-muted-foreground">{pkg.description}</p>
      </div>
      {#if isRunning}
        <div class="flex items-center space-x-2">
          <div class="flex items-center space-x-1 rounded-full bg-green-500/10 px-3 py-1.5">
            <Activity class="h-4 w-4 text-green-500 animate-pulse" />
            <span class="text-sm font-medium text-green-700 dark:text-green-400">Running</span>
          </div>
        </div>
      {:else if isStopped}
        <div class="flex items-center space-x-2">
          <div class="flex items-center space-x-1 rounded-full bg-yellow-500/10 px-3 py-1.5">
            <Activity class="h-4 w-4 text-yellow-500" />
            <span class="text-sm font-medium text-yellow-700 dark:text-yellow-400">Stopped</span>
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
      {#if isStopped}
        <Alert.Root>
          <WifiOff class="size-4" />
          <Alert.Title>Node containers are stopped</Alert.Title>
          <Alert.Description>
            Start the containers in Docker Desktop or delete and reinstall the node from the Package Store.
          </Alert.Description>
        </Alert.Root>
      {/if}

      <!-- Quick Actions -->
      <div class="grid gap-4 md:grid-cols-3">
        <Card.Root>
          <Card.Header class="pb-3">
            <Card.Title class="text-sm font-medium">Node Status</Card.Title>
          </Card.Header>
          <Card.Content>
            <div class="flex items-center space-x-2">
              {#if isRunning}
                <Wifi class="h-4 w-4 text-green-500" />
                <span class="text-sm font-medium">Connected</span>
              {:else}
                <WifiOff class="h-4 w-4 text-yellow-500" />
                <span class="text-sm font-medium">Containers stopped</span>
              {/if}
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
                <Select.Trigger class="w-full md:w-[200px]">
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
      <Card.Root>
        <Card.Header>
          <Card.Title class="flex items-center gap-2">
            <FileText class="h-5 w-5" />
            Logs
          </Card.Title>
          <Card.Description>
            View real-time logs from your node
          </Card.Description>
        </Card.Header>
        <Card.Content>
          <div class="flex gap-2 mb-4">
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

          {#if activeLogType === 'execution'}
            <div class="space-y-2">
              <div class="text-sm text-muted-foreground">Execution client logs:</div>
              <div class="rounded-lg border bg-muted/50 p-4">
                <DockerLogs containerName="reth-node" tailLines={1000} />
              </div>
            </div>
          {/if}

          {#if activeLogType === 'consensus'}
            <div class="space-y-2">
              <div class="text-sm text-muted-foreground">Consensus client logs:</div>
              <div class="rounded-lg border bg-muted/50 p-4">
                <DockerLogs containerName="lighthouse-node" tailLines={1000} />
              </div>
            </div>
          {/if}

          {#if !activeLogType}
            <div class="text-center py-8 text-muted-foreground">
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
