<script lang="ts">
import { goto } from "$app/navigation";
import { page } from "$app/state";
import { Button } from "$lib/components/ui/button";
import * as Card from "$lib/components/ui/card";
import { ArrowLeft } from "@lucide/svelte";

const packageName = $derived(page.params.name || "");
const parentHref = $derived(packageName ? `/node/${packageName}` : "/node");

$effect(() => {
  if (packageName && packageName !== "ethereum") {
    void goto(parentHref);
  }
});
</script>

{#if packageName === "ethereum"}
  <div class="mx-auto flex w-full max-w-4xl flex-col gap-6">
    <div>
      <Button
        variant="ghost"
        size="sm"
        class="gap-2 px-2"
        onclick={() => void goto(parentHref)}
      >
        <ArrowLeft class="h-4 w-4" />
        <span>Back to Manage Node</span>
      </Button>
    </div>
    <Card.Root>
      <Card.Content class="space-y-2">
        <p class="text-sm text-muted-foreground">
          This is the validator config page, where soon you'll be able to manage your validator. In the meantime, you can manage your validators with the Kittynode CLI.
        </p>
        <p class="text-sm text-muted-foreground">
          Install it at
          <a
            href="https://kittynode.com/download"
            target="_blank"
            rel="noreferrer"
            class="link"
            >kittynode.com/download</a
          >.
        </p>
      </Card.Content>
    </Card.Root>
  </div>
{:else}
  <div class="space-y-4">
    <Button
      variant="ghost"
      size="sm"
      class="gap-2 px-2"
      onclick={() => void goto(parentHref)}
    >
      <ArrowLeft class="h-4 w-4" />
      <span>Back to Manage Node</span>
    </Button>
  </div>
{/if}
