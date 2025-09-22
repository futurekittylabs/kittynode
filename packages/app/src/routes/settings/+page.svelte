<script lang="ts">
import { coreClient } from "$lib/client";
import { Button } from "$lib/components/ui/button";
import * as Card from "$lib/components/ui/card";
import { platform } from "@tauri-apps/plugin-os";
import { remoteAccessStore } from "$stores/remoteAccess.svelte";
import { serverUrlStore } from "$stores/serverUrl.svelte";
import { operationalStateStore } from "$stores/operationalState.svelte";
import { updates } from "$stores/updates.svelte";
import { appConfigStore } from "$stores/appConfig.svelte";
import { onMount } from "svelte";
import { Checkbox } from "$lib/components/ui/checkbox";
import { Input } from "$lib/components/ui/input";
import * as Dialog from "$lib/components/ui/dialog";
import {
  Globe,
  Moon,
  Sun,
  Monitor,
  HardDrive,
  Download,
  ArrowUpRight,
  MessageSquare,
  Trash2,
  Wifi,
  WifiOff,
  Link2,
  Unlink,
} from "@lucide/svelte";
import { refetchStores } from "$utils/refetchStores";
import { notifySuccess, notifyError, notifyInfo } from "$utils/notify";
import { setMode, userPrefersMode } from "mode-watcher";
import * as Select from "$lib/components/ui/select";

let currentTheme = $state<"light" | "dark" | "system">(userPrefersMode.current);
let updatingAutoStartDocker = $state(false);
let remoteServerDialogOpen = $state(false);
let remoteServerUrlInput = $state("");
let remoteServerError = $state("");
let remoteDialogLoading = $state(false);
let remoteDialogAction = $state<"connect" | "disconnect" | null>(null);

const autoStartDockerEnabled = $derived(appConfigStore.autoStartDocker);
const configInitialized = $derived(appConfigStore.initialized);
const configLoading = $derived(appConfigStore.loading);
const downloadsUrl = "https://kittynode.com/download";
const remoteServerConnected = $derived(serverUrlStore.serverUrl !== "");

onMount(() => {
  void appConfigStore.load().catch((e) => {
    console.error(`Failed to load Kittynode config: ${e}`);
  });
  void remoteAccessStore.refresh();
  remoteAccessStore.startPolling();
  return () => {
    remoteAccessStore.stopPolling();
  };
});

async function handleAutoStartDockerChange(enabled: boolean) {
  if (!configInitialized) {
    return;
  }

  if (enabled === autoStartDockerEnabled) {
    return;
  }

  if (updatingAutoStartDocker) {
    return;
  }

  updatingAutoStartDocker = true;
  try {
    await appConfigStore.setAutoStartDocker(enabled);
    notifySuccess(
      enabled ? "Docker auto-start enabled" : "Docker auto-start disabled",
    );
  } catch (e) {
    notifyError("Failed to update Docker auto-start preference", e);
    try {
      await appConfigStore.reload();
    } catch (reloadError) {
      console.error(`Failed to reload Kittynode config: ${reloadError}`);
    }
  } finally {
    updatingAutoStartDocker = false;
  }
}

async function enableRemoteAccess() {
  try {
    const status = await remoteAccessStore.enable();
    const portDescription = status.port ?? remoteAccessStore.port;
    notifySuccess("Remote access enabled", {
      description:
        portDescription !== null
          ? `Listening on http://localhost:${portDescription}`
          : undefined,
    });
  } catch (e) {
    notifyError("Failed to enable remote access", e);
  }
}

async function disableRemoteAccess() {
  try {
    await remoteAccessStore.disable();
    notifySuccess("Remote access disabled");
  } catch (e) {
    notifyError("Failed to disable remote access", e);
  }
}

function validateRemoteUrl(url: string) {
  const trimmed = url.trim();
  if (!trimmed) {
    return "Remote URL cannot be empty";
  }
  if (!/^https?:\/\//i.test(trimmed)) {
    return "Remote URL must start with http:// or https://";
  }
  return null;
}

function openRemoteDialog() {
  remoteServerUrlInput = serverUrlStore.serverUrl || "http://localhost:3000";
  remoteServerError = "";
  remoteDialogAction = null;
  remoteServerDialogOpen = true;
}

async function applyRemoteConnection(url: string) {
  try {
    await appConfigStore.setServerUrl(url);
    await operationalStateStore.refresh();
    refetchStores();
    notifySuccess("Connected to remote");
    return true;
  } catch (e) {
    notifyError("Failed to connect to remote", e);
    return false;
  }
}

