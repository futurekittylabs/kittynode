<script lang="ts">
let { class: className = "", ...rest } = $props();
</script>

<svg
  xmlns="http://www.w3.org/2000/svg"
  viewBox="0 0 24 24"
  fill="none"
  stroke="currentColor"
  stroke-width="2"
  stroke-linecap="round"
  stroke-linejoin="round"
  class={className}
  {...rest}
>
  <path d="M12 3c2 0 3 1 3 3v2a4 4 0 0 1 2 3v5a3 3 0 0 1-3 3h-4a3 3 0 0 1-3-3v-5a4 4 0 0 1 2-3V6c0-2 1-3 3-3Z" />
  <path d="M8 13c-1 0-2 .5-2 2s1 2 2 2" />
  <path d="M16 13c1 0 2 .5 2 2s-1 2-2 2" />
</svg>
