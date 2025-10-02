<script lang="ts">
import { page } from "$app/state";
import * as Alert from "$lib/components/ui/alert";
import { Button } from "$lib/components/ui/button";
import * as Card from "$lib/components/ui/card";
import { Input } from "$lib/components/ui/input";
import { packageConfigStore } from "$stores/packageConfig.svelte";
import { operationalStateStore } from "$stores/operationalState.svelte";
import { notifyError, notifySuccess } from "$utils/notify";
import { KeyRound, Lock, RefreshCw } from "@lucide/svelte";

const packageName = $derived(page.params.name || "");
const packageNameLower = $derived(packageName.toLowerCase());
const isEthereumPackage = $derived(packageNameLower === "ethereum");

let isLoading = $state(true);
let isSaving = $state(false);
let configValues = $state<Record<string, string>>({});
let feeRecipient = $state("");
let graffiti = $state("");
let builderEndpoint = $state("");
let lastLoadedPackage: string | null = null;

const canManageValidator = $derived(operationalStateStore.canManage);
const formDisabled = $derived(isSaving || !canManageValidator);

function resetFields() {
  feeRecipient = configValues.feeRecipient ?? "";
  graffiti = configValues.graffiti ?? "";
  builderEndpoint = configValues.builderEndpoint ?? "";
}

async function loadValidatorConfig(name: string) {
  if (!name || lastLoadedPackage === name) {
    return;
  }

  isLoading = true;

  try {
    const config = await packageConfigStore.getConfig(name);
    configValues = { ...config.values };
    feeRecipient = configValues.feeRecipient ?? "";
    graffiti = configValues.graffiti ?? "";
    builderEndpoint = configValues.builderEndpoint ?? "";
    lastLoadedPackage = name;
  } catch (error) {
    notifyError("Failed to load validator config", error);
    configValues = {};
    lastLoadedPackage = null;
  } finally {
    isLoading = false;
  }
}

function applyField(
  values: Record<string, string>,
  key: string,
  value: string,
) {
  const trimmed = value.trim();
  if (trimmed) {
    values[key] = trimmed;
  } else {
    delete values[key];
  }
}

async function refreshConfig() {
  if (!packageName) {
    return;
  }
  lastLoadedPackage = null;
  await loadValidatorConfig(packageName);
}

async function saveConfig(event: SubmitEvent) {
  event.preventDefault();

  if (!packageName || formDisabled) {
    return;
  }

  isSaving = true;

  try {
    const updatedValues: Record<string, string> = { ...configValues };

    applyField(updatedValues, "feeRecipient", feeRecipient);
    applyField(updatedValues, "graffiti", graffiti);
    applyField(updatedValues, "builderEndpoint", builderEndpoint);

    await packageConfigStore.updateConfig(packageName, {
      values: updatedValues,
    });

    configValues = updatedValues;
    notifySuccess("Saved validator configuration");
  } catch (error) {
    notifyError("Failed to save validator configuration", error);
  } finally {
    isSaving = false;
  }
}

$effect(() => {
  if (!isEthereumPackage) {
    isLoading = false;
    lastLoadedPackage = null;
    return;
  }

  if (!packageName) {
    return;
  }

  void loadValidatorConfig(packageName);
});
</script>

{#if !isEthereumPackage}
  <div class="mx-auto flex w-full max-w-4xl flex-col gap-6">
    <Card.Root>
      <Card.Header>
        <Card.Title>Validator config unavailable</Card.Title>
        <Card.Description>
          This page is only available for the Ethereum package.
        </Card.Description>
      </Card.Header>
      <Card.Footer>
        <Button href={`/node/${packageName}`} variant="outline">
          Return to node overview
        </Button>
      </Card.Footer>
    </Card.Root>
  </div>
{:else}
  <div class="mx-auto flex w-full max-w-4xl flex-col gap-6">
    <div class="flex flex-wrap items-start justify-between gap-4">
      <div>
        <h2 class="text-3xl font-bold tracking-tight">Validator config</h2>
        <p class="text-muted-foreground">
          Tune validator-specific settings for your Ethereum node.
        </p>
      </div>
      <Button
        variant="outline"
        size="sm"
        onclick={refreshConfig}
        disabled={isLoading}
      >
        <RefreshCw class="mr-2 h-4 w-4" />
        Refresh
      </Button>
    </div>

    {#if !canManageValidator}
      <Alert.Root>
        <Lock class="size-4" />
        <Alert.Title>Read-only mode</Alert.Title>
        <Alert.Description>
          Connect to a Kittynode core with management access to edit validator
          settings.
        </Alert.Description>
      </Alert.Root>
    {/if}

    <Card.Root>
      <Card.Header class="gap-2">
        <Card.Title class="flex items-center gap-2">
          <KeyRound class="h-5 w-5" />
          Ethereum validator preferences
        </Card.Title>
        <Card.Description>
          Configure payout, graffiti, and builder settings used by Lighthouse.
        </Card.Description>
      </Card.Header>

      {#if isLoading}
        <Card.Content class="space-y-4">
          <div class="h-5 w-32 animate-pulse rounded bg-muted"></div>
          <div class="h-10 w-full animate-pulse rounded bg-muted"></div>
          <div class="h-5 w-40 animate-pulse rounded bg-muted"></div>
          <div class="h-10 w-full animate-pulse rounded bg-muted"></div>
          <div class="h-10 w-full animate-pulse rounded bg-muted"></div>
        </Card.Content>
      {:else}
        <form class="space-y-6" onsubmit={saveConfig}>
          <Card.Content class="space-y-6">
            <div class="space-y-2">
              <label class="text-sm font-medium" for="feeRecipient">
                Fee recipient address
              </label>
              <Input
                id="feeRecipient"
                name="feeRecipient"
                bind:value={feeRecipient}
                placeholder="0x..."
                spellcheck={false}
                disabled={formDisabled}
              />
              <p class="text-xs text-muted-foreground">
                Execution payloads will pay tips to this address. Leave blank to
                use the client default.
              </p>
            </div>

            <div class="space-y-2">
              <label class="text-sm font-medium" for="graffiti">
                Graffiti
              </label>
              <Input
                id="graffiti"
                name="graffiti"
                maxlength={32}
                bind:value={graffiti}
                placeholder="hello kittynode"
                disabled={formDisabled}
              />
              <p class="text-xs text-muted-foreground">
                Optional message displayed on blocks produced by your validator
                (32 characters max).
              </p>
            </div>

            <div class="space-y-2">
              <label class="text-sm font-medium" for="builderEndpoint">
                Builder endpoint
              </label>
              <Input
                id="builderEndpoint"
                name="builderEndpoint"
                bind:value={builderEndpoint}
                placeholder="https://relay.example"
                disabled={formDisabled}
              />
              <p class="text-xs text-muted-foreground">
                URL for a builder or relay service. Leave blank to broadcast
                directly to the beacon network.
              </p>
            </div>
          </Card.Content>

          <Card.Footer class="flex flex-wrap gap-2">
            <Button type="submit" disabled={formDisabled}>
              {isSaving ? "Saving..." : "Save changes"}
            </Button>
            <Button
              type="button"
              variant="outline"
              size="sm"
              onclick={resetFields}
              disabled={isSaving}
            >
              Reset
            </Button>
          </Card.Footer>
        </form>
      {/if}
    </Card.Root>
  </div>
{/if}
