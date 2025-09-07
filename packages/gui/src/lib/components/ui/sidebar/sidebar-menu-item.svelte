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

<li class={cn("", className)} {...restProps}>
  {@render children?.()}
</li>
