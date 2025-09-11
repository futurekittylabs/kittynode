<script lang="ts">
import { Button } from "$lib/components/ui/button/index.js";
import { Download } from "@lucide/svelte";
</script>

<div class="flex flex-1 items-center justify-center">
	<div class="container mx-auto px-4 py-20">
		<div class="mx-auto max-w-4xl text-center">
			<h1 class="text-4xl font-bold tracking-tight sm:text-5xl md:text-6xl lg:text-7xl">
				Operate the world computer
			</h1>
			<p class="mt-6 text-lg text-muted-foreground sm:text-xl">
				Kittynode is a control center for world computer operators.
			</p>
			<div class="mt-10 flex flex-col items-center gap-4">
				<Button size="lg" class="gap-2">
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
