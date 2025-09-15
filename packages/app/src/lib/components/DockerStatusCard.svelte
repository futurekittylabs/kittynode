<script lang="ts">
import * as Card from "$lib/components/ui/card";
import { dockerStatus } from "$stores/dockerStatus.svelte";
import { CircleCheck, CircleAlert, Server, Loader2 } from "@lucide/svelte";

export let showServerIcon: boolean = false;
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
      {#if dockerStatus.isStarting}
        <Loader2 class="h-4 w-4 text-blue-500 animate-spin" />
        <span class="text-sm font-medium">Starting Docker...</span>
      {:else if dockerStatus.isRunning}
        <CircleCheck class="h-4 w-4 text-green-500" />
        <span class="text-sm font-medium">Running</span>
      {:else}
        <CircleAlert class="h-4 w-4 text-yellow-500" />
        <span class="text-sm font-medium">Not Running</span>
      {/if}
    </div>
  </Card.Content>
</Card.Root>
