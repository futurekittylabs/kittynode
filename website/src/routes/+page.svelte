<script lang="ts">
import { Download } from "@lucide/svelte";
import { Button } from "$lib/components/ui/button/index.js";
</script>

<div class="flex flex-1 items-center justify-center">
	<div class="container mx-auto px-4 py-16">
		<div class="mx-auto max-w-3xl text-center">
			<h1 class="text-4xl font-normal tracking-tight sm:text-5xl md:text-5xl lg:text-6xl">
				Operate the world<br />computer
			</h1>
			<p class="mt-6 text-lg text-muted-foreground sm:text-lg">
				Kittynode is a control center for world computer operators.
			</p>
			<img src="/black-kitty.gif" alt="Black Kitty" class="nyan-cat mt-12" />
			<div class="mt-8 flex flex-col items-center gap-4">
				<Button href="https://github.com/blackkittylabs/kittynode/releases" size="lg" class="gap-2">
					<Download class="h-5 w-5" />
					Download now
				</Button>
				<p class="text-sm text-muted-foreground">
					Available for macOS, Linux, and Windows
				</p>
			</div>
		</div>
	</div>
</div>

<style>
	.nyan-cat {
		width: 180px;
		height: auto;
		image-rendering: pixelated;
		display: block;
		margin-left: auto;
		margin-right: auto;
	}

	@media (min-width: 640px) {
		.nyan-cat {
			width: 220px;
		}
	}
</style>
