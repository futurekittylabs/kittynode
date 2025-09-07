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

<ul class={cn("space-y-1", className)} {...restProps}>
  {@render children?.()}
</ul>
