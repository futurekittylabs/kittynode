<script lang="ts">
import { Button } from "$lib/components/ui/button";

interface Props {
  stdout: string[];
  showLogs: boolean;
  onToggleLogs: () => void;
  title?: string;
}

let { stdout, showLogs, onToggleLogs, title = "CLI output" }: Props = $props();
</script>

<div class="space-y-3">
  <Button
    size="sm"
    variant="outline"
    onclick={onToggleLogs}
    class="gap-2"
  >
    {showLogs ? `Hide ${title}` : `View ${title}`}
  </Button>
  {#if showLogs}
    <pre class="max-h-52 overflow-auto rounded-md bg-background p-3 text-xs">
{stdout.join("\n")}
    </pre>
  {/if}
</div>