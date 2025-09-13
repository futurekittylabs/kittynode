<script lang="ts">
import { Button } from "$lib/components/ui/button/index.js";
import { Apple } from "@lucide/svelte";
import Windows from "$lib/components/icons/Windows.svelte";
import Linux from "$lib/components/icons/Linux.svelte";
</script>

<div class="container mx-auto px-4 py-16">
  <div class="mx-auto flex max-w-md flex-col items-center text-center">
    <picture>
      <source type="image/webp" srcset="/kittynode-logo-app-160.webp" />
      <img
        src="/kittynode-logo-app-160.png"
        alt="Kittynode logo"
        class="mb-8 h-20 w-20"
        width="80"
        height="80"
        decoding="async"
      />
    </picture>
    <h1 class="text-3xl font-medium">Download Kittynode</h1>
    <p class="mt-2 text-sm text-muted-foreground">Version 0.0.0</p>
  </div>

  <div class="mx-auto mt-12 grid max-w-4xl gap-6 sm:grid-cols-3">
    <div class="flex flex-col items-center rounded-lg border p-6 text-center">
      <Apple class="mb-4 h-12 w-12" />
      <Button href="#" class="w-full">Download for macOS</Button>
    </div>
    <div class="flex flex-col items-center rounded-lg border p-6 text-center">
      <Windows class="mb-4 h-12 w-12" />
      <Button href="#" class="w-full">Download for Windows</Button>
    </div>
    <div class="flex flex-col items-center rounded-lg border p-6 text-center">
      <Linux class="mb-4 h-12 w-12" />
      <Button href="#" class="w-full">Download for Linux</Button>
    </div>
  </div>
</div>
