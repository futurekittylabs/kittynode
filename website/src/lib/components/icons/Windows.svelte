<script lang="ts">
let { class: className = "", ...rest } = $props();
</script>

<svg
  xmlns="http://www.w3.org/2000/svg"
  viewBox="0 0 24 24"
  fill="currentColor"
  class={className}
  {...rest}
>
  <path d="M3 5L11 4v7H3zM13 3l8-1v9h-8zM3 13h8v7l-8-1zM13 13h8v8l-8-1z" />
</svg>
