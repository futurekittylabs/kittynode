<script lang="ts">
import { cn } from "$lib/utils";

let {
  class: className = "",
  children,
  ...restProps
}: {
  class?: string;
  children?: any;
} = $props();
</script>

<div class={cn("border-t px-4 py-4", className)} {...restProps}>
  {@render children?.()}
</div>