async function clearRemoteConnection() {
  try {
    await appConfigStore.setServerUrl("");
    await operationalStateStore.refresh();
    refetchStores();
    notifySuccess("Disconnected from remote");
    return true;
  } catch (e) {
    notifyError("Failed to disconnect from remote", e);
    return false;
  }
}

async function submitRemoteDialog() {
  const validationError = validateRemoteUrl(remoteServerUrlInput);
  if (validationError) {
    remoteServerError = validationError;
    notifyError(validationError);
    return;
  }

  remoteServerError = "";
  const url = remoteServerUrlInput.trim();
  remoteServerUrlInput = url;
  remoteDialogAction = "connect";
  remoteDialogLoading = true;
  try {
    const success = await applyRemoteConnection(url);
    if (success) {
      remoteServerDialogOpen = false;
    }
  } finally {
    remoteDialogLoading = false;
    remoteDialogAction = null;
  }
}

async function disconnectRemoteFromDialog() {
  remoteServerError = "";
  remoteDialogAction = "disconnect";
  remoteDialogLoading = true;
  try {
    const success = await clearRemoteConnection();
    if (success) {
      remoteServerDialogOpen = false;
    }
  } finally {
    remoteDialogLoading = false;
    remoteDialogAction = null;
  }
}

async function deleteKittynode() {
  try {
    await coreClient.deleteKittynode();
    // Immediately restart the app with fresh config
    await coreClient.restartApp();
  } catch (e) {
    notifyError("Failed to delete Kittynode data", e);
  }
}

async function handleUpdate() {
  notifyInfo("Installing update...", {
    description: "Kittynode will restart when the update is complete.",
  });
  await updates.installUpdate();
}

async function checkForUpdates() {
  try {
    await updates.getUpdate(true);
    if (!updates.hasUpdate) {
      notifySuccess("You're up to date!", {
        description: "No updates available at this time.",
      });
    } else {
      notifyInfo("Update available!", {
        description: updates.requiresManualInstall
          ? "Download the latest version from kittynode.com/download."
          : "A new version of Kittynode is ready to install.",
      });
    }
  } catch (e) {
    notifyError("Failed to check for updates", e);
  }
}
</script>

