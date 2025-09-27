<script lang="ts">
import { Button } from "$lib/components/ui/button";
import { ArrowUpRight, Download } from "@lucide/svelte";
import { updates } from "$stores/updates.svelte";

const downloadsUrl = "https://kittynode.com/download";

const handleInstall = () => updates.installUpdate();
const handleDismiss = () => updates.dismiss();
</script>

<div
  class="p-4 mb-4 border rounded-lg bg-muted flex items-center justify-between"
>
  <span class="text-sm">
    {#if updates.requiresManualInstall}
      A new version of Kittynode is available! Download it from
      <a
        href={downloadsUrl}
        target="_blank"
        rel="noreferrer noopener"
        class="link"
      >
        kittynode.com/download
      </a>. ✨
    {:else}
      A new version of Kittynode is available! ✨
    {/if}
  </span>
  <div class="flex items-center gap-3">
    {#if updates.requiresManualInstall}
      <Button
        size="sm"
        href={downloadsUrl}
        target="_blank"
        rel="noreferrer noopener"
        class="gap-2"
      >
        Open Downloads
        <ArrowUpRight class="h-4 w-4" />
      </Button>
    {:else}
      <Button size="sm" onclick={handleInstall} disabled={updates.isProcessing}>
        <Download />
        {updates.isProcessing ? "Installing..." : "Install update"}
      </Button>
    {/if}
    <Button size="sm" variant="outline" onclick={handleDismiss}>Dismiss</Button>
  </div>
</div>
