<script lang="ts">
import { Dialog as DialogPrimitive } from "bits-ui";
import XIcon from "@lucide/svelte/icons/x";
import type { Snippet } from "svelte";
import { cn, type WithoutChildrenOrChild } from "$lib/utils.js";
import DialogOverlay from "./dialog-overlay.svelte";

let {
  ref = $bindable(null),
  class: className,
  portalProps,
  children,
  ...restProps
}: WithoutChildrenOrChild<DialogPrimitive.ContentProps> & {
  portalProps?: DialogPrimitive.PortalProps;
  children: Snippet;
} = $props();
</script>

<DialogPrimitive.Portal {...portalProps}>
  <DialogOverlay />
  <DialogPrimitive.Content
    bind:ref
    data-slot="dialog-content"
    class={cn(
      "fixed left-1/2 top-1/2 z-50 grid w-full max-w-lg translate-x-[-50%] translate-y-[-50%] gap-4 border bg-background p-6 shadow-lg duration-200",
      "data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=open]:fade-in-0 data-[state=closed]:fade-out-0 data-[state=open]:zoom-in-95 data-[state=closed]:zoom-out-95",
      "sm:rounded-lg",
      className,
    )}
    {...restProps}
  >
    {@render children?.()}
    <DialogPrimitive.Close
      class="ring-offset-background focus-visible:ring-ring absolute right-4 top-4 rounded-xs opacity-70 transition-opacity hover:opacity-100 focus-visible:outline-hidden focus-visible:ring-2 focus-visible:ring-offset-2 disabled:pointer-events-none"
    >
      <XIcon class="size-4" />
      <span class="sr-only">Close</span>
    </DialogPrimitive.Close>
  </DialogPrimitive.Content>
</DialogPrimitive.Portal>
