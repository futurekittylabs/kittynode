<script lang="ts">
import { cn } from "$lib/utils";
import { getContext } from "svelte";
import type { Writable } from "svelte/store";

type SidebarState = {
  isOpen: boolean;
  isMobile: boolean;
};

let {
  class: className = "",
  side = "left",
  children,
  ...restProps
}: {
  class?: string;
  side?: "left" | "right";
  children?: any;
} = $props();

const context = getContext<{
  state: Writable<SidebarState>;
  toggle: () => void;
  setOpen: (value: boolean) => void;
}>("sidebar");

let state = $state<SidebarState>({ isOpen: true, isMobile: false });

$effect(() => {
  if (context?.state) {
    const unsubscribe = context.state.subscribe((value) => {
      state = value;
    });
    return unsubscribe;
  }
});

const isOpen = $derived(state?.isOpen ?? true);
const isMobile = $derived(state?.isMobile ?? false);

function handleOverlayClick() {
  if (isMobile) {
    context?.setOpen(false);
  }
}
</script>

{#if isMobile && isOpen}
  <button
    type="button"
    class="fixed inset-0 z-40 bg-background/80 backdrop-blur-sm"
    onclick={handleOverlayClick}
    aria-label="Close sidebar"
  ></button>
{/if}

<aside
  class={cn(
    "fixed top-0 z-40 h-screen w-64 border-r bg-sidebar transition-transform duration-300",
    {
      "left-0": side === "left",
      "right-0": side === "right",
      "-translate-x-full": side === "left" && !isOpen && isMobile,
      "translate-x-full": side === "right" && !isOpen && isMobile,
      "translate-x-0": isOpen || !isMobile,
      "md:sticky md:translate-x-0": !isMobile,
    },
    className
  )}
  {...restProps}
>
  <div class="flex h-full flex-col">
    {@render children?.()}
  </div>
</aside>
