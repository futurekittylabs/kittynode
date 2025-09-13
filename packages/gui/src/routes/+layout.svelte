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
  MessageSquare,
  Github,
  Users,
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
<Toaster position="top-right" richColors theme={mode.current} />
{#if !initializedStore.initialized}
  <Splash />
{:else}
  <Sidebar.Provider>
    <Sidebar.Root variant="inset">
      <Sidebar.Header>
        <div class="flex items-center gap-2.5 px-2 py-1">
          <img
            src="/images/kittynode-logo-app-no-padding.png"
            alt="Kittynode Logo"
            class="h-8 w-8"
          />
          <span class="kittynode-brand">Kittynode</span>
        </div>
      </Sidebar.Header>

      <Sidebar.Content>
        <Sidebar.Group class="px-2 py-2 md:p-2">
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
        </Sidebar.Group>

        {#if packagesStore.installedPackages.length > 0}
          <Sidebar.Group class="px-2 py-2 md:p-2">
            <Sidebar.GroupLabel>Installed Nodes</Sidebar.GroupLabel>
            <Sidebar.Menu>
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
          </Sidebar.Group>
        {/if}
      </Sidebar.Content>

      <Sidebar.Footer class="px-2 py-2 md:p-2">
        <Sidebar.Separator />
        <Sidebar.Menu>
          <Sidebar.MenuItem>
            <Sidebar.MenuButton>
              {#snippet child({ props })}
                <a
                  href="https://github.com/blackkittylabs/kittynode/discussions/new?category=feedback"
                  target="_blank"
                  rel="noreferrer noopener"
                  {...props}
                >
                  <MessageSquare class="h-4 w-4" />
                  <span>Feedback</span>
                </a>
              {/snippet}
            </Sidebar.MenuButton>
          </Sidebar.MenuItem>
          <Sidebar.MenuItem>
            <Sidebar.MenuButton>
              {#snippet child({ props })}
                <a
                  href="https://github.com/blackkittylabs/kittynode"
                  target="_blank"
                  rel="noreferrer noopener"
                  {...props}
                >
                  <Github class="h-4 w-4" />
                  <span>GitHub</span>
                </a>
              {/snippet}
            </Sidebar.MenuButton>
          </Sidebar.MenuItem>
          <Sidebar.MenuItem>
            <Sidebar.MenuButton>
              {#snippet child({ props })}
                <a
                  href="https://discord.kittynode.com"
                  target="_blank"
                  rel="noreferrer noopener"
                  {...props}
                >
                  <Users class="h-4 w-4" />
                  <span>Discord</span>
                </a>
              {/snippet}
            </Sidebar.MenuButton>
          </Sidebar.MenuItem>
        </Sidebar.Menu>
      </Sidebar.Footer>

      <Sidebar.Rail />
    </Sidebar.Root>

    <Sidebar.Inset>
      <!-- Desktop header with always-visible sidebar toggle -->
      <header class="hidden md:flex h-12 items-center gap-4 border-b px-4">
        <Sidebar.Trigger class="cursor-pointer" />
      </header>

      <!-- Mobile header: keep only the toggle button -->
      <header class="flex h-14 items-center border-b px-4 md:hidden">
        <Sidebar.Trigger class="cursor-pointer" />
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
