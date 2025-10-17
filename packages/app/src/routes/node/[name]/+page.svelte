<script lang="ts">
import { page } from "$app/state";
import { Button } from "$lib/components/ui/button";
import * as Card from "$lib/components/ui/card";
import { packagesStore } from "$stores/packages.svelte";
import { onDestroy, onMount } from "svelte";
import DockerLogs from "$lib/components/DockerLogs.svelte";
import { operationalStateStore } from "$stores/operationalState.svelte";
import { packageConfigStore } from "$stores/packageConfig.svelte";
import { usePackageDeleter } from "$lib/composables/usePackageDeleter.svelte";
import * as Select from "$lib/components/ui/select";
import * as Alert from "$lib/components/ui/alert";
import {
  defaultEthereumNetwork,
  ethereumNetworks,
  ethereumNetworkValues,
  formatEthereumNetworks,
} from "$lib/constants/ethereumNetworks";
import { createPackageRuntimeController } from "$lib/runtime/packageRuntime.svelte";
import {
  Terminal,
  Trash2,
  Play,
  Square,
  Activity,
  Globe,
  Settings,
  FileText,
  CircleAlert,
  PauseCircle,
  ShieldCheck,
  ArrowUpRight,
} from "@lucide/svelte";
import { notifyError, notifySuccess } from "$utils/notify";
import { formatPackageName } from "$lib/utils";

const { isDeleting, deletePackage } = usePackageDeleter();

const packageName = $derived(page.params.name || "");
const pkg = $derived(
  packageName ? packagesStore.packages[packageName] : undefined,
);
const installedState = $derived(packagesStore.installedState);
const packageStatus = $derived(
  pkg ? packagesStore.installationStatus(pkg.name) : "unknown",
);

const runtime = createPackageRuntimeController();
let lastLoadedConfig: string | null = null;

let activeLogType = $state<null | "execution" | "consensus">("execution");
let configLoading = $state(false);
let selectedNetwork = $state<string>(defaultEthereumNetwork);
let currentNetwork = $state<string>(defaultEthereumNetwork);

const networks = ethereumNetworks;
const supportedNetworkValues: string[] = [...ethereumNetworkValues];
const defaultNetworkLabel =
  networks.find((option) => option.value === defaultEthereumNetwork)?.label ??
  defaultEthereumNetwork;
const supportedNetworksMessage = formatEthereumNetworks(", ");
const RESOURCE_PREFIX = "kittynode-";

const logSources = {
  execution: {
    description: "Execution client logs",
    containerName: `${RESOURCE_PREFIX}reth-node`,
  },
  consensus: {
    description: "Consensus client logs",
    containerName: `${RESOURCE_PREFIX}lighthouse-node`,
  },
} as const;

const activeLogSource = $derived(
  activeLogType ? logSources[activeLogType] : null,
);

const networkTriggerContent = $derived(
  networks.find((n) => n.value === selectedNetwork)?.label ||
    defaultNetworkLabel,
);

const currentNetworkDisplay = $derived(
  networks.find((n) => n.value === currentNetwork)?.label ??
    (!currentNetwork ? "Not configured" : `${currentNetwork} (unsupported)`),
);

const installedStatus = $derived(installedState.status);
const isInstalled = $derived(packageStatus === "installed");
const isDeletingPackage = $derived(pkg ? isDeleting(pkg.name) : false);

const statusKind = $derived(
  runtime.lifecycle === "stopping"
    ? "stopping"
    : runtime.lifecycle === "starting"
      ? "starting"
      : runtime.loading && runtime.status === "checking"
        ? "checking"
        : runtime.status,
);

const canStopNode = $derived(
  runtime.lifecycle === "idle" &&
    runtime.status === "running" &&
    operationalStateStore.canManage,
);

const canStartNode = $derived(
  runtime.lifecycle === "idle" &&
    runtime.status === "stopped" &&
    operationalStateStore.canManage,
);

