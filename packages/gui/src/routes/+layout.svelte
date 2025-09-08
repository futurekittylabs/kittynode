<script lang="ts">
import "../app.css";
import { onMount } from "svelte";
import { windowShownStore } from "$stores/windowShown.svelte.ts";
import { initializedStore } from "$stores/initialized.svelte";
import { ModeWatcher, mode } from "mode-watcher";
import Splash from "./Splash.svelte";
import UpdateBanner from "./UpdateBanner.svelte";
import { platform } from "@tauri-apps/plugin-os";
import { Toaster } from "svelte-sonner";
import { getVersion } from "@tauri-apps/api/app";
import * as Sidebar from "$lib/components/ui/sidebar";
import {
  House,
  HeartPulse,
  Settings,
  Package2,
  Activity,
} from "@lucide/svelte";
import { packagesStore } from "$stores/packages.svelte";
import { page } from "$app/state";

const { children } = $props();

const currentPath = $derived(page.url?.pathname || "");

const navigationItems = [
  { icon: House, label: "Dashboard", href: "/" },
  { icon: Package2, label: "Package Store", href: "/packages" },
  { icon: HeartPulse, label: "System Info", href: "/system-info" },
  { icon: Settings, label: "Settings", href: "/settings" },
];

let appVersion = $state("");
let versionError = $state(false);

onMount(async () => {
  await windowShownStore.show();
  await packagesStore.loadPackages();
  await packagesStore.loadInstalledPackages();

  try {
    appVersion = await getVersion();
    versionError = false;
  } catch (error) {
    console.error("Failed to get app version:", error);
    versionError = true;
  }
});
</script>

<ModeWatcher />
<Toaster closeButton position="top-right" richColors theme={mode.current} />
{#if !initializedStore.initialized}
  <Splash />
{:else}
  <Sidebar.Provider>
    <Sidebar.Root variant="inset">
      <Sidebar.Header>
        <div class="flex items-center gap-3 px-2">
          <img
            src="/images/kittynode-logo-circle.png"
            alt="Kittynode"
            class="h-8 w-8"
          />
          <span class="text-lg font-semibold">Kittynode</span>
        </div>
      </Sidebar.Header>

      <Sidebar.Content>
        <Sidebar.Menu>
          {#each navigationItems as item}
            <Sidebar.MenuItem>
              <Sidebar.MenuButton
                isActive={currentPath === item.href || currentPath.startsWith(item.href + "/")}
              >
                {#snippet child({ props })}
                  <a href={item.href} {...props}>
                    <item.icon class="h-4 w-4" />
                    <span>{item.label}</span>
                  </a>
                {/snippet}
              </Sidebar.MenuButton>
            </Sidebar.MenuItem>
          {/each}
        </Sidebar.Menu>

        {#if packagesStore.installedPackages.length > 0}
          <div class="mt-6">
            <div class="px-3 text-xs font-medium text-muted-foreground uppercase tracking-wider">Installed Nodes</div>
            <Sidebar.Menu class="mt-2">
              {#each packagesStore.installedPackages as pkg}
                <Sidebar.MenuItem>
                  <Sidebar.MenuButton
                    isActive={currentPath.startsWith(`/node/${pkg.name}`)}
                  >
                    {#snippet child({ props })}
                      <a href={`/node/${pkg.name}`} {...props}>
                        <Activity class="h-4 w-4" />
                        <span>{pkg.name}</span>
                      </a>
                    {/snippet}
                  </Sidebar.MenuButton>
                </Sidebar.MenuItem>
              {/each}
            </Sidebar.Menu>
          </div>
        {/if}
      </Sidebar.Content>

      <Sidebar.Footer>
        <div class="px-2 py-2 text-xs text-muted-foreground">
          {#if appVersion}
            Version {appVersion}
          {:else if versionError}
            <span class="opacity-50">Failed to get version</span>
          {:else}
            <span class="opacity-50">Loading version...</span>
          {/if}
        </div>
      </Sidebar.Footer>

      <Sidebar.Rail />
    </Sidebar.Root>

    <Sidebar.Inset>
      <!-- Desktop header with always-visible sidebar toggle -->
      <header class="hidden md:flex h-12 items-center gap-4 border-b px-4">
        <Sidebar.Trigger />
      </header>

      <header class="flex h-14 items-center gap-4 border-b px-4 md:hidden">
        <Sidebar.Trigger />
        <div class="flex items-center gap-2">
          <img
            src="/images/kittynode-logo-circle.png"
            alt="Kittynode"
            class="h-6 w-6"
          />
          <span class="text-lg font-semibold">Kittynode</span>
        </div>
      </header>

      <div class="flex-1 overflow-y-auto">
        <div class="container mx-auto px-4 py-6">
          {#if !["ios", "android"].includes(platform())}
            <UpdateBanner />
          {/if}
          {@render children()}
        </div>
      </div>
    </Sidebar.Inset>
</Sidebar.Provider>
{/if}
