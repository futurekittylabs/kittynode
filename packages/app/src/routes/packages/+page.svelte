<script lang="ts">
import { onMount } from "svelte";
import * as Card from "$lib/components/ui/card";
import { Button } from "$lib/components/ui/button";
import { packagesStore } from "$stores/packages.svelte";
import { operationalStateStore } from "$stores/operationalState.svelte";
import { goto } from "$app/navigation";
import { usePackageInstaller } from "$lib/composables/usePackageInstaller.svelte";
import {
  Package2,
  Download,
  CircleCheck,
  CircleAlert,
  Settings2,
  Search,
} from "@lucide/svelte";

const { isInstalling, installPackage } = usePackageInstaller();

let searchQuery = $state("");

const catalogState = $derived(packagesStore.catalogState);
const installedState = $derived(packagesStore.installedState);

const filteredPackages = $derived(() => {
  if (catalogState.status !== "ready") {
    return [];
  }

  const query = searchQuery.toLowerCase();
  return Object.entries(packagesStore.packages)
    .filter(
      ([name, pkg]) =>
        name.toLowerCase().includes(query) ||
        pkg.description.toLowerCase().includes(query),
    )
    .sort(([a], [b]) => a.localeCompare(b));
});

function managePackage(packageName: string) {
  goto(`/node/${packageName}`);
}

onMount(() => {
  operationalStateStore.startPolling();
  void operationalStateStore.refresh();
  packagesStore.loadPackages();
  packagesStore.loadInstalledPackages();

  return () => {
    operationalStateStore.stopPolling();
  };
});
</script>

<div class="space-y-6">
  <div>
    <h2 class="text-3xl font-bold tracking-tight">Package Store</h2>
    <p class="text-muted-foreground">
      Browse and install node infrastructure packages
    </p>
  </div>

  <div class="relative">
    <Search
      class="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground"
    />
    <input
      type="text"
      placeholder="Search packages..."
      bind:value={searchQuery}
      class="w-full rounded-md border bg-background pl-10 pr-4 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-primary"
    />
  </div>

  {#if catalogState.status === "error"}
    <Card.Root>
      <Card.Content class="flex items-center justify-between">
        <p class="text-sm text-muted-foreground">Failed to load packages.</p>
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
    <Card.Root class="border-yellow-500/50 bg-yellow-500/10">
      <Card.Header>
        <Card.Title class="flex items-center space-x-2">
          <CircleAlert class="h-5 w-5 text-yellow-500" />
          <span>Docker Required</span>
        </Card.Title>
      </Card.Header>
      <Card.Content>
        <p class="text-sm">
          Docker needs to be running to install packages. Please start Docker
          Desktop.
        </p>
      </Card.Content>
    </Card.Root>
  {:else if catalogState.status !== "ready" || installedState.status === "loading" || installedState.status === "idle"}
    <Card.Root>
      <Card.Content>
        <p class="text-sm text-muted-foreground">Loading packages...</p>
      </Card.Content>
    </Card.Root>
  {:else if filteredPackages().length > 0}
    <div class="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
      {#each filteredPackages() as [name, pkg]}
        {@const status = packagesStore.installationStatus(name)}
        {@const isInstallingPackage = isInstalling(name)}

        <Card.Root>
          <Card.Header>
            <div class="flex items-start justify-between">
              <div class="flex items-start space-x-3">
                <Package2 class="h-5 w-5 text-muted-foreground mt-0.5" />
                <div class="flex-1">
                  <Card.Title class="text-base">{name}</Card.Title>
                  <Card.Description class="mt-1">
                    {pkg.description}
                  </Card.Description>
                </div>
              </div>
              {#if status === "installed"}
                <div
                  class="flex items-center space-x-1 rounded-full bg-green-500/10 px-2 py-1"
                >
                  <CircleCheck class="h-3 w-3 text-green-500" />
                  <span
                    class="text-xs font-medium text-green-700 dark:text-green-400"
                    >Installed</span
                  >
                </div>
              {/if}
            </div>
          </Card.Header>

          <Card.Footer>
            {#if status === "installed"}
              <Button
                size="sm"
                variant="default"
                onclick={() => managePackage(name)}
                class="w-full"
              >
                <Settings2 class="h-4 w-4 mr-1" />
                Manage
              </Button>
            {:else if status === "available"}
              <Button
                size="sm"
                variant="default"
                onclick={async () => {
                  const installed = await installPackage(name);
                  if (installed) {
                    managePackage(name);
                  }
                }}
                disabled={!operationalStateStore.canInstall ||
                  isInstallingPackage}
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
            {:else}
              <Button size="sm" variant="outline" disabled class="w-full">
                Checking status...
              </Button>
            {/if}
          </Card.Footer>
        </Card.Root>
      {/each}
    </div>
  {:else}
    <Card.Root>
      <Card.Content>
        <p class="text-center text-muted-foreground">
          {searchQuery
            ? "No packages found matching your search."
            : "No packages available."}
        </p>
      </Card.Content>
    </Card.Root>
  {/if}
</div>
