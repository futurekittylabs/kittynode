<script lang="ts">
import * as Card from "$lib/components/ui/card";
import { operationalState } from "$lib/states/operational.svelte";
import { CircleCheck, CircleAlert, Server, LoaderCircle } from "@lucide/svelte";

const { showServerIcon = false } = $props<{ showServerIcon?: boolean }>();

const state = $derived(operationalState.state);
const isStarting = $derived(operationalState.isStarting);
const loading = $derived(operationalState.loading && !state);
</script>

<Card.Root>
  <Card.Header class="pb-3">
    <Card.Title class="text-sm font-medium flex items-center justify-between">
      Docker Status
      {#if showServerIcon}
        <Server class="h-4 w-4 text-muted-foreground" />
      {/if}
    </Card.Title>
  </Card.Header>
  <Card.Content>
    <div class="flex items-center space-x-2">
      {#if isStarting}
        <LoaderCircle class="h-4 w-4 text-blue-500 animate-spin" />
        <span class="text-sm font-medium">Starting Docker...</span>
      {:else if loading}
        <LoaderCircle class="h-4 w-4 text-muted-foreground animate-spin" />
        <span class="text-sm font-medium">Checking status...</span>
      {:else if (state?.mode === "remote" && state?.canManage) || state?.dockerRunning}
        <CircleCheck class="h-4 w-4 text-green-500" />
        <span class="text-sm font-medium">Running</span>
      {:else}
        <CircleAlert class="h-4 w-4 text-yellow-500" />
        <span class="text-sm font-medium">
          {state?.mode === "remote"
            ? "Remote Docker unavailable"
            : "Not running"}
        </span>
      {/if}
    </div>
  </Card.Content>
</Card.Root>
