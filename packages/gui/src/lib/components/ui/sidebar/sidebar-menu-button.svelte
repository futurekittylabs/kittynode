<script lang="ts">
import { cn } from "$lib/utils";
import { page } from "$app/state";

let {
  class: className = "",
  href = undefined,
  isActive = false,
  children,
  ...restProps
}: {
  class?: string;
  href?: string;
  isActive?: boolean;
  children?: any;
} = $props();

const currentPath = $derived(page.url?.pathname);
const active = $derived(isActive || (href && currentPath === href));
</script>

{#if href}
  <a
    {href}
    class={cn(
      "flex items-center gap-3 rounded-lg px-3 py-2 text-sm font-medium transition-colors hover:bg-accent hover:text-accent-foreground",
      active && "bg-accent text-accent-foreground",
      className
    )}
    aria-current={active ? "page" : undefined}
    {...restProps}
  >
    {@render children?.()}
  </a>
{:else}
  <button
    class={cn(
      "flex w-full items-center gap-3 rounded-lg px-3 py-2 text-sm font-medium transition-colors hover:bg-accent hover:text-accent-foreground",
      active && "bg-accent text-accent-foreground",
      className
    )}
    {...restProps}
  >
    {@render children?.()}
  </button>
{/if}
