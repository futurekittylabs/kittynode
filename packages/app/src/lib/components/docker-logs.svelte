<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { coreClient } from "$lib/client";
  import Convert from "ansi-to-html";

  let {
    containerName,
    tailLines,
  }: { containerName: string; tailLines: number | null } = $props();

  const convert = new Convert();
  let rawLogs = "";
  let logsHtml = $state("");
  let logsElement: HTMLDivElement;
  let shouldAutoScroll = $state(true);
  let polling: NodeJS.Timeout;
  const hasLogs = $derived(Boolean(logsHtml.length));

  async function fetchLogs() {
    try {
      const newLogs = await coreClient.getContainerLogs(
        containerName,
        tailLines
      );

      const combinedLogs = newLogs.join("\n");
      if (combinedLogs === rawLogs) {
        return;
      }

      const preserveScroll =
        logsElement && !shouldAutoScroll ? logsElement.scrollTop : null;
      const autoScroll = shouldAutoScroll;

      rawLogs = combinedLogs;
      logsHtml = convert.toHtml(combinedLogs);

      // Schedule scroll after render if we should auto scroll
      queueMicrotask(() => {
        if (!logsElement) {
          return;
        }

        if (autoScroll) {
          logsElement.scrollTop = logsElement.scrollHeight;
          return;
        }

        if (preserveScroll !== null) {
          logsElement.scrollTop = preserveScroll;
        }
      });
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
    if (polling) {
      clearInterval(polling);
    }
  });

  function handleScroll(e: Event) {
    const target = e.target as HTMLDivElement;
    const isAtBottom =
      Math.abs(target.scrollHeight - target.clientHeight - target.scrollTop) <
      1;
    shouldAutoScroll = isAtBottom;
  }
</script>

<div class="flex min-w-0 flex-col gap-3">
  <div
    bind:this={logsElement}
    onscroll={handleScroll}
    class="h-[400px] w-full overflow-y-auto overflow-x-auto rounded-lg border bg-background/60"
  >
    <div
      class="flex min-w-0 flex-col gap-2 p-4 font-mono text-sm leading-6 text-muted-foreground"
    >
      {#if !hasLogs}
        <div class="text-muted-foreground">No logs available</div>
      {:else}
        <div class="whitespace-pre-wrap wrap:anywhere">{@html logsHtml}</div>
      {/if}
    </div>
  </div>
</div>
