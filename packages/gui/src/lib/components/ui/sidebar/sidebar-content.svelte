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

<div class={cn("flex-1 overflow-y-auto px-3 py-2", className)} {...restProps}>
  {@render children?.()}
</div>
