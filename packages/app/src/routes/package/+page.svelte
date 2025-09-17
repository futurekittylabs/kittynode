<script lang="ts">
import Button from "$lib/components/ui/button/button.svelte";
import Link from "$lib/components/ui/link/link.svelte";
import { selectedPackageStore } from "$stores/selectedPackage.svelte";
import { packagesStore } from "$stores/packages.svelte";
import { onDestroy, onMount } from "svelte";
import DockerLogs from "./DockerLogs.svelte";
import { dockerStatus } from "$stores/dockerStatus.svelte";
import { packageConfigStore } from "$stores/packageConfig.svelte";
import * as Select from "$lib/components/ui/select";
import * as Alert from "$lib/components/ui/alert";
import { Terminal, CircleCheck } from "@lucide/svelte";
import { notifyError, notifySuccess } from "$utils/notify";

let installLoading: string | null = $state(null);
let deleteLoading: string | null = $state(null);
let activeLogType = $state<null | "execution" | "consensus">(null);
let configLoading = $state(false);
let selectedNetwork = $state("holesky");
let currentNetwork = $state("holesky");

const packageStatus = $derived(
  selectedPackageStore.package
    ? packagesStore.installationStatus(selectedPackageStore.package.name)
    : "unknown",
);
const isInstalled = $derived(packageStatus === "installed");
const installedState = $derived(packagesStore.installedState);
const installedStatus = $derived(installedState.status);

const networks = [
  { value: "mainnet", label: "Mainnet" },
  { value: "holesky", label: "Holesky" },
];

const networkTriggerContent = $derived(
  networks.find((n) => n.value === selectedNetwork)?.label || "Holesky",
);

function canInstallPackage(packageName: string): boolean {
  return (
    (dockerStatus.isRunning ?? false) &&
    !installLoading &&
    !deleteLoading &&
    packagesStore.installationStatus(packageName) === "available"
  );
}

async function installPackage(name: string) {
  if (!dockerStatus.isRunning) {
    notifyError("Docker must be running to install packages");
    return;
  }

  installLoading = name;
  try {
    await packagesStore.installPackage(name);
    activeLogType = "execution";
    notifySuccess(`Successfully installed ${name}`);
    await loadConfig();
  } catch (e) {
    notifyError(`Failed to install ${name}`, e);
  } finally {
    installLoading = null;
  }
}

async function deletePackage(name: string) {
  if (!dockerStatus.isRunning) {
    notifyError("Docker must be running to delete packages");
    return;
  }

  deleteLoading = name;
  try {
    await packagesStore.deletePackage(name);
    notifySuccess(`Successfully deleted ${name}`);
    activeLogType = null;
  } catch (e) {
    notifyError(`Failed to delete ${name}`, e);
  } finally {
    deleteLoading = null;
  }
}

function toggleLogs(logType: "execution" | "consensus") {
  activeLogType = activeLogType === logType ? null : logType;
}

async function loadConfig() {
  if (!selectedPackageStore.package) return;
  try {
    const config = await packageConfigStore.getConfig(
      selectedPackageStore.package.name,
    );
    const network = config.values.network || "holesky";
    currentNetwork = selectedNetwork = network;
  } catch (e) {
    notifyError("Failed to get package config", e);
  }
}

