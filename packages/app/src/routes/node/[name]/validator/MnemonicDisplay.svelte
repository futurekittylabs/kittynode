<script lang="ts">
import { Button } from "$lib/components/ui/button";
import { ClipboardCopy } from "@lucide/svelte";
import { notifySuccess, notifyError } from "$utils/notify";

interface Props {
  mnemonic: string;
  showMnemonic: boolean;
  onToggleShow: () => void;
}

let { mnemonic, showMnemonic, onToggleShow }: Props = $props();

async function copyToClipboard() {
  try {
    await navigator.clipboard.writeText(mnemonic);
    notifySuccess("Mnemonic copied to clipboard");
  } catch (error) {
    notifyError("Failed to copy to clipboard", error);
  }
}
</script>

<div class="space-y-3">
  <div class="flex items-center justify-between">
    <p class="text-xs font-medium uppercase tracking-wide text-muted-foreground">
      Recovery phrase (mnemonic)
    </p>
    <div class="flex gap-2">
      <Button size="sm" variant="outline" onclick={onToggleShow}>
        {showMnemonic ? "Hide mnemonic" : "Reveal mnemonic"}
      </Button>
      {#if showMnemonic}
        <Button size="sm" variant="outline" onclick={copyToClipboard} class="gap-1">
          <ClipboardCopy class="h-3 w-3" />
          Copy
        </Button>
      {/if}
    </div>
  </div>
  {#if showMnemonic}
    <p class="break-words rounded-md bg-background p-3 font-mono text-sm leading-relaxed">
      {mnemonic}
    </p>
  {:else}
    <p class="rounded-md bg-muted p-3 text-sm text-muted-foreground">
      Mnemonic hidden. Click "Reveal mnemonic" to view.
    </p>
  {/if}
</div>