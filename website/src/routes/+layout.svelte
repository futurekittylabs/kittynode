<script lang="ts">
import favicon from "$lib/assets/favicon.ico";

let { children } = $props();
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
	<title>Kittynode | Control center for world computer operators</title>
	<meta name="description" content="Control center for world computer operators." />
</svelte:head>

{@render children?.()}
