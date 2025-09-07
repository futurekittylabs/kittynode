<script lang="ts">
import "../app.css";
import { onMount } from "svelte";
import { windowShownStore } from "$stores/windowShown.svelte.ts";
import { initializedStore } from "$stores/initialized.svelte";
import { ModeWatcher, mode } from "mode-watcher";
import Splash from "./Splash.svelte";
import Navigation from "./Navigation.svelte";
import UpdateBanner from "./UpdateBanner.svelte";
import { platform } from "@tauri-apps/plugin-os";
import { Toaster } from "svelte-sonner";

const { children } = $props();

onMount(async () => {
  await windowShownStore.show();
});
</script>

<ModeWatcher />
<Toaster richColors theme={mode.current} />
{#if !initializedStore.initialized}
  <Splash />
{:else}
  <div class="flex flex-col h-screen">
    <main class="flex-1 overflow-y-auto">
      <div class="container mx-auto pt-8">
        {#if !["ios", "android"].includes(platform())}
          <UpdateBanner />
        {/if}
        {@render children()}
        <div class="h-32 md:h-10"></div>
      </div>
    </main>
    <nav class="sticky bottom-0 left-0 right-0 border-t">
      <div class="container mx-auto py-4">
        <Navigation />
      </div>
    </nav>
  </div>
{/if}

<style>
  :global(html, body) {
    height: 100%;
    margin: 0;
    padding: 0;
    overflow: hidden;
  }
</style>
