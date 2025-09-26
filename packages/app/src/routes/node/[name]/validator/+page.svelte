<script lang="ts">
import { page } from "$app/state";
import { onMount, onDestroy } from "svelte";
import { Button } from "$lib/components/ui/button";
import { Input } from "$lib/components/ui/input";
import * as Card from "$lib/components/ui/card";
import * as Alert from "$lib/components/ui/alert";
import { Switch } from "$lib/components/ui/switch";
import { packagesStore } from "$stores/packages.svelte";
import { packageConfigStore } from "$stores/packageConfig.svelte";
import { notifyError, notifySuccess } from "$utils/notify";
import { coreClient, type GenerateMnemonicResult } from "$lib/client";
import {
  KeyRound,
  ClipboardCopy,
  AlertTriangle,
  ArrowLeft,
} from "@lucide/svelte";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import ValidatorGenerationForm from "./ValidatorGenerationForm.svelte";
import MnemonicDisplay from "./MnemonicDisplay.svelte";
import CliOutputViewer from "./CliOutputViewer.svelte";

type ValidatorCliOutput = {
  run_directory: string;
  line: string;
  stream: "stdout" | "stderr";
  generation_id: string | null;
};

const packageName = $derived(page.params.name || "");
const pkg = $derived(
  packageName ? packagesStore.packages[packageName] : undefined,
);

const installedState = $derived(packagesStore.installedState);
const installedStatus = $derived(installedState.status);
const packageStatus = $derived(
  pkg ? packagesStore.installationStatus(pkg.name) : "unknown",
);
const isInstalled = $derived(packageStatus === "installed");

const networks = [
  { value: "mainnet", label: "Mainnet" },
  { value: "hoodi", label: "Hoodi" },
];

let selectedNetwork = $state("hoodi");
let currentNetwork = $state("hoodi");
let lastLoadedConfig: string | null = null;

const currentNetworkDisplay = $derived(
  networks.find((network) => network.value === currentNetwork)?.label ||
    "Hoodi",
);

let newMnemonicCount = $state("1");
let newMnemonicPassword = $state("");
let newMnemonicPasswordConfirm = $state("");
let newMnemonicWithdrawal = $state("");
let newMnemonicCompounding = $state(false);
let newMnemonicAmount = $state<string | number>("32");
let newMnemonicUsePbkdf2 = $state(false);
let generatingNewMnemonic = $state(false);
let newMnemonicResult = $state<GenerateMnemonicResult | null>(null);
let showNewMnemonicLogs = $state(false);
let showMnemonic = $state(false);
let newMnemonicLiveStdout = $state<string[]>([]);
let currentMnemonicRun: string | null = null;
let currentGenerationId: string | null = null;
let unlistenMnemonicStream: UnlistenFn | null = null;

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

function normalizeOptionalAmount(raw: string | number): string | null {
  if (typeof raw === "number") {
    return Number.isFinite(raw) ? raw.toString() : null;
  }
  const trimmed = raw.trim();
  return trimmed === "" ? null : trimmed;
}

function isValidWithdrawalAddress(value: string): boolean {
  const trimmed = value.trim();
  if (trimmed === "") return true;
  return /^0x[a-fA-F0-9]{40}$/.test(trimmed);
}

function createGenerationId(): string {
  if (typeof crypto !== "undefined" && "randomUUID" in crypto) {
    return crypto.randomUUID();
  }
  return `${Date.now().toString(36)}-${Math.random().toString(36).slice(2)}`;
}

function validateCompoundingAmount(
  amount: string | null,
  network: string,
): boolean {
  if (!amount) return false;
  const parsed = Number.parseFloat(amount);
  if (!Number.isFinite(parsed)) return false;
  const minimum = network === "mainnet" ? 32 : 1;
  return parsed >= minimum;
}

async function copyToClipboard(value: string, successMessage: string) {
  try {
    await navigator.clipboard.writeText(value);
    notifySuccess(successMessage);
  } catch (error) {
    notifyError("Failed to copy to clipboard", error);
  }
}

async function handleGenerateNewMnemonic() {
  if (generatingNewMnemonic) return;

  const numValidators = Number.parseInt(newMnemonicCount, 10);
  if (!Number.isFinite(numValidators) || numValidators < 1) {
    notifyError("Number of validators must be at least 1");
    return;
  }

  if (newMnemonicPassword.length < 12) {
    notifyError("Keystore password must be at least 12 characters long");
    return;
  }

  if (
    newMnemonicPassword.includes("\n") ||
    newMnemonicPassword.includes("\r")
  ) {
    notifyError("Keystore password cannot contain newline characters");
    return;
  }

  if (newMnemonicPassword !== newMnemonicPasswordConfirm) {
    notifyError("Keystore password confirmation does not match");
    return;
  }

  if (!isValidWithdrawalAddress(newMnemonicWithdrawal)) {
    notifyError("Withdrawal address must be a 0x-prefixed checksum address");
    return;
  }

  const generationId = createGenerationId();
  currentGenerationId = generationId;
  currentMnemonicRun = null;
  newMnemonicLiveStdout = [];
  newMnemonicResult = null;
  showMnemonic = false;
  showNewMnemonicLogs = false;

  generatingNewMnemonic = true;
  try {
    const withdrawal = newMnemonicWithdrawal.trim();
    const amountOverride = normalizeOptionalAmount(newMnemonicAmount);

    if (
      newMnemonicCompounding &&
      !validateCompoundingAmount(amountOverride, selectedNetwork)
    ) {
      notifyError(
        selectedNetwork === "mainnet"
          ? "Compounding validators require an amount of at least 32 ETH"
          : "Compounding validators require an amount of at least 1 ETH",
      );
      generatingNewMnemonic = false;
      return;
    }

    const response = await coreClient.generateValidatorMnemonic({
      num_validators: numValidators,
      chain: selectedNetwork,
      keystore_password: newMnemonicPassword,
      withdrawal_address: withdrawal ? withdrawal : null,
      compounding: newMnemonicCompounding,
      amount: newMnemonicCompounding ? amountOverride : null,
      mnemonic_language: "english",
      pbkdf2: newMnemonicUsePbkdf2,
      generation_id: generationId,
    });
    newMnemonicResult = response;
    showNewMnemonicLogs = false;
    showMnemonic = false;
    notifySuccess("Generated validator keys with EthStaker CLI");
  } catch (error) {
    notifyError("Failed to generate validator keys", error);
  } finally {
    generatingNewMnemonic = false;
    currentMnemonicRun = null;
    currentGenerationId = null;
  }
}

$effect(() => {
  if (packageStatus === "installed" && packageName) {
    void loadConfigFor(packageName);
  } else {
    lastLoadedConfig = null;
    selectedNetwork = "hoodi";
    currentNetwork = "hoodi";
  }
});

onMount(async () => {
  try {
    unlistenMnemonicStream = await listen<ValidatorCliOutput>(
      "validator:new-mnemonic-output",
      (event) => {
        const payload = event.payload;
        if (!payload) return;
        if (!currentGenerationId) return;
        if (payload.generation_id !== currentGenerationId) return;
        if (
          currentMnemonicRun &&
          payload.run_directory !== currentMnemonicRun
        ) {
          return;
        }
        if (!currentMnemonicRun) {
          currentMnemonicRun = payload.run_directory;
        }

        if (payload.stream === "stdout") {
          newMnemonicLiveStdout = [...newMnemonicLiveStdout, payload.line];
        }
        // Stderr is captured but not displayed - errors go through toast notifications
      },
    );
  } catch (error) {
    console.error("Failed to subscribe to validator CLI output", error);
  }

  await packagesStore.loadInstalledPackages({ force: true });
});

onDestroy(() => {
  if (unlistenMnemonicStream) {
    unlistenMnemonicStream();
    unlistenMnemonicStream = null;
  }
});
</script>

