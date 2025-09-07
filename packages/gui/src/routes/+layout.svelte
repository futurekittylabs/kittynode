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
import {
  SidebarProvider,
  Sidebar,
  SidebarHeader,
  SidebarContent,
  SidebarFooter,
  SidebarMenu,
  SidebarMenuItem,
  SidebarMenuButton,
  SidebarTrigger,
} from "$lib/components/ui/sidebar";
import {
  House,
  HeartPulse,
  Settings,
  Package2,
  Activity,
} from "@lucide/svelte";
import { packagesStore } from "$stores/packages.svelte";

const { children } = $props();

const navigationItems = [
  { icon: House, label: "Dashboard", href: "/" },
  { icon: Package2, label: "Package Store", href: "/packages" },
  { icon: HeartPulse, label: "System Info", href: "/system-info" },
  { icon: Settings, label: "Settings", href: "/settings" },
];

let appVersion = $state("");

onMount(async () => {
  await windowShownStore.show();
  await packagesStore.loadPackages();
  await packagesStore.loadInstalledPackages();

  try {
    appVersion = await getVersion();
  } catch (error) {
    console.error("Failed to get app version:", error);
    appVersion = "0.23.0"; // Fallback to version from Cargo.toml
  }
});
</script>

<ModeWatcher />
<Toaster richColors theme={mode.current} />
{#if !initializedStore.initialized}
  <Splash />
{:else}
  <SidebarProvider defaultOpen={true}>
    <div class="flex h-screen w-full">
      <Sidebar>
        <SidebarHeader>
          <div class="flex items-center gap-3 px-2">
            <img 
              src="/images/kittynode-logo-circle.png" 
              alt="Kittynode" 
              class="h-8 w-8"
            />
            <span class="text-lg font-semibold">Kittynode</span>
          </div>
        </SidebarHeader>
        
        <SidebarContent>
          <SidebarMenu>
            {#each navigationItems as item}
              <SidebarMenuItem>
                <SidebarMenuButton href={item.href}>
                  <item.icon class="h-4 w-4" />
                  <span>{item.label}</span>
                </SidebarMenuButton>
              </SidebarMenuItem>
            {/each}
          </SidebarMenu>
          
          {#if packagesStore.installedPackages.length > 0}
            <div class="mt-6">
              <div class="px-3 text-xs font-medium text-muted-foreground uppercase tracking-wider">Installed Nodes</div>
              <SidebarMenu class="mt-2">
                {#each packagesStore.installedPackages as pkg}
                  <SidebarMenuItem>
                    <SidebarMenuButton href="/node/{pkg.name}">
                      <Activity class="h-4 w-4" />
                      <span>{pkg.name}</span>
                    </SidebarMenuButton>
                  </SidebarMenuItem>
                {/each}
              </SidebarMenu>
            </div>
          {/if}
        </SidebarContent>
        
        <SidebarFooter>
          <div class="px-2 py-2 text-xs text-muted-foreground">
            {#if appVersion}
              Version {appVersion}
            {:else}
              <span class="opacity-50">Loading version...</span>
            {/if}
          </div>
        </SidebarFooter>
      </Sidebar>
      
      <div class="flex-1 flex flex-col">
        <header class="flex h-14 items-center gap-4 border-b px-4 md:hidden">
          <SidebarTrigger />
          <div class="flex items-center gap-2">
            <img 
              src="/images/kittynode-logo-circle.png" 
              alt="Kittynode" 
              class="h-6 w-6"
            />
            <span class="text-lg font-semibold">Kittynode</span>
          </div>
        </header>
        
        <main class="flex-1 overflow-y-auto">
          <div class="container mx-auto px-4 py-6">
            {#if !["ios", "android"].includes(platform())}
              <UpdateBanner />
            {/if}
            {@render children()}
          </div>
        </main>
      </div>
    </div>
  </SidebarProvider>
{/if}

<style>
  :global(html, body) {
    height: 100%;
    margin: 0;
    padding: 0;
    overflow: hidden;
  }
</style>
