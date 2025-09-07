<script lang="ts">
import { cn } from "$lib/utils";
import { getContext } from "svelte";
import { Menu } from "@lucide/svelte";
import { Button } from "$lib/components/ui/button";

let {
  class: className = "",
  ...restProps
}: {
  class?: string;
} = $props();

const context = getContext<{
  toggle: () => void;
}>("sidebar");
</script>

<Button
  variant="ghost"
  size="icon"
  class={cn("", className)}
  onclick={context?.toggle}
  {...restProps}
>
  <Menu class="h-5 w-5" />
  <span class="sr-only">Toggle sidebar</span>
</Button>
