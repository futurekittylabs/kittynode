<script lang="ts">
import { Button } from "$lib/components/ui/button";
import { Download, X } from "@lucide/svelte";
import { updates } from "$stores/updates.svelte";

const handleInstall = () => updates.installUpdate();
const handleDismiss = () => updates.dismiss();
</script>

<div class="mb-4 p-4 border rounded-lg bg-muted flex items-center justify-between gap-4">
  <div class="flex items-center gap-3">
    <Download class="h-4 w-4" />
    <span class="text-sm">A new version of Kittynode is available!</span>
  </div>
  <div class="flex items-center gap-2">
    <Button
      size="sm"
      onclick={handleInstall}
      disabled={updates.isProcessing}
    >
      {updates.isProcessing ? "Installing..." : "Update"}
    </Button>
    <Button
      size="icon"
      variant="ghost"
      onclick={handleDismiss}
      class="h-8 w-8"
    >
      <X class="h-4 w-4" />
    </Button>
  </div>
</div>