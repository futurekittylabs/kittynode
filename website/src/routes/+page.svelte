<script lang="ts">
import { Button } from "$lib/components/ui/button/index.js";
import { Download } from "@lucide/svelte";
</script>

<div class="flex flex-1 items-center justify-center">
	<div class="container mx-auto px-4 py-16">
		<div class="mx-auto max-w-3xl text-center">
			<h1 class="text-3xl font-bold tracking-tight sm:text-4xl md:text-5xl lg:text-6xl">
				Operate the world computer
			</h1>
			<p class="mt-6 text-base text-muted-foreground sm:text-lg">
				Kittynode is a control center for world computer operators.
			</p>
			<div class="mt-12 flex flex-col items-center gap-4">
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
