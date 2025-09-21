<script lang="ts">
import "../app.css";
import { onMount } from "svelte";
import { windowShownStore } from "$stores/windowShown.svelte.ts";
import { initializedStore } from "$stores/initialized.svelte";
import { appConfigStore } from "$stores/appConfig.svelte";
import { ModeWatcher, mode } from "mode-watcher";
import Splash from "./Splash.svelte";
import { platform } from "@tauri-apps/plugin-os";
import { updates } from "$stores/updates.svelte";
import { Toaster } from "svelte-sonner";
import { getVersion } from "@tauri-apps/api/app";
import { invoke } from "@tauri-apps/api/core";
import { Button } from "$lib/components/ui/button";
import UpdateBanner from "$lib/components/UpdateBanner.svelte";
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
import { operationalStateStore } from "$stores/operationalState.svelte";
import { page } from "$app/state";

const { children } = $props();

const currentPath = $derived(page.url?.pathname || "");

const installedState = $derived(packagesStore.installedState);
const installedNodes = $derived(
  installedState.status === "ready" ? packagesStore.installedPackages : [],
);

$effect(() => {
  packagesStore.handleOperationalStateChange(operationalStateStore.state);
});

const navigationItems = [
  { icon: House, label: "Dashboard", href: "/" },
  { icon: Package2, label: "Package Store", href: "/packages" },
  { icon: HeartPulse, label: "System Info", href: "/system-info" },
  { icon: Settings, label: "Settings", href: "/settings" },
];

let appVersion = $state("");
let versionError = $state(false);
let onboardingCompleted = $state(false);
let checkingOnboarding = $state(true);

onMount(async () => {
  await windowShownStore.show();

  try {
    await appConfigStore.load();
  } catch (e) {
    console.error(`Failed to load Kittynode config: ${e}`);
  }

  // Check if onboarding has been completed
  try {
    onboardingCompleted = await invoke("get_onboarding_completed");
    if (onboardingCompleted) {
      // Skip splash screen without re-initializing config
      // Just set the initialized flag to bypass the splash
      await initializedStore.fakeInitialize();
    }
  } catch (e) {
    console.error(`Failed to check onboarding status: ${e}`);
    // Treat as not completed if we can't check
    onboardingCompleted = false;
  }
  checkingOnboarding = false;

  await packagesStore.loadPackages();
  await operationalStateStore.refresh();
  await packagesStore.loadInstalledPackages();

  try {
    appVersion = await getVersion();
    versionError = false;
  } catch (error) {
    console.error(`Failed to get app version: ${error}`);
    versionError = true;
  }

  try {
    await updates.getUpdate();
  } catch (e) {
    console.error(`Failed to check for update: ${e}`);
  }
});
</script>

<ModeWatcher />
<Toaster position="top-right" richColors theme={mode.current} />
{#if checkingOnboarding}
  <!-- Show nothing while checking onboarding status -->
{:else if !onboardingCompleted && !initializedStore.initialized}
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

        {#if installedState.status === "ready" && installedNodes.length > 0}
          <Sidebar.Group class="px-2 py-2 md:p-2">
            <Sidebar.GroupLabel>Installed Nodes</Sidebar.GroupLabel>
            <Sidebar.Menu>
              {#each installedNodes as pkg}
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
                  href="https://github.com/futurekittylabs/kittynode/discussions/new?category=feedback"
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
                  href="https://github.com/futurekittylabs/kittynode"
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
          {#if !["ios", "android"].includes(platform()) && updates.hasUpdate && !updates.isDismissed}
            <UpdateBanner />
          {/if}
          {@render children()}
        </div>
      </div>
    </Sidebar.Inset>
</Sidebar.Provider>
{/if}
