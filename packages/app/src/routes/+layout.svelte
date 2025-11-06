<script lang="ts">
import "../app.css";
import { onMount } from "svelte";
import { initializedState } from "$lib/states/initialized.svelte";
import { appConfigState } from "$lib/states/appConfig.svelte";
import { ModeWatcher, mode } from "mode-watcher";
import Splash from "./Splash.svelte";
import { platform } from "@tauri-apps/plugin-os";
import { updates } from "$lib/states/updates.svelte";
import { Toaster } from "svelte-sonner";
import { formatPackageName } from "$lib/utils";
import { packageConfigState } from "$lib/states/packageConfig.svelte";
import UpdateBanner from "$lib/components/UpdateBanner.svelte";
import { Button } from "$lib/components/ui/button";
import * as Sidebar from "$lib/components/ui/sidebar";
import {
  House,
  HeartPulse,
  Settings,
  Package2,
  Activity,
  MessageCircleMore,
  Globe,
  Unlink,
} from "@lucide/svelte";
import { packagesState } from "$lib/states/packages.svelte";
import { operationalState } from "$lib/states/operational.svelte";
import { page } from "$app/state";
import { serverUrlState } from "$lib/states/serverUrl.svelte";
import { notifySuccess, notifyError } from "$lib/utils/notify";
import { refetchStates } from "$lib/utils/refetchStates";
import { coreClient } from "$lib/client";

const { children } = $props();

const currentPath = $derived(page.url?.pathname || "");

const installedState = $derived(packagesState.installedState);
const installedNodes = $derived(
  installedState.status === "ready" ? packagesState.installedPackages : [],
);
const remoteServerUrl = $derived(serverUrlState.serverUrl);
const remoteConnected = $derived(remoteServerUrl !== "");
const validatorGuideUrl = "https://docs.kittynode.com/guides/set-up-validator";
const remoteHelpDescription = `Follow the validator guide: ${validatorGuideUrl}`;
let remoteBannerLoading = $state(false);
let ethereumNetworkLabel = $state<string | null>(null);

$effect(() => {
  packagesState.handleOperationalStateChange(operationalState.state);
});

async function setRemote(endpoint: string) {
  if (remoteBannerLoading) {
    return;
  }

  remoteBannerLoading = true;
  const connectAction = endpoint !== "";
  try {
    if (connectAction) {
      await coreClient.checkRemoteHealth(endpoint);
    }
    await appConfigState.setServerUrl(endpoint);
    refetchStates();
    notifySuccess(
      connectAction ? "Connected to remote" : "Disconnected from remote",
    );
  } catch (error) {
    notifyError(
      connectAction
        ? "Failed to connect to remote"
        : "Failed to disconnect from remote",
      error,
      connectAction ? { description: remoteHelpDescription } : undefined,
    );
  } finally {
    remoteBannerLoading = false;
  }
}

const handleRemoteDisconnect = () => setRemote("");

const navigationItems = [
  { icon: House, label: "Dashboard", href: "/" },
  { icon: Package2, label: "Package Store", href: "/packages" },
  { icon: HeartPulse, label: "System Info", href: "/system-info" },
  { icon: Settings, label: "Settings", href: "/settings" },
];

const nodeSubNavigation: Record<string, { label: string; href: string }[]> = {
  ethereum: [
    {
      label: "Validator Config",
      href: "/node/ethereum/validator-config",
    },
  ],
};

let onboardingCompleted = $state(false);
let checkingOnboarding = $state(true);

