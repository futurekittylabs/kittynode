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
import { Terminal } from "@lucide/svelte";
import { notifyError, notifySuccess } from "$utils/notify";

let installLoading: string | null = $state(null);
let deleteLoading: string | null = $state(null);
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

function canInstallPackage(packageName: string): boolean {
  return (
    (dockerStatus.isRunning ?? false) &&
    !installLoading &&
    !deleteLoading &&
    !packagesStore.isInstalled(packageName)
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
  if (dockerStatus.isRunning) {
    packagesStore.loadInstalledPackages();
  }
});

onMount(async () => {
  dockerStatus.startPolling();
  await loadConfig();
});

onDestroy(() => {
  selectedPackageStore.clear();
  dockerStatus.stopPolling();
});
</script>

<Link href="/" text="← Back home" />

{#if selectedPackageStore.package}
    {@const pkg = selectedPackageStore.package}

    <!-- Overview -->
    <h3 class="scroll-m-20 text-2xl font-semibold tracking-tight my-4">
        Overview
    </h3>
    <p><strong>Name:</strong> {pkg.name}</p>

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
    {:else}
        {#if !packagesStore.isInstalled(pkg.name)}
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
    {#if packagesStore.isInstalled(pkg.name)}
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
    {#if packagesStore.isInstalled(pkg.name)}
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
