<script lang="ts">
import { Dialog as DialogPrimitive } from "bits-ui";
import type { Snippet } from "svelte";
import { cn, type WithElementRef } from "$lib/utils.js";

let {
  ref = $bindable(null),
  class: className,
  children,
  ...restProps
}: WithElementRef<DialogPrimitive.TitleProps> & {
  children?: Snippet;
} = $props();
</script>

<DialogPrimitive.Title
  bind:ref
  data-slot="dialog-title"
  class={cn("text-lg font-semibold leading-none tracking-tight", className)}
  {...restProps}
>
  {@render children?.()}
</DialogPrimitive.Title>
