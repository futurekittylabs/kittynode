<script lang="ts">
import { page } from "$app/state";
import { Button } from "$lib/components/ui/button";
import { Input } from "$lib/components/ui/input";
import * as Card from "$lib/components/ui/card";
import { packagesStore } from "$stores/packages.svelte";
import { onDestroy, onMount } from "svelte";
import DockerLogs from "$lib/components/DockerLogs.svelte";
import { operationalStateStore } from "$stores/operationalState.svelte";
import { packageConfigStore } from "$stores/packageConfig.svelte";
import { usePackageDeleter } from "$lib/composables/usePackageDeleter.svelte";
import * as Select from "$lib/components/ui/select";
import * as Alert from "$lib/components/ui/alert";
import { createPackageRuntimeController } from "$lib/runtime/packageRuntime.svelte";
import { Switch } from "$lib/components/ui/switch";
import { coreClient, type GenerateMnemonicResult } from "$lib/client";
import {
  Terminal,
  Trash2,
  Play,
  Square,
  Activity,
  Settings,
  FileText,
  CircleAlert,
  Wifi,
  WifiOff,
  PauseCircle,
  KeyRound,
  ClipboardCopy,
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

const runtime = createPackageRuntimeController();
let lastLoadedConfig: string | null = null;

let activeLogType = $state<null | "execution" | "consensus">("execution");
let configLoading = $state(false);
let selectedNetwork = $state("hoodi");
let currentNetwork = $state("hoodi");

let newMnemonicCount = $state("1");
let newMnemonicPassword = $state("");
let newMnemonicPasswordConfirm = $state("");
let newMnemonicWithdrawal = $state("");
let newMnemonicCompounding = $state(false);
let newMnemonicAmount = $state<string | number>("");
let newMnemonicUsePbkdf2 = $state(false);
let generatingNewMnemonic = $state(false);
let newMnemonicResult = $state<GenerateMnemonicResult | null>(null);
let showNewMnemonicLogs = $state(false);

const networks = [
  { value: "mainnet", label: "Mainnet" },
  { value: "hoodi", label: "Hoodi" },
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
  networks.find((n) => n.value === selectedNetwork)?.label || "Hoodi",
);

const currentNetworkDisplay = $derived(
  networks.find((n) => n.value === currentNetwork)?.label || "Hoodi",
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

const canResumeNode = $derived(
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
    const network = config.values.network || "hoodi";
    currentNetwork = network;
    selectedNetwork = network;
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

async function resumeNode() {
  if (!packageName || !canResumeNode) {
    if (!operationalStateStore.canManage) {
      notifyError("Cannot manage packages in the current operational state");
    }
    return;
  }

  try {
    const success = await runtime.performLifecycle("starting", () =>
      packagesStore.resumePackage(packageName),
    );
    if (success) {
      notifySuccess(`Resumed ${packageName}`);
    }
  } catch (error) {
    notifyError(`Failed to resume ${packageName}`, error);
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

async function copyToClipboard(value: string, successMessage: string) {
  try {
    await navigator.clipboard.writeText(value);
    notifySuccess(successMessage);
  } catch (error) {
    notifyError("Failed to copy to clipboard", error);
  }
}

function normalizeOptionalAmount(raw: string | number): string | null {
  if (typeof raw === "number") {
    return Number.isFinite(raw) ? raw.toString() : null;
  }
  const trimmed = raw.trim();
  return trimmed === "" ? null : trimmed;
}

async function handleGenerateNewMnemonic() {
  if (generatingNewMnemonic) return;

  const numValidators = Number.parseInt(newMnemonicCount, 10);
  if (!Number.isFinite(numValidators) || numValidators < 1) {
    notifyError("Number of validators must be at least 1");
    return;
  }

  if (!newMnemonicPassword) {
    notifyError("Enter a keystore password before generating keys");
    return;
  }

  if (newMnemonicPassword !== newMnemonicPasswordConfirm) {
    notifyError("Keystore password confirmation does not match");
    return;
  }

  generatingNewMnemonic = true;
  try {
    const withdrawal = newMnemonicWithdrawal.trim();
    const amountOverride = normalizeOptionalAmount(newMnemonicAmount);

    const response = await coreClient.generateValidatorMnemonic({
      num_validators: numValidators,
      chain: selectedNetwork,
      keystore_password: newMnemonicPassword,
      withdrawal_address: withdrawal ? withdrawal : null,
      compounding: newMnemonicCompounding,
      amount: newMnemonicCompounding ? amountOverride : null,
      mnemonic_language: "english",
      pbkdf2: newMnemonicUsePbkdf2,
    });
    newMnemonicResult = response;
    showNewMnemonicLogs = false;
    notifySuccess("Generated validator keys with EthStaker CLI");
  } catch (error) {
    notifyError("Failed to generate validator keys", error);
  } finally {
    generatingNewMnemonic = false;
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
    selectedNetwork = "hoodi";
    currentNetwork = "hoodi";
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
        <h2 class="text-3xl font-bold tracking-tight">{pkg.name}</h2>
        <p class="text-muted-foreground">Manage your {pkg.name} node.</p>
      </div>
      {#if isInstalled}
        <div class="flex items-center space-x-2">
          {#if statusKind === "stopping"}
            <div class="flex items-center space-x-2 rounded-full bg-amber-500/10 px-3 py-1.5">
              <div class="h-3 w-3 animate-spin rounded-full border-2 border-amber-500 border-t-transparent"></div>
              <span class="text-sm font-medium text-amber-700 dark:text-amber-200">Stopping…</span>
            </div>
          {:else if statusKind === "starting"}
            <div class="flex items-center space-x-2 rounded-full bg-emerald-500/10 px-3 py-1.5">
              <div class="h-3 w-3 animate-spin rounded-full border-2 border-emerald-500 border-t-transparent"></div>
              <span class="text-sm font-medium text-emerald-700 dark:text-emerald-200">Starting…</span>
            </div>
          {:else if statusKind === "checking"}
            <div class="flex items-center space-x-2 rounded-full bg-muted px-3 py-1.5">
              <div class="h-3 w-3 animate-spin rounded-full border-2 border-current border-t-transparent"></div>
              <span class="text-sm font-medium text-muted-foreground">Checking status</span>
            </div>
          {:else if statusKind === "running"}
            <div class="flex items-center space-x-1 rounded-full bg-green-500/10 px-3 py-1.5">
              <Activity class="h-4 w-4 text-green-500 animate-pulse" />
              <span class="text-sm font-medium text-green-700 dark:text-green-400">Running</span>
            </div>
          {:else if statusKind === "stopped"}
            <div class="flex items-center space-x-1 rounded-full bg-muted px-3 py-1.5">
              <PauseCircle class="h-4 w-4 text-muted-foreground" />
              <span class="text-sm font-medium text-muted-foreground">Stopped</span>
            </div>
          {:else}
            <div class="flex items-center space-x-1 rounded-full bg-muted px-3 py-1.5">
              <CircleAlert class="h-4 w-4 text-muted-foreground" />
              <span class="text-sm font-medium text-muted-foreground">Status unknown</span>
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
          {#if statusKind === "stopping"}
            <div class="flex items-center space-x-2 text-amber-700 dark:text-amber-200">
              <div class="h-4 w-4 animate-spin rounded-full border-2 border-amber-500 border-t-transparent"></div>
              <span class="text-sm font-medium">Stopping…</span>
            </div>
          {:else if statusKind === "starting"}
            <div class="flex items-center space-x-2 text-emerald-700 dark:text-emerald-200">
              <div class="h-4 w-4 animate-spin rounded-full border-2 border-emerald-500 border-t-transparent"></div>
              <span class="text-sm font-medium">Starting…</span>
            </div>
          {:else if statusKind === "checking"}
            <div class="flex items-center space-x-2 text-muted-foreground">
              <div class="h-4 w-4 animate-spin rounded-full border-2 border-current border-t-transparent"></div>
              <span class="text-sm font-medium">Checking status...</span>
            </div>
          {:else if statusKind === "running"}
            <div class="flex items-center space-x-2">
              <Wifi class="h-4 w-4 text-green-500" />
              <span class="text-sm font-medium">Running</span>
            </div>
          {:else if statusKind === "stopped"}
            <div class="flex items-center space-x-2">
              <WifiOff class="h-4 w-4 text-muted-foreground" />
              <span class="text-sm font-medium">Stopped</span>
            </div>
          {:else}
            <div class="flex items-center space-x-2 text-muted-foreground">
              <CircleAlert class="h-4 w-4" />
              <span class="text-sm font-medium">Status unavailable</span>
            </div>
          {/if}
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
        <Card.Content class="space-y-2">
          {#if statusKind === "checking"}
            <Button size="sm" variant="outline" disabled class="w-full">
              <div class="h-4 w-4 mr-1 animate-spin rounded-full border-2 border-current border-t-transparent"></div>
              Checking status...
            </Button>
          {:else if statusKind === "stopping"}
            <Button size="sm" variant="outline" disabled class="w-full border-amber-200 text-amber-700 dark:border-amber-400/40 dark:text-amber-200">
              <div class="h-4 w-4 mr-1 animate-spin rounded-full border-2 border-amber-500 border-t-transparent"></div>
              Stopping…
            </Button>
          {:else if statusKind === "starting"}
            <Button size="sm" variant="outline" disabled class="w-full border-emerald-200 text-emerald-700 dark:border-emerald-400/40 dark:text-emerald-200">
              <div class="h-4 w-4 mr-1 animate-spin rounded-full border-2 border-emerald-500 border-t-transparent"></div>
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
              onclick={resumeNode}
              disabled={!canResumeNode}
            >
              <Play class="h-4 w-4 mr-1" />
              Resume Node
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

  <Card.Root>
    <Card.Header>
      <Card.Title class="flex items-center gap-2">
        <KeyRound class="h-5 w-5" />
        Validator Tools
      </Card.Title>
      <Card.Description>
        Run the EthStaker deposit CLI directly from Kittynode. Outputs are stored under
        ~/.kittynode/validator-assets.
      </Card.Description>
    </Card.Header>
    <Card.Content class="space-y-8">
      <section class="space-y-4">
        <div class="space-y-1">
          <h3 class="text-sm font-semibold uppercase tracking-wide text-muted-foreground">
            New mnemonic
          </h3>
          <p class="text-sm text-muted-foreground">
            Generate a fresh mnemonic, keystores, and deposit data. Review the seed phrase carefully and
            store it offline before continuing.
          </p>
        </div>
        <form
          class="grid gap-4 md:grid-cols-2"
          onsubmit={(event) => {
            event.preventDefault();
            handleGenerateNewMnemonic();
          }}
        >
          <div class="space-y-2">
            <label class="text-sm font-medium" for="mnemonic-validator-count">
              Number of validators
            </label>
            <Input
              id="mnemonic-validator-count"
              type="number"
              min="1"
              step="1"
              bind:value={newMnemonicCount}
              disabled={generatingNewMnemonic}
              inputmode="numeric"
            />
          </div>
          <div class="space-y-2">
            <label class="text-sm font-medium" for="mnemonic-password">
              Keystore password
            </label>
            <Input
              id="mnemonic-password"
              type="password"
              bind:value={newMnemonicPassword}
              disabled={generatingNewMnemonic}
              autocomplete="new-password"
            />
          </div>
          <div class="space-y-2">
            <label class="text-sm font-medium" for="mnemonic-password-confirm">
              Confirm password
            </label>
            <Input
              id="mnemonic-password-confirm"
              type="password"
              bind:value={newMnemonicPasswordConfirm}
              disabled={generatingNewMnemonic}
              autocomplete="new-password"
            />
          </div>
          <div class="space-y-2">
            <label class="text-sm font-medium" for="mnemonic-withdrawal">
              Withdrawal address (optional)
            </label>
            <Input
              id="mnemonic-withdrawal"
              type="text"
              bind:value={newMnemonicWithdrawal}
              placeholder="0x..."
              disabled={generatingNewMnemonic}
              autocomplete="off"
            />
          </div>
          <div class="md:col-span-2 grid gap-3">
            <div class="flex items-center justify-between rounded-md border border-border px-3 py-2">
              <div>
                <p class="text-sm font-medium">Compounding validators</p>
                <p class="text-xs text-muted-foreground">
                  Enables 0x02 withdrawal credentials with balances above 32 ETH.
                </p>
              </div>
              <Switch
                checked={newMnemonicCompounding}
                onCheckedChange={(checked) => (newMnemonicCompounding = checked)}
                disabled={generatingNewMnemonic}
                aria-label="Toggle compounding withdrawals"
              />
            </div>
            {#if newMnemonicCompounding}
              <div class="space-y-2">
                <label class="text-sm font-medium" for="mnemonic-amount">
                  Validator amount (ETH)
                </label>
                <Input
                  id="mnemonic-amount"
                  type="number"
                  step="0.01"
                  min="1"
                  bind:value={newMnemonicAmount}
                  placeholder="32"
                  disabled={generatingNewMnemonic}
                  inputmode="decimal"
                />
              </div>
            {/if}
            <div class="flex items-center justify-between rounded-md border border-border px-3 py-2">
              <div>
                <p class="text-sm font-medium">PBKDF2 keystores</p>
                <p class="text-xs text-muted-foreground">
                  Enable when you require PBKDF2-compatible keystores for downstream tooling.
                </p>
              </div>
              <Switch
                checked={newMnemonicUsePbkdf2}
                onCheckedChange={(checked) => (newMnemonicUsePbkdf2 = checked)}
                disabled={generatingNewMnemonic}
                aria-label="Toggle PBKDF2 for keystores"
              />
            </div>
          </div>
          <div class="md:col-span-2 flex items-center gap-3">
            <Button type="submit" disabled={generatingNewMnemonic}>
              {#if generatingNewMnemonic}
                <div class="mr-2 h-4 w-4 animate-spin rounded-full border-2 border-current border-t-transparent"></div>
                Generating…
              {:else}
                Generate keys
              {/if}
            </Button>
            <p class="text-xs text-muted-foreground">
              Outputs follow the node network ({networkTriggerContent}).
            </p>
          </div>
        </form>
        {#if newMnemonicResult}
          {@const result = newMnemonicResult}
          <div class="space-y-4 rounded-lg border border-border bg-muted/10 p-4">
            <div class="flex items-center justify-between gap-2">
              <div class="flex items-center gap-2 text-sm font-medium">
                <KeyRound class="h-4 w-4" />
                Mnemonic
              </div>
              <Button
                size="sm"
                variant="outline"
                onclick={() => copyToClipboard(result.mnemonic, "Mnemonic copied to clipboard")}
                class="gap-1"
              >
                <ClipboardCopy class="h-3 w-3" />
                Copy
              </Button>
            </div>
            <p class="rounded-md bg-background p-3 font-mono text-sm leading-relaxed break-words">
              {result.mnemonic}
            </p>
            <div class="space-y-3">
              <div class="space-y-1">
                <p class="text-xs font-medium uppercase tracking-wide text-muted-foreground">
                  Validator directory
                </p>
                <div class="flex items-center justify-between gap-2 rounded-md border border-border bg-background px-3 py-2">
                  <span class="font-mono text-xs break-all">{result.validator_keys_dir}</span>
                  <Button
                    size="sm"
                    variant="ghost"
                    onclick={() => copyToClipboard(result.validator_keys_dir, "Path copied")}
                    class="gap-1"
                  >
                    <ClipboardCopy class="h-3 w-3" />
                    Copy
                  </Button>
                </div>
              </div>
              <div class="space-y-1">
                <p class="text-xs font-medium uppercase tracking-wide text-muted-foreground">Deposit JSON</p>
                <ul class="space-y-1">
                  {#each result.deposit_files as file}
                    <li class="flex items-center justify-between gap-2 rounded-md border border-border bg-background px-3 py-2">
                      <span class="font-mono text-xs break-all">{file}</span>
                      <Button
                        size="sm"
                        variant="ghost"
                        onclick={() => copyToClipboard(file, "Path copied")}
                        class="gap-1"
                      >
                        <ClipboardCopy class="h-3 w-3" />
                        Copy
                      </Button>
                    </li>
                  {:else}
                    <li class="text-xs text-muted-foreground">No deposit files detected.</li>
                  {/each}
                </ul>
              </div>
              <div class="space-y-1">
                <p class="text-xs font-medium uppercase tracking-wide text-muted-foreground">Keystores</p>
                <p class="text-xs text-muted-foreground">
                  {result.keystore_files.length} keystore file(s) written to the validator directory.
                </p>
              </div>
            </div>
            <div class="space-y-2">
              <Button
                size="sm"
                variant="outline"
                onclick={() => (showNewMnemonicLogs = !showNewMnemonicLogs)}
                class="gap-2"
              >
                {showNewMnemonicLogs ? "Hide CLI output" : "View CLI output"}
              </Button>
              {#if showNewMnemonicLogs}
                <pre class="max-h-52 overflow-auto rounded-md bg-background p-3 text-xs">
{result.stdout.join("\n")}
                </pre>
                {#if result.stderr.length > 0}
                  <pre class="max-h-40 overflow-auto rounded-md bg-destructive/10 p-3 text-xs text-destructive">
{result.stderr.join("\n")}
                  </pre>
                {/if}
              {/if}
            </div>
          </div>
        {/if}
      </section>

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