<div class="space-y-6">
  <div>
    <h2 class="text-3xl font-bold tracking-tight">Settings</h2>
    <p class="text-muted-foreground">
      Manage your Kittynode preferences and configuration
    </p>
  </div>

  <!-- Network Settings -->
  <Card.Root>
    <Card.Header>
      <Card.Title class="flex items-center gap-2">
        <Globe class="h-5 w-5" />
        Network
      </Card.Title>
      <Card.Description>
        Configure remote access and connections
      </Card.Description>
    </Card.Header>
    <Card.Content class="space-y-4">
      <!-- Remote Access -->
      <div class="flex items-center justify-between">
        <div>
          <p class="text-sm font-medium">Remote Access</p>
          <p class="text-xs text-muted-foreground">
            Allow external connections to this node
          </p>
          {#if remoteAccessStore.remoteAccess && remoteAccessStore.port !== null}
            <p class="mt-1 text-xs text-muted-foreground">
              Access via
              <span class="font-mono">
                http://localhost:{remoteAccessStore.port}
              </span>
            </p>
          {/if}
          {#if remoteAccessStore.lastError}
            <p class="mt-1 text-xs text-destructive">
              {remoteAccessStore.lastError}
            </p>
          {/if}
        </div>
        {#if remoteAccessStore.status === null}
          <span class="text-sm text-muted-foreground">Loading...</span>
        {:else if !remoteAccessStore.remoteAccess}
          <Button
            size="sm"
            onclick={enableRemoteAccess}
            disabled={
              remoteAccessStore.loading || ["ios", "android"].includes(platform())
            }
          >
            <Wifi class="h-4 w-4 mr-1" />
            Enable
          </Button>
        {:else}
          <Button
            size="sm"
            variant="outline"
            onclick={disableRemoteAccess}
            disabled={remoteAccessStore.loading}
          >
            <WifiOff class="h-4 w-4 mr-1" />
            Disable
          </Button>
        {/if}
      </div>

      <div class="border-t pt-4"></div>

      <!-- Remote Connection -->
      <div class="flex items-center justify-between">
        <div>
          <p class="text-sm font-medium">Remote Server</p>
          <p class="text-xs text-muted-foreground">
            {serverUrlStore.serverUrl || "Not connected"}
          </p>
        </div>
        <Button
          size="sm"
          variant="outline"
          onclick={openRemoteDialog}
        >
          <Link2 class="h-4 w-4 mr-1" />
          {remoteServerConnected ? "Manage" : "Connect"}
        </Button>
      </div>
      <Dialog.Root bind:open={remoteServerDialogOpen}>
        <Dialog.Content>
          <Dialog.Header>
            <Dialog.Title>
              {remoteServerConnected ? "Manage remote connection" : "Connect to remote server"}
            </Dialog.Title>
            <Dialog.Description>
              Enter the server URL you want Kittynode to use when operating remotely.
            </Dialog.Description>
          </Dialog.Header>
          <div class="space-y-4">
            <div class="space-y-2">
              <label class="block text-sm font-medium" for="remote-server-url">
                Server URL
              </label>
              <Input
                id="remote-server-url"
                type="url"
                bind:value={remoteServerUrlInput}
                placeholder="https://example.com"
                aria-invalid={remoteServerError ? "true" : undefined}
                disabled={remoteDialogLoading}
              />
              {#if remoteServerError}
                <p class="text-xs text-destructive">{remoteServerError}</p>
              {/if}
            </div>
          </div>
          <Dialog.Footer>
            <Button
              type="button"
              variant="ghost"
              onclick={() => (remoteServerDialogOpen = false)}
              disabled={remoteDialogLoading}
            >
              Cancel
            </Button>
            {#if remoteServerConnected}
              <Button
                type="button"
                variant="destructive"
                onclick={disconnectRemoteFromDialog}
                disabled={remoteDialogLoading}
                class="gap-2"
              >
                {#if remoteDialogLoading && remoteDialogAction === "disconnect"}
                  <div class="h-4 w-4 animate-spin rounded-full border-2 border-current border-t-transparent"></div>
                {/if}
                <Unlink class="h-4 w-4" />
                Disconnect
              </Button>
            {/if}
            <Button
              type="button"
              onclick={submitRemoteDialog}
              disabled={remoteDialogLoading}
              class="gap-2"
            >
              {#if remoteDialogLoading && remoteDialogAction === "connect"}
                <div class="h-4 w-4 animate-spin rounded-full border-2 border-current border-t-transparent"></div>
              {:else}
                <Link2 class="h-4 w-4" />
              {/if}
              {remoteServerConnected ? "Update connection" : "Connect"}
            </Button>
          </Dialog.Footer>
        </Dialog.Content>
      </Dialog.Root>
    </Card.Content>
  </Card.Root>

  <!-- Docker -->
  <Card.Root>
    <Card.Header>
      <Card.Title class="flex items-center gap-2">
        <HardDrive class="h-5 w-5" />
        Docker
      </Card.Title>
      <Card.Description>
        Control how Kittynode interacts with Docker Desktop
      </Card.Description>
    </Card.Header>
    <Card.Content>
      <div class="flex items-center justify-between">
        <div>
          <p class="text-sm font-medium">Auto-start Docker</p>
          <p class="text-xs text-muted-foreground">
            Start Docker Desktop when Kittynode launches
          </p>
        </div>
        {#if configLoading && !configInitialized}
          <span class="text-sm text-muted-foreground">Loading...</span>
        {:else if !configInitialized}
          <span class="text-sm text-destructive">Failed to load</span>
        {:else}
          <label class="flex items-center gap-2">
            <Checkbox
              id="auto-start-docker"
              checked={autoStartDockerEnabled}
              onCheckedChange={handleAutoStartDockerChange}
              disabled={!configInitialized || updatingAutoStartDocker}
            />
            <span class="text-sm text-muted-foreground">
              {autoStartDockerEnabled ? "Enabled" : "Disabled"}
            </span>
          </label>
        {/if}
      </div>
      <p class="mt-3 text-xs text-muted-foreground">
        Enabling this may prompt for your system password on Linux the next time Kittynode starts.
      </p>
    </Card.Content>
  </Card.Root>

  <!-- Appearance -->
  <Card.Root>
    <Card.Header>
      <Card.Title class="flex items-center gap-2">
        <Sun class="h-5 w-5" />
        Appearance
      </Card.Title>
      <Card.Description>
        Customize how Kittynode looks
      </Card.Description>
    </Card.Header>
    <Card.Content>
      <div class="flex items-center justify-between">
        <div>
          <p class="text-sm font-medium">Theme</p>
          <p class="text-xs text-muted-foreground">
            Select your preferred color scheme
          </p>
        </div>
        <Select.Root
          type="single"
          bind:value={currentTheme}
          onValueChange={(value) => setMode(value as "light" | "dark" | "system")}
        >
          <Select.Trigger class="w-[140px]">
            <div class="flex items-center gap-2">
              {#if currentTheme === "light"}
                <Sun class="h-4 w-4" />
              {:else if currentTheme === "dark"}
                <Moon class="h-4 w-4" />
              {:else}
                <Monitor class="h-4 w-4" />
              {/if}
              <span class="capitalize">{currentTheme || "System"}</span>
            </div>
          </Select.Trigger>
          <Select.Content>
            <Select.Item value="light">
              <div class="flex items-center gap-2">
                <Sun class="h-4 w-4" />
                Light
              </div>
            </Select.Item>
            <Select.Item value="dark">
              <div class="flex items-center gap-2">
                <Moon class="h-4 w-4" />
                Dark
              </div>
            </Select.Item>
            <Select.Item value="system">
              <div class="flex items-center gap-2">
                <Monitor class="h-4 w-4" />
                System
              </div>
            </Select.Item>
          </Select.Content>
        </Select.Root>
      </div>
    </Card.Content>
  </Card.Root>

  {#if !["ios", "android"].includes(platform())}
    <!-- Updates -->
    <Card.Root>
      <Card.Header>
        <Card.Title class="flex items-center gap-2">
          <Download class="h-5 w-5" />
          Updates
        </Card.Title>
        <Card.Description>
          Keep Kittynode up to date with the latest features
        </Card.Description>
      </Card.Header>
      <Card.Content>
        <div class="flex items-center">
          <div>
            <p class="text-sm font-medium">
              {updates.hasUpdate ? "Update Available" : "Check for Updates"}
            </p>
            <p class="text-xs text-muted-foreground">
              {#if updates.hasUpdate}
                {#if updates.requiresManualInstall}
                  A new version is available! Download it from
                  <a
                    href={downloadsUrl}
                    target="_blank"
                    rel="noreferrer noopener"
                    class="link"
                  >
                    kittynode.com/download
                  </a>. ✨
                {:else}
                  A new version is ready to install
                {/if}
              {:else}
                You're running the latest version
              {/if}
            </p>
          </div>
          <div class="ml-auto flex items-center gap-2">
            {#if updates.requiresManualInstall}
              <Button
                size="sm"
                variant="default"
                href={downloadsUrl}
                target="_blank"
                rel="noreferrer noopener"
                disabled={updates.isChecking}
                class="gap-2"
              >
                {#if updates.isChecking}
                  <div class="h-4 w-4 mr-1 animate-spin rounded-full border-2 border-current border-t-transparent"></div>
                  Checking...
                {:else}
                  Open Downloads
                  <ArrowUpRight class="h-4 w-4" />
                {/if}
              </Button>
            {:else if updates.hasUpdate}
              <Button
                size="sm"
                variant="default"
                onclick={handleUpdate}
                disabled={updates.isProcessing || updates.isChecking}
              >
                {#if updates.isProcessing}
                  <div class="h-4 w-4 mr-1 animate-spin rounded-full border-2 border-current border-t-transparent"></div>
                  Updating...
                {:else if updates.isChecking}
                  <div class="h-4 w-4 mr-1 animate-spin rounded-full border-2 border-current border-t-transparent"></div>
                  Checking...
                {:else}
                  <Download class="h-4 w-4 mr-1" />
                  Install Update
                {/if}
              </Button>
            {:else}
              <Button
                size="sm"
                variant="outline"
                onclick={checkForUpdates}
                disabled={updates.isChecking}
              >
                {#if updates.isChecking}
                  <div class="h-4 w-4 mr-1 animate-spin rounded-full border-2 border-current border-t-transparent"></div>
                  Checking...
                {:else}
                  Check Now
                {/if}
              </Button>
            {/if}
          </div>
        </div>
      </Card.Content>
    </Card.Root>
  {/if}



  <!-- Danger Zone -->
  <Card.Root class="border-destructive/50">
    <Card.Header>
      <Card.Title class="flex items-center gap-2 text-destructive">
        <Trash2 class="h-5 w-5" />
        Danger Zone
      </Card.Title>
      <Card.Description>
        Irreversible actions that affect your Kittynode data
      </Card.Description>
    </Card.Header>
    <Card.Content>
      <div class="flex items-center justify-between">
        <div>
          <p class="text-sm font-medium">Delete All Data</p>
          <p class="text-xs text-muted-foreground">
            Permanently remove all Kittynode data and settings
          </p>
        </div>
        <Button
          size="sm"
          onclick={deleteKittynode}
          disabled={serverUrlStore.serverUrl !== ""}
          variant="destructive"
        >
          <Trash2 class="h-4 w-4 mr-1" />
          Delete Data
        </Button>
      </div>
    </Card.Content>
  </Card.Root>
</div>
