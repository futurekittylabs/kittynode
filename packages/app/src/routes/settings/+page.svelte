<script lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { Button } from "$lib/components/ui/button";
import * as Card from "$lib/components/ui/card";
import { platform } from "@tauri-apps/plugin-os";
import { remoteAccessStore } from "$stores/remoteAccess.svelte";
import { serverUrlStore } from "$stores/serverUrl.svelte";
import { updates } from "$stores/updates.svelte";
import { appConfigStore } from "$stores/appConfig.svelte";
import { onMount } from "svelte";
import { Checkbox } from "$lib/components/ui/checkbox";
import {
  Globe,
  Moon,
  Sun,
  Monitor,
  HardDrive,
  Download,
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

const autoStartDockerEnabled = $derived(appConfigStore.autoStartDocker);
const configInitialized = $derived(appConfigStore.initialized);
const configLoading = $derived(appConfigStore.loading);

onMount(() => {
  void appConfigStore.load().catch((e) => {
    console.error(`Failed to load Kittynode config: ${e}`);
  });
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
    remoteAccessStore.enable();
    notifySuccess("Remote access enabled");
  } catch (e) {
    notifyError("Failed to enable remote access", e);
  }
}

async function disableRemoteAccess() {
  try {
    remoteAccessStore.disable();
    notifySuccess("Remote access disabled");
  } catch (e) {
    notifyError("Failed to disable remote access", e);
  }
}

async function connectRemote() {
  try {
    const current = serverUrlStore.serverUrl || "http://localhost:3000";
    const input = window.prompt("Enter remote server URL", current);
    if (input === null) return; // cancelled
    const url = input.trim();
    if (!url) {
      notifyError("Remote URL cannot be empty");
      return;
    }
    if (!/^https?:\/\//i.test(url)) {
      notifyError("Remote URL must start with http:// or https://");
      return;
    }
    setRemote(url);
    notifySuccess("Connected to remote");
  } catch (e) {
    notifyError("Failed to connect to remote", e);
  }
}

async function disconnectRemote() {
  try {
    setRemote("");
    notifySuccess("Disconnected from remote");
  } catch (e) {
    notifyError("Failed to disconnect from remote", e);
  }
}

async function deleteKittynode() {
  try {
    await invoke("delete_kittynode", { serverUrl: serverUrlStore.serverUrl });
    // Immediately restart the app with fresh config
    await invoke("restart_app");
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
        description: "A new version of Kittynode is ready to install.",
      });
    }
  } catch (e) {
    notifyError("Failed to check for updates", e);
  }
}

function setRemote(serverUrl: string) {
  serverUrlStore.setServerUrl(serverUrl);
  refetchStores();
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
        </div>
        {#if remoteAccessStore.remoteAccess === null}
          <span class="text-sm text-muted-foreground">Loading...</span>
        {:else if !remoteAccessStore.remoteAccess}
          <Button
            size="sm"
            onclick={enableRemoteAccess}
            disabled={["ios", "android"].includes(platform())}
          >
            <Wifi class="h-4 w-4 mr-1" />
            Enable
          </Button>
        {:else}
          <Button
            size="sm"
            variant="outline"
            onclick={disableRemoteAccess}
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
        {#if serverUrlStore.serverUrl === ""}
          <Button
            size="sm"
            variant="outline"
            onclick={connectRemote}
          >
            <Link2 class="h-4 w-4 mr-1" />
            Connect
          </Button>
        {:else}
          <Button
            size="sm"
            variant="outline"
            onclick={disconnectRemote}
          >
            <Unlink class="h-4 w-4 mr-1" />
            Disconnect
          </Button>
        {/if}
      </div>
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
              {updates.hasUpdate ? "A new version is ready to install" : "You're running the latest version"}
            </p>
          </div>
          <div class="ml-auto flex items-center gap-2">
            <Button
              size="sm"
              variant={updates.hasUpdate ? "default" : "outline"}
              disabled={updates.isProcessing || updates.isChecking}
              onclick={updates.hasUpdate ? handleUpdate : checkForUpdates}
            >
              {#if updates.isProcessing}
                <div class="h-4 w-4 mr-1 animate-spin rounded-full border-2 border-current border-t-transparent"></div>
                Updating...
              {:else if updates.isChecking}
                <div class="h-4 w-4 mr-1 animate-spin rounded-full border-2 border-current border-t-transparent"></div>
                Checking...
              {:else if updates.hasUpdate}
                <Download class="h-4 w-4 mr-1" />
                Install Update
              {:else}
                Check Now
              {/if}
            </Button>
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
