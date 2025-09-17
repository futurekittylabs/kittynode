<script lang="ts">
import { Download } from "@lucide/svelte";
import { Button } from "$lib/components/ui/button/index.js";
</script>

<div class="flex flex-1 items-center justify-center">
	<div class="container max-w-6xl mx-auto px-6 py-16">
		<div class="mx-auto max-w-3xl text-center">
			<h1 class="hero-heading">
				Operate the world<br />computer
			</h1>
			<p class="hero-subtitle mt-6 text-muted-foreground">
				Kittynode is a control center for world computer operators.
			</p>
			<img src="/black-kitty.gif" alt="Black Kitty" class="nyan-cat mt-12" />
			<div class="mt-8 flex flex-col items-center gap-4">
				<Button href="/download" size="lg" class="gap-2">
					<Download class="h-5 w-5" />
					Download now
				</Button>
				<p class="text-sm text-muted-foreground">
					Available for Linux, macOS, and Windows
				</p>
			</div>
		</div>
	</div>
</div>

<style>
	.hero-heading {
		font-size: 2.25rem;
		font-weight: 500;
	}

	.hero-subtitle {
		font-size: 1rem;
	}

	@media (min-width: 640px) {
		.hero-heading {
			font-size: 3.5rem;
			font-weight: 500;
		}

		.hero-subtitle {
			font-size: 1.125rem;
		}
	}

	.nyan-cat {
		width: 180px;
		height: auto;
		min-width: 180px;
		max-width: none;
		image-rendering: pixelated;
		display: block;
		margin-left: auto;
		margin-right: auto;
	}

	@media (min-width: 640px) {
		.nyan-cat {
			width: 220px;
			min-width: 220px;
		}
	}
</style>
