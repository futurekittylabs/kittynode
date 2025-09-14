<script lang="ts">
import { Button } from "$lib/components/ui/button";
import { Download } from "@lucide/svelte";
import { updates } from "$stores/updates.svelte";

const handleInstall = () => updates.installUpdate();
const handleDismiss = () => updates.dismiss();
</script>

<div class="p-4 mb-4 border rounded-lg bg-muted flex items-center justify-between">
  <span class="text-sm">A new version of Kittynode is available! ✨</span>
  <div class="flex items-center gap-3">
    <Button
      size="sm"
      onclick={handleInstall}
      disabled={updates.isProcessing}
    >
      <Download />
      {updates.isProcessing ? "Installing..." : "Install update"}
    </Button>
    <Button
      size="sm"
      variant="outline"
      onclick={handleDismiss}
    >
      Dismiss
    </Button>
  </div>
</div>