async function handleDeletePackage(name: string) {
  await deletePackage(name, { redirectToDashboard: true });
}

function toggleLogs(logType: "execution" | "consensus") {
  activeLogType = activeLogType === logType ? null : logType;
}

async function loadConfigFor(name: string) {
  if (!name || lastLoadedConfig === name) {
    return;
  }

  try {
    const config = await packageConfigStore.getConfig(name);
    const network = config.values.network || defaultEthereumNetwork;
    currentNetwork = network;
    if (!supportedNetworkValues.includes(network)) {
      notifyError(
        `Network "${network}" is not supported. Please choose ${supportedNetworksMessage}.`,
      );
      selectedNetwork = defaultEthereumNetwork;
    } else {
      selectedNetwork = network;
    }
    lastLoadedConfig = name;
  } catch (error) {
    notifyError("Failed to get package config", error);
  }
}

async function stopNode() {
  if (!packageName || !canStopNode) {
    if (!operationalStateStore.canManage) {
      notifyError("Cannot manage packages in the current operational state");
    }
    return;
  }

  try {
    const success = await runtime.performLifecycle("stopping", () =>
      packagesStore.stopPackage(packageName),
    );
    if (success) {
      notifySuccess(`Stopped ${packageName}`);
    }
  } catch (error) {
    notifyError(`Failed to stop ${packageName}`, error);
  }
}

async function startNode() {
  if (!packageName || !canStartNode) {
    if (!operationalStateStore.canManage) {
      notifyError("Cannot manage packages in the current operational state");
    }
    return;
  }

  try {
    const success = await runtime.performLifecycle("starting", () =>
      packagesStore.startPackage(packageName),
    );
    if (success) {
      notifySuccess(`Started ${packageName}`);
    }
  } catch (error) {
    notifyError(`Failed to start ${packageName}`, error);
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
    lastLoadedConfig = packageName;
    notifySuccess("Configuration updated successfully");
  } catch (error) {
    notifyError("Failed to update package config", error);
  } finally {
    configLoading = false;
  }
}

$effect(() => {
  const name = isInstalled && packageName ? packageName : null;
  runtime.attach({
    name,
    enabled: Boolean(name),
    pollInterval: operationalStateStore.isStarting ? 2000 : 5000,
  });

  if (name) {
    void loadConfigFor(name);
  } else {
    lastLoadedConfig = null;
    selectedNetwork = defaultEthereumNetwork;
    currentNetwork = defaultEthereumNetwork;
  }
});

onMount(async () => {
  operationalStateStore.startPolling();
  await operationalStateStore.refresh();
  await packagesStore.loadInstalledPackages({ force: true });
});

onDestroy(() => {
  operationalStateStore.stopPolling();
  runtime.stop();
});
</script>

{#if pkg}
  <div class="mx-auto flex w-full max-w-6xl flex-col gap-6">
    <!-- Header -->
    <div class="flex flex-wrap items-start justify-between gap-4">
      <div>
        <h2 class="text-3xl font-bold tracking-tight">
          {formatPackageName(pkg.name)}
        </h2>
        <p class="text-muted-foreground">
          Manage your {formatPackageName(pkg.name)} node
        </p>
      </div>
      {#if isInstalled}
        <div class="flex items-center space-x-2">
          {#if statusKind === "stopping"}
            <div
              class="flex items-center space-x-2 rounded-full bg-amber-500/10 px-3 py-1.5"
            >
              <div
                class="h-3 w-3 animate-spin rounded-full border-2 border-amber-500 border-t-transparent"
              ></div>
              <span
                class="text-sm font-medium text-amber-700 dark:text-amber-200"
                >Stopping…</span
              >
            </div>
          {:else if statusKind === "starting"}
            <div
              class="flex items-center space-x-2 rounded-full bg-emerald-500/10 px-3 py-1.5"
            >
              <div
                class="h-3 w-3 animate-spin rounded-full border-2 border-emerald-500 border-t-transparent"
              ></div>
              <span
                class="text-sm font-medium text-emerald-700 dark:text-emerald-200"
                >Starting…</span
              >
            </div>
          {:else if statusKind === "checking"}
            <div
              class="flex items-center space-x-2 rounded-full bg-muted px-3 py-1.5"
            >
              <div
                class="h-3 w-3 animate-spin rounded-full border-2 border-current border-t-transparent"
              ></div>
              <span class="text-sm font-medium text-muted-foreground"
                >Checking status</span
              >
            </div>
          {:else if statusKind === "running"}
            <div
              class="flex items-center space-x-1 rounded-full bg-green-500/10 px-3 py-1.5"
            >
              <Activity class="h-4 w-4 text-green-500 animate-pulse" />
              <span
                class="text-sm font-medium text-green-700 dark:text-green-400"
                >Running</span
              >
            </div>
          {:else if statusKind === "stopped"}
            <div
              class="flex items-center space-x-1 rounded-full bg-amber-500/10 px-3 py-1.5"
            >
              <PauseCircle class="h-4 w-4 text-amber-500 dark:text-amber-200" />
              <span
                class="text-sm font-medium text-amber-700 dark:text-amber-200"
                >Stopped</span
              >
            </div>
          {:else}
            <div
              class="flex items-center space-x-1 rounded-full bg-muted px-3 py-1.5"
            >
              <CircleAlert class="h-4 w-4 text-muted-foreground" />
              <span class="text-sm font-medium text-muted-foreground"
                >Status unknown</span
              >
            </div>
          {/if}
        </div>
      {/if}
    </div>

    {#if operationalStateStore.state?.mode === "local" && operationalStateStore.dockerRunning === false}
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
          <Button
            size="sm"
            variant="outline"
            onclick={() => packagesStore.loadInstalledPackages({ force: true })}
          >
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
            This node package is not currently installed. Visit the Package
            Store to install it.
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
      <div class="grid gap-4 sm:grid-cols-2">
        <Card.Root>
          <Card.Header class="pb-3">
            <Card.Title class="text-sm font-medium">Network</Card.Title>
          </Card.Header>
          <Card.Content>
            <div class="flex items-center space-x-2">
              <Globe class="h-4 w-4 text-muted-foreground" />
              <span class="text-sm font-medium">{currentNetworkDisplay}</span>
            </div>
          </Card.Content>
        </Card.Root>

        <Card.Root>
          <Card.Header class="pb-3">
            <Card.Title class="text-sm font-medium">Actions</Card.Title>
          </Card.Header>
          <Card.Content class="space-y-2">
            {#if statusKind === "checking"}
              <Button size="sm" variant="outline" disabled class="w-full">
                <div
                  class="h-4 w-4 mr-1 animate-spin rounded-full border-2 border-current border-t-transparent"
                ></div>
                Checking status...
              </Button>
            {:else if statusKind === "stopping"}
              <Button
                size="sm"
                variant="outline"
                disabled
                class="w-full border-amber-200 text-amber-700 dark:border-amber-400/40 dark:text-amber-200"
              >
                <div
                  class="h-4 w-4 mr-1 animate-spin rounded-full border-2 border-amber-500 border-t-transparent"
                ></div>
                Stopping…
              </Button>
            {:else if statusKind === "starting"}
              <Button
                size="sm"
                variant="outline"
                disabled
                class="w-full border-emerald-200 text-emerald-700 dark:border-emerald-400/40 dark:text-emerald-200"
              >
                <div
                  class="h-4 w-4 mr-1 animate-spin rounded-full border-2 border-emerald-500 border-t-transparent"
                ></div>
                Starting…
              </Button>
            {:else if runtime.status === "running"}
              <Button
                size="sm"
                variant="outline"
                class="w-full border-amber-200 text-amber-700 hover:bg-amber-50 dark:border-amber-400/40 dark:text-amber-200 dark:hover:bg-amber-400/10"
                onclick={stopNode}
                disabled={!canStopNode}
              >
                <Square class="h-4 w-4 mr-1" />
                Stop Node
              </Button>
            {:else if runtime.status === "stopped"}
              <Button
                size="sm"
                variant="outline"
                class="w-full border-emerald-200 text-emerald-700 hover:bg-emerald-50 dark:border-emerald-400/40 dark:text-emerald-200 dark:hover:bg-emerald-400/10"
                onclick={startNode}
                disabled={!canStartNode}
              >
                <Play class="h-4 w-4 mr-1" />
                Start Node
              </Button>
            {:else}
              <Button size="sm" variant="outline" disabled class="w-full">
                Status unavailable
              </Button>
            {/if}
            <Button
              size="sm"
              variant="destructive"
              onclick={() => handleDeletePackage(pkg.name)}
              disabled={isDeletingPackage}
              class="w-full"
            >
              {#if isDeletingPackage}
                <div
                  class="h-4 w-4 mr-1 animate-spin rounded-full border-2 border-current border-t-transparent"
                ></div>
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
      <div class="grid gap-4 {pkg.name === 'ethereum' ? 'sm:grid-cols-2' : ''}">
        <Card.Root>
          <Card.Header>
            <Card.Title class="flex items-center gap-2">
              <Settings class="h-5 w-5" />
              Configuration
            </Card.Title>
            <Card.Description>
              Adjust settings for your {formatPackageName(pkg.name)} node
            </Card.Description>
          </Card.Header>
          <Card.Content>
            <form
              class="space-y-4"
              onsubmit={(e) => {
                e.preventDefault();
                updateConfig();
              }}
            >
              <div class="space-y-2">
                <label for="network" class="text-sm font-medium">Network</label>
                <Select.Root
                  type="single"
                  name="network"
                  bind:value={selectedNetwork}
                >
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

        {#if pkg.name === "ethereum"}
          <Card.Root>
            <Card.Header>
              <Card.Title class="flex items-center gap-2">
                <ShieldCheck class="h-5 w-5" />
                Validator Config
              </Card.Title>
              <Card.Description>
                Manage your validators with Kittynode CLI
              </Card.Description>
            </Card.Header>
            <Card.Content class="flex h-full flex-col">
              <p class="text-sm text-muted-foreground">
                Currently, validator management is only supported through the Kittynode CLI. Install the Kittynode CLI, and you'll be able to monitor your validators from here!
              </p>
              <div class="mt-4 sm:mt-auto sm:pt-4">
                <Button
                  size="sm"
                  href="https://kittynode.com/download"
                  target="_blank"
                  rel="noreferrer"
                  class="w-full sm:w-auto"
                >
                  <span>Install Kittynode CLI</span>
                  <ArrowUpRight class="h-4 w-4" />
                </Button>
              </div>
            </Card.Content>
          </Card.Root>
        {/if}
      </div>

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
              variant={activeLogType === "execution" ? "default" : "outline"}
              onclick={() => toggleLogs("execution")}
            >
              {activeLogType === "execution" ? "Hide" : "Show"} Execution Logs
            </Button>
            <Button
              size="sm"
              variant={activeLogType === "consensus" ? "default" : "outline"}
              onclick={() => toggleLogs("consensus")}
            >
              {activeLogType === "consensus" ? "Hide" : "Show"} Consensus Logs
            </Button>
          </div>

          {#if activeLogSource}
            <div class="space-y-2">
              <div class="text-sm text-muted-foreground">
                {activeLogSource.description}:
              </div>
              <DockerLogs
                containerName={activeLogSource.containerName}
                tailLines={1000}
              />
            </div>
          {:else}
            <div
              class="rounded-lg border border-dashed bg-muted/30 py-10 text-center text-muted-foreground"
            >
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
