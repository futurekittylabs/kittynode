<script lang="ts">
import { onMount, onDestroy } from "svelte";
import { invoke } from "@tauri-apps/api/core";
import Convert from "ansi-to-html";

const convert = new Convert();

let {
  containerName,
  tailLines,
}: { containerName: string; tailLines: number | null } = $props();

let logs: string[] = $state([]);
let logsElement: HTMLDivElement;
let shouldAutoScroll = $state(true);
let polling: NodeJS.Timeout;

async function fetchLogs() {
  try {
    const newLogs = await invoke<string[]>("get_container_logs", {
      containerName,
      tailLines,
    });

    // Convert ANSI escape sequences to HTML
    logs = newLogs.map((log) => convert.toHtml(log));

    // Schedule scroll after render if we should auto scroll
    if (shouldAutoScroll) {
      queueMicrotask(() => {
        if (logsElement) {
          logsElement.scrollTop = logsElement.scrollHeight;
        }
      });
    }
  } catch (error) {
    console.error("Failed to fetch logs:", error);
  }
}

function startPolling() {
  polling = setInterval(fetchLogs, 2000);
}

onMount(() => {
  fetchLogs();
  startPolling();
});

onDestroy(() => {
  if (polling) clearInterval(polling);
});

function handleScroll(e: Event) {
  const target = e.target as HTMLDivElement;
  const isAtBottom =
    Math.abs(target.scrollHeight - target.clientHeight - target.scrollTop) < 1;
  shouldAutoScroll = isAtBottom;
}
</script>

<div class="flex min-w-0 flex-col gap-3">
	<div
		bind:this={logsElement}
		onscroll={handleScroll}
		class="h-[400px] w-full overflow-y-auto overflow-x-auto rounded-lg border bg-background/60"
	>
		<div class="flex min-w-0 flex-col gap-2 p-4 font-mono text-sm leading-6 text-muted-foreground">
			{#if logs.length === 0}
				<div class="text-muted-foreground">No logs available</div>
			{:else}
				{#each logs as log}
					<div class="whitespace-pre-wrap break-words [overflow-wrap:anywhere]">
						{@html log}
					</div>
				{/each}
			{/if}
		</div>
	</div>
</div>
