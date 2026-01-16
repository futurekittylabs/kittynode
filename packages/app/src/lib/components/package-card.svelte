<script lang="ts">
  import * as Card from "$lib/components/ui/card";
  import { Button } from "$lib/components/ui/button";
  import * as Select from "$lib/components/ui/select";
  import { packagesState } from "$lib/states/packages.svelte";
  import { operationalState } from "$lib/states/operational.svelte";
  import { notifyError, notifySuccess } from "$lib/utils/notify";
  import { formatPackageName } from "$lib/utils";
  import {
    defaultEthereumNetwork,
    ethereumNetworks,
  } from "$lib/constants/ethereum-networks";
  import { Package2, Download, CircleCheck, Settings2 } from "@lucide/svelte";

  let {
    name,
    description,
    onManage,
    onInstalled,
    showStatusBadge = true,
  } = $props<{
    name: string;
    description: string;
    onManage: (packageName: string) => void;
    onInstalled?: (packageName: string) => void;
    showStatusBadge?: boolean;
  }>();

  const status = $derived(packagesState.installationStatus(name));
  let isInstalling = $state(false);
  let selectedEthereumNetwork = $state(defaultEthereumNetwork);

  const defaultEthereumNetworkLabel =
    ethereumNetworks.find((option) => option.value === defaultEthereumNetwork)
      ?.label ?? defaultEthereumNetwork;

  const selectedEthereumNetworkLabel = $derived(
    ethereumNetworks.find((option) => option.value === selectedEthereumNetwork)
      ?.label || defaultEthereumNetworkLabel
  );

  const installDisabled = $derived(
    !operationalState.canInstall || status !== "available" || isInstalling
  );

  async function handleInstall() {
    if (!operationalState.canInstall) {
      notifyError("Cannot install packages in the current operational state");
      return;
    }

    const currentStatus = packagesState.installationStatus(name);

    if (currentStatus === "unknown") {
      notifyError(
        "Package status is still loading. Try again once it finishes."
      );
      return;
    }

    if (currentStatus === "installed") {
      notifyError(`${name} is already installed`);
      return;
    }

    if (isInstalling) {
      return;
    }

    isInstalling = true;
    try {
      const network = name === "ethereum" ? selectedEthereumNetwork : undefined;
      await packagesState.installPackage(name, network);
      notifySuccess(`Successfully installed ${name}`);
      onInstalled?.(name);
    } catch (error) {
      notifyError(`Failed to install ${name}`, error);
    } finally {
      isInstalling = false;
    }
  }
</script>

<Card.Root>
  <Card.Header>
    <div class="flex items-start justify-between">
      <div class="flex items-start gap-3">
        <Package2 class="h-5 w-5 text-muted-foreground mt-0.5" />
        <div class="min-w-0 flex-1">
          <Card.Title class="text-base">{formatPackageName(name)}</Card.Title>
          <Card.Description class="mt-1">{description}</Card.Description>
        </div>
      </div>
      {#if showStatusBadge && status === "installed"}
        <div
          class="flex items-center space-x-1 rounded-full bg-green-500/10 px-2 py-1"
        >
          <CircleCheck class="h-3 w-3 text-green-500" />
          <span class="text-xs font-medium text-green-700 dark:text-green-400">
            Installed
          </span>
        </div>
      {/if}
    </div>
  </Card.Header>

  <Card.Footer class="flex flex-col gap-3">
    {#if status === "installed"}
      <Button
        size="sm"
        variant="default"
        onclick={() => onManage(name)}
        class="w-full"
      >
        <Settings2 class="h-4 w-4 mr-1" />
        Manage
      </Button>
    {:else if status === "available"}
      {#if name === "ethereum"}
        <div class="w-full space-y-1">
          <Select.Root
            type="single"
            bind:value={selectedEthereumNetwork}
            disabled={installDisabled}
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
                  <Select.Item value={option.value} label={option.label}>
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
        onclick={handleInstall}
        disabled={installDisabled}
        class="w-full"
      >
        {#if isInstalling}
          <div
            class="h-4 w-4 mr-1 animate-spin rounded-full border-2 border-current border-t-transparent"
          ></div>
          Installing...
        {:else}
          <Download class="h-4 w-4 mr-1" />
          Install
        {/if}
      </Button>
    {:else}
      <Button size="sm" variant="outline" disabled class="w-full">
        Checking status...
      </Button>
    {/if}
  </Card.Footer>
</Card.Root>
