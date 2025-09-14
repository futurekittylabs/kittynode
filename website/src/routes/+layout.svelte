<script lang="ts">
import "../app.css";
import { ModeWatcher } from "mode-watcher";
import { Button } from "$lib/components/ui/button/index.js";
import { Github, BookOpen } from "@lucide/svelte";

let { children } = $props();
</script>

<ModeWatcher />

<svelte:head>
	<link rel="icon" type="image/x-icon" href="/favicon.ico" />
	<link rel="icon" type="image/png" sizes="16x16" href="/favicon-16x16.png" />
	<link rel="icon" type="image/png" sizes="32x32" href="/favicon-32x32.png" />
	<link rel="apple-touch-icon" sizes="180x180" href="/apple-touch-icon.png" />
	<title>Kittynode | Operate the world computer</title>
	<meta name="description" content="Control center for world computer operators." />
</svelte:head>

<div class="flex min-h-screen flex-col">
	<header class="border-b">
		<div class="container max-w-6xl mx-auto flex h-16 items-center justify-between px-6">
			<a href="/" class="wordmark">
				<picture>
					<source type="image/webp" srcset="/kittynode-logo-app-160.webp" />
					<img
						src="/kittynode-logo-app-160.png"
						alt="Kittynode logo"
						class="app-logo"
						width="40"
						height="40"
						decoding="async"
					/>
				</picture>
				<span class="wordmark-text">Kittynode</span>
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
		<div class="container max-w-6xl mx-auto px-6 text-center text-sm text-muted-foreground">
			Kittynode is <a href="https://github.com/blackkittylabs/kittynode" class="link">free software</a> released under the MIT License.
		</div>
	</footer>
</div>

<style>
	.wordmark {
		display: flex;
		align-items: center;
		gap: 0.375rem;
		text-decoration: none;
	}

	.app-logo {
		height: 2.5rem;
		width: 2.5rem;
	}

	.wordmark-text {
		font-size: 1.5rem;
		font-weight: 500;
	}
</style>