onMount(async () => {
  try {
    await appConfigState.load();
  } catch (e) {
    console.error(`Failed to load Kittynode config: ${e}`);
  }

  // Check if onboarding has been completed
  try {
    onboardingCompleted = await coreClient.getOnboardingCompleted();
    if (onboardingCompleted) {
      // Skip splash screen without re-initializing config
      // Just set the initialized flag to bypass the splash
      await initializedState.fakeInitialize();
    }
  } catch (e) {
    console.error(`Failed to check onboarding status: ${e}`);
    // Treat as not completed if we can't check
    onboardingCompleted = false;
  }
  checkingOnboarding = false;

  await packagesState.loadPackages();
  await operationalState.refresh();
  await packagesState.syncInstalledPackages();

  // Load Ethereum network label once for display in sidebar
  try {
    const cfg = await packageConfigState.getConfig("ethereum");
    const network = cfg.values.network;
    if (typeof network === "string") {
      const trimmedNetwork = network.trim();
      if (trimmedNetwork) {
        ethereumNetworkLabel =
          trimmedNetwork[0].toUpperCase() + trimmedNetwork.slice(1);
      }
    }
  } catch (e) {
    notifyError(`Failed to load Ethereum network label: ${e}`);
  }

  try {
    await updates.getUpdate();
  } catch (e) {
    console.error(`Failed to check for update: ${e}`);
  }
});
</script>

<ModeWatcher />
<Toaster position="bottom-right" richColors theme={mode.current} />
{#if checkingOnboarding}
  <!-- Show nothing while checking onboarding status -->
{:else if !onboardingCompleted && !initializedState.initialized}
  <Splash />
{:else}
  <Sidebar.Provider>
    <Sidebar.Root variant="inset">
      <Sidebar.Header>
        <div class="flex items-center gap-2.5 py-1">
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
                  isActive={currentPath === item.href ||
                    currentPath.startsWith(item.href + "/")}
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
                {@const subPages = nodeSubNavigation[pkg.name]}
                <Sidebar.MenuItem>
                  <Sidebar.MenuButton
                    isActive={currentPath.startsWith(`/node/${pkg.name}`)}
                  >
                    {#snippet child({ props })}
                      <a href={`/node/${pkg.name}`} {...props}>
                        <Activity class="h-4 w-4" />
                        <span>
                          {pkg.name === "ethereum" && ethereumNetworkLabel
                            ? `${formatPackageName(pkg.name)} (${ethereumNetworkLabel})`
                            : formatPackageName(pkg.name)}
                        </span>
                      </a>
                    {/snippet}
                  </Sidebar.MenuButton>
                  {#if subPages}
                    <Sidebar.MenuSub>
                      {#each subPages as subPage}
                        <Sidebar.MenuSubItem>
                          <Sidebar.MenuSubButton
                            isActive={currentPath === subPage.href}
                          >
                            {#snippet child({ props })}
                              <a href={subPage.href} {...props}>
                                <span>{subPage.label}</span>
                              </a>
                            {/snippet}
                          </Sidebar.MenuSubButton>
                        </Sidebar.MenuSubItem>
                      {/each}
                    </Sidebar.MenuSub>
                  {/if}
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
                  href="https://discord.kittynode.com"
                  target="_blank"
                  rel="noreferrer noopener"
                  {...props}
                >
                  <MessageCircleMore class="h-4 w-4" />
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
      {#if remoteConnected}
        <div
          class="flex flex-wrap items-center justify-between gap-3 border-b border-primary/40 bg-primary/10 px-4 py-3 text-sm font-semibold text-primary dark:bg-primary/20 dark:text-primary-foreground md:flex-nowrap md:rounded-t-xl"
          role="status"
          aria-live="polite"
        >
          <div class="flex min-w-0 items-center gap-2">
            <Globe class="h-5 w-5 shrink-0 text-primary dark:text-primary-foreground" />
            <span class="leading-tight">Remote connected</span>
          </div>
          <Button
            size="sm"
            variant="outline"
            class="gap-2"
            onclick={handleRemoteDisconnect}
            disabled={remoteBannerLoading}
            aria-label="Disconnect from remote server"
          >
            {#if remoteBannerLoading}
              <span
                class="h-4 w-4 animate-spin rounded-full border-2 border-current border-t-transparent"
                aria-hidden="true"
              ></span>
            {:else}
              <Unlink class="h-4 w-4 shrink-0" />
            {/if}
            <span class="hidden sm:inline">Disconnect</span>
          </Button>
        </div>
      {/if}
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
