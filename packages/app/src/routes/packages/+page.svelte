<script lang="ts">
  import { CircleAlert, Search } from "@lucide/svelte";
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import PackageCard from "$lib/components/package-card.svelte";
  import { Button } from "$lib/components/ui/button";
  import * as Card from "$lib/components/ui/card";
  import { operationalState } from "$lib/states/operational.svelte";
  import { packagesState } from "$lib/states/packages.svelte";

  let searchQuery = $state("");

  const catalogState = $derived(packagesState.catalogState);
  const installedState = $derived(packagesState.installedState);

  const filteredPackages = $derived(() => {
    if (catalogState.status !== "ready") {
      return [];
    }

    const query = searchQuery.toLowerCase();
    return Object.entries(packagesState.packages)
      .filter(
        ([name, pkg]) =>
          name.toLowerCase().includes(query) ||
          pkg.description.toLowerCase().includes(query)
      )
      .sort(([a], [b]) => a.localeCompare(b));
  });

  function managePackage(packageName: string) {
    goto(`/node/${packageName}`);
  }

  onMount(() => {
    operationalState.startPolling();
    operationalState.refresh().catch((e) => {
      console.error("Failed to refresh operational state:", e);
    });
    packagesState.loadPackages();
    packagesState.syncInstalledPackages().catch((e) => {
      console.error("Failed to sync installed packages:", e);
    });

    return () => {
      operationalState.stopPolling();
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
    >
  </div>

  {#if catalogState.status === "error"}
    <Card.Root>
      <Card.Content class="flex items-center justify-between">
        <p class="text-sm text-muted-foreground">Failed to load packages.</p>
        <Button
          size="sm"
          variant="outline"
          onclick={() => packagesState.loadPackages({ force: true })}
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
          onclick={() => packagesState.loadInstalledPackages({ force: true })}
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
        <PackageCard
          name={name}
          description={pkg.description}
          onManage={managePackage}
          onInstalled={managePackage}
        />
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