async function updateConfig() {
  if (!selectedPackageStore.package) return;

  configLoading = true;
  try {
    await packageConfigStore.updateConfig(selectedPackageStore.package.name, {
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
  if (isInstalled) {
    loadConfig();
  }
});

onMount(async () => {
  dockerStatus.startPolling();
  await packagesStore.loadInstalledPackages({ force: true });
});

onDestroy(() => {
  selectedPackageStore.clear();
  dockerStatus.stopPolling();
});
</script>

<Link href="/" text="← Back home" />

{#if selectedPackageStore.package}
    {@const pkg = selectedPackageStore.package}

    <!-- Package Header -->
    <div class="flex items-center justify-between mb-6">
        <div>
            <h2 class="text-3xl font-bold tracking-tight">{pkg.name}</h2>
            <p class="text-muted-foreground">{pkg.description}</p>
        </div>
        <div class="flex items-center space-x-2">
            {#if isInstalled}
                <div class="flex items-center space-x-1 rounded-full bg-green-500/10 px-3 py-1.5">
                    <CircleCheck class="h-4 w-4 text-green-500" />
                    <span class="text-sm font-medium text-green-700 dark:text-green-400">Installed</span>
                </div>
            {/if}
        </div>
    </div>

    <!-- Lifecycle -->
    <h3 class="scroll-m-20 text-2xl font-semibold tracking-tight my-4">
        Lifecycle
    </h3>
    {#if !dockerStatus.isRunning}
        <Alert.Root>
          <Terminal class="size-4" />
          <Alert.Title>Start Docker to use this package</Alert.Title>
          <Alert.Description>If you need to install Docker, follow the installation guide <Link href="https://docs.docker.com/engine/install" targetBlank text="here" />.</Alert.Description>
        </Alert.Root>
        <br />
    {:else if installedStatus === "error"}
        <div class="flex items-center gap-2">
            <Button
                variant="outline"
                size="sm"
                onclick={() => packagesStore.loadInstalledPackages({ force: true })}
            >
                Retry status check
            </Button>
            <span class="text-sm text-muted-foreground">Failed to load package status.</span>
        </div>
    {:else if installedStatus === "unavailable"}
        <Alert.Root>
          <Terminal class="size-4" />
          <Alert.Title>Docker is not available</Alert.Title>
          <Alert.Description>Start Docker Desktop to manage this package.</Alert.Description>
        </Alert.Root>
        <br />
    {:else if packageStatus === "unknown"}
        <Button disabled variant="outline">Checking package status...</Button>
    {:else}
        {#if !isInstalled}
            <Button
                onclick={() => installPackage(pkg.name)}
                disabled={!canInstallPackage(pkg.name)}
            >
                {installLoading === pkg.name ? "Installing..." : "Install"}
            </Button>
        {:else}
            <Button
                variant="destructive"
                onclick={() => deletePackage(pkg.name)}
                disabled={deleteLoading === pkg.name}
            >
                {deleteLoading === pkg.name ? "Deleting..." : "Delete"}
            </Button>
        {/if}
    {/if}

    <br />

    <!-- Configuration -->
    {#if isInstalled}
        <h3 class="scroll-m-20 text-2xl font-semibold tracking-tight my-4">
            Configuration
        </h3>
        <form class="space-y-4" onsubmit={(e) => { e.preventDefault(); updateConfig(); }}>
            <div class="space-y-2">
                <label for="network" class="font-medium text-sm">Network</label>
                <Select.Root type="single" name="network" bind:value={selectedNetwork}>
                    <Select.Trigger class="w-[180px]">
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
            >
                {configLoading ? "Updating..." : "Update Configuration"}
            </Button>
        </form>
    {/if}

    <br />

    <!-- Logging -->
    {#if isInstalled}
        <h3 class="scroll-m-20 text-2xl font-semibold tracking-tight my-4">
            Logging
        </h3>
        <div class="flex gap-2">
            <Button
                variant="default"
                onclick={() => toggleLogs('execution')}
            >
                {activeLogType === 'execution' ? 'Hide execution logs' : 'View execution logs'}
            </Button>

            <Button
                variant="default"
                onclick={() => toggleLogs('consensus')}
            >
                {activeLogType === 'consensus' ? 'Hide consensus logs' : 'View consensus logs'}
            </Button>
        </div>

        <div class="logs-container">
            {#if activeLogType === 'execution'}
                <p class="mt-4">Execution logs:</p>
                <div class="mt-4">
                    <DockerLogs containerName="reth-node" tailLines={1000} />
                </div>
            {/if}

            {#if activeLogType === 'consensus'}
                <p class="mt-4">Consensus logs:</p>
                <div class="mt-4">
                    <DockerLogs containerName="lighthouse-node" tailLines={1000} />
                </div>
            {/if}
        </div>
    {/if}
{/if}