{#if pkg}
  <div class="mx-auto flex w-full max-w-4xl flex-col gap-6">
    <div class="flex flex-wrap items-center justify-between gap-4">
      <div>
        <h2 class="text-3xl font-bold tracking-tight">Validator management</h2>
        <p class="text-muted-foreground">
          Generate validator keys and deposits for {pkg.name}. Keys follow the {currentNetworkDisplay} network.
        </p>
      </div>
      <Button href={`/node/${pkg.name}`} variant="outline">
        <ArrowLeft class="mr-2 h-4 w-4" />
        Back to node
      </Button>
    </div>

    {#if installedStatus === "loading"}
      <Card.Root>
        <Card.Content>
          <p class="text-sm text-muted-foreground">Loading node details...</p>
        </Card.Content>
      </Card.Root>
    {:else if installedStatus === "error"}
      <Card.Root>
        <Card.Content class="flex items-center justify-between gap-4">
          <p class="text-sm text-muted-foreground">Failed to load node details.</p>
          <Button
            size="sm"
            variant="outline"
            onclick={() => packagesStore.loadInstalledPackages({ force: true })}
          >
            Retry
          </Button>
        </Card.Content>
      </Card.Root>
    {:else if packageStatus === "unknown"}
      <Card.Root>
        <Card.Content>
          <p class="text-sm text-muted-foreground">Checking node status...</p>
        </Card.Content>
      </Card.Root>
    {:else if !isInstalled}
      <Card.Root>
        <Card.Header>
          <Card.Title>Node not installed</Card.Title>
          <Card.Description>
            Install this node package before managing validators.
          </Card.Description>
        </Card.Header>
        <Card.Footer>
          <Button href="/packages" variant="default">
            Go to Package Store
          </Button>
        </Card.Footer>
      </Card.Root>
    {:else}
      <Card.Root>
        <Card.Header>
          <Card.Title class="flex items-center gap-2">
            <KeyRound class="h-5 w-5" />
            Validator tools
          </Card.Title>
          <Card.Description>
            Run the EthStaker deposit CLI directly from Kittynode. Outputs are stored under ~/.kittynode/validator-assets.
          </Card.Description>
        </Card.Header>
        <Card.Content class="space-y-8">
          <section class="space-y-4">
            <div class="space-y-1">
              <h3 class="text-sm font-semibold uppercase tracking-wide text-muted-foreground">
                New mnemonic
              </h3>
              <p class="text-sm text-muted-foreground">
                Generate a fresh mnemonic, keystores, and deposit data. Review the seed phrase carefully and store it offline before continuing.
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
                  Outputs follow the node network ({currentNetworkDisplay}).
                </p>
              </div>
              {#if generatingNewMnemonic}
                <div class="md:col-span-2 space-y-2 rounded-lg border border-border bg-muted/10 p-3">
                  <div class="flex items-center justify-between">
                    <p class="text-xs font-medium uppercase tracking-wide text-muted-foreground">
                      EthStaker CLI output
                    </p>
                  </div>
                  {#if newMnemonicLiveStdout.length > 0}
                    <pre class="max-h-52 overflow-auto rounded-md bg-background p-3 text-xs">{newMnemonicLiveStdout.join("\n")}</pre>
                  {/if}
                  {#if newMnemonicLiveStdout.length === 0}
                    <p class="text-xs text-muted-foreground">
                      Waiting for CLI output…
                    </p>
                  {/if}
                </div>
              {/if}
            </form>
            {#if newMnemonicResult}
              {@const result = newMnemonicResult}
              <div class="space-y-4 rounded-lg border border-border bg-muted/10 p-4">
                <Alert.Root variant="destructive">
                  <AlertTriangle class="size-4" />
                  <Alert.Title>Store this mnemonic safely</Alert.Title>
                  <Alert.Description>
                    Write the phrase down offline. Anyone with access can control your validator.
                  </Alert.Description>
                </Alert.Root>
                <div class="flex items-center justify-between gap-2">
                  <div class="flex items-center gap-2 text-sm font-medium">
                    <KeyRound class="h-4 w-4" />
                    Mnemonic
                  </div>
                  <Button
                    size="sm"
                    variant="outline"
                    onclick={() => {
                      if (!showMnemonic) {
                        showMnemonic = true;
                      }
                      copyToClipboard(result.mnemonic, "Mnemonic copied to clipboard");
                    }}
                    class="gap-1"
                  >
                    <ClipboardCopy class="h-3 w-3" />
                    Copy
                  </Button>
                </div>
                <div class="space-y-2">
                  <Button
                    size="sm"
                    variant="secondary"
                    onclick={() => (showMnemonic = !showMnemonic)}
                    class="gap-2"
                  >
                    {showMnemonic ? "Hide mnemonic" : "Reveal mnemonic"}
                  </Button>
                  {#if showMnemonic}
                    <p class="break-words rounded-md bg-background p-3 font-mono text-sm leading-relaxed">
                      {result.mnemonic}
                    </p>
                  {:else}
                    <p class="rounded-md bg-muted p-3 text-sm text-muted-foreground">
                      Mnemonic hidden. Click “Reveal mnemonic” to view.
                    </p>
                  {/if}
                </div>
                <div class="space-y-3">
                  <div class="space-y-1">
                    <p class="text-xs font-medium uppercase tracking-wide text-muted-foreground">
                      Validator directory
                    </p>
                    <div class="flex items-center justify-between gap-2 rounded-md border border-border bg-background px-3 py-2">
                      <span class="break-all font-mono text-xs">{result.validator_keys_dir}</span>
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
                          <span class="break-all font-mono text-xs">{file}</span>
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
                  {/if}
                </div>
              </div>
            {/if}
          </section>
        </Card.Content>
      </Card.Root>
    {/if}
  </div>
{:else}
  <div class="flex min-h-[400px] items-center justify-center">
    <Card.Root class="max-w-md">
      <Card.Header>
        <Card.Title class="flex items-center gap-2">
          <AlertTriangle class="h-5 w-5" />
          Package not found
        </Card.Title>
      </Card.Header>
      <Card.Content>
        <p class="text-muted-foreground">
          The package "{packageName}" could not be found.
        </p>
      </Card.Content>
      <Card.Footer>
        <Button href="/packages" variant="default">
          Browse available packages
        </Button>
      </Card.Footer>
    </Card.Root>
  </div>
{/if}
