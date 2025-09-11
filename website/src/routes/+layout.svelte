<script lang="ts">
import "../app.css";
import favicon from "$lib/assets/favicon.ico";
import { ModeWatcher, toggleMode } from "mode-watcher";
import { Button } from "$lib/components/ui/button/index.js";
import { Sun, Moon, Github, BookOpen } from "@lucide/svelte";

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
			<a href="/" class="flex items-center">
				<img src="/kittynode-wordmark-light.png" alt="Kittynode" class="h-8 dark:hidden" />
				<img src="/kittynode-wordmark-dark.png" alt="Kittynode" class="hidden h-8 dark:block" />
			</a>
			
			<nav class="flex items-center gap-2">
				<Button href="https://github.com/blackkittylabs/kittynode" variant="ghost" size="sm">
					<Github class="h-4 w-4" />
					GitHub
				</Button>
				<Button href="/docs" variant="ghost" size="sm">
					<BookOpen class="h-4 w-4" />
					Docs
				</Button>
				<Button
					variant="ghost"
					size="icon"
					class="cursor-pointer"
					onclick={toggleMode}
					aria-label="Toggle theme"
				>
					<Sun class="h-5 w-5 dark:hidden" />
					<Moon class="hidden h-5 w-5 dark:block" />
				</Button>
			</nav>
		</div>
	</header>

	<main class="flex-1">
		{@render children?.()}
	</main>

	<footer class="border-t py-6">
		<div class="container mx-auto px-4 text-center text-sm text-muted-foreground">
			Kittynode is <a href="https://github.com/blackkittylabs/kittynode" class="hover:underline">free software</a> released under the MIT License.
		</div>
	</footer>
</div>
