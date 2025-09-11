<script lang="ts">
import "../app.css";
import favicon from "$lib/assets/favicon.ico";
import { ModeWatcher } from "mode-watcher";
import { Button } from "$lib/components/ui/button/index.js";
import { Github, BookOpen } from "@lucide/svelte";

let { children } = $props();
</script>

<ModeWatcher />

<svelte:head>
	<link rel="icon" href={favicon} />
	<title>Kittynode | Control center for world computer operators</title>
	<meta name="description" content="Control center for world computer operators." />
</svelte:head>

<div class="flex min-h-screen flex-col">
	<header class="border-b">
		<div class="container mx-auto flex h-16 items-center justify-between px-4">
			<a href="/" class="flex items-center gap-2">
				<img src="/kittynode-logo-app.png" alt="Kittynode" class="h-10 w-10" />
				<span class="text-lg font-semibold">Kittynode</span>
			</a>

			<nav class="flex items-center gap-2">
				<Button href="https://github.com/blackkittylabs/kittynode" variant="ghost" size="sm">
					<Github class="h-4 w-4" />
					GitHub
				</Button>
				<Button href="https://docs.kittynode.com" variant="ghost" size="sm">
					<BookOpen class="h-4 w-4" />
					Docs
				</Button>
			</nav>
		</div>
	</header>

	<main class="flex-1">
		{@render children?.()}
	</main>

	<footer class="border-t py-6">
		<div class="container mx-auto px-4 text-center text-sm text-muted-foreground">
			Kittynode is <a href="https://github.com/blackkittylabs/kittynode" class="link">free software</a> released under the MIT License.
		</div>
	</footer>
</div>
