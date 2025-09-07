<script lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { initializedStore } from "$stores/initialized.svelte";
import { Button } from "$lib/components/ui/button";
import { platform } from "@tauri-apps/plugin-os";
import { remoteAccessStore } from "$stores/remoteAccess.svelte";
import { serverUrlStore } from "$stores/serverUrl.svelte";
import { updates } from "$stores/updates.svelte";
import { LoaderCircle } from "@lucide/svelte";
import { refetchStores } from "$utils/refetchStores";
import { notifySuccess, notifyError, notifyInfo } from "$utils/notify";
import { setMode, userPrefersMode } from "mode-watcher";
import * as Select from "$lib/components/ui/select";

let currentTheme = $state<"light" | "dark" | "system">(userPrefersMode.current);

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
    setRemote("http://merlin:3000");
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
    await initializedStore.uninitialize();
    notifySuccess("Kittynode data deleted", {
      description: "All data has been removed successfully.",
    });
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
  // Refetch store caches
  refetchStores();
}
</script>

<h3 class="scroll-m-20 text-2xl font-semibold tracking-tight mb-4">Settings</h3>

<ul class="settings-list">
  {#if remoteAccessStore.remoteAccess === null}
    <li>Loading remote access status...</li>
  {:else if !remoteAccessStore.remoteAccess}
    <li>
      <span>Enable remote access</span>
      <Button onclick={enableRemoteAccess} disabled={ ["ios", "android"].includes(platform()) }>Enable</Button>
    </li>
    <hr />
  {:else}
    <li>
      <span>Disable remote access</span>
      <Button onclick={disableRemoteAccess}>Disable</Button>
    </li>
    <hr />
  {/if}
  {#if serverUrlStore.serverUrl === ""}
    <li>
      <span>Connect to remote</span>
      <Button onclick={connectRemote}>Connect</Button>
    </li>
    <hr />
  {:else}
    <li>
      <span>Disconnect from remote</span>
      <Button onclick={disconnectRemote}>Disconnect</Button>
    </li>
    <hr />
  {/if}
  {#if !["ios", "android"].includes(platform())}
    <li>
      <span>{updates.hasUpdate ? "Update Kittynode" : "Check for updates"}</span>
      <Button disabled={updates.isProcessing || updates.isChecking} onclick={updates.hasUpdate ? handleUpdate : checkForUpdates}>
        {#if updates.isProcessing}
          <LoaderCircle class="animate-spin" />
          Updating
        {:else if updates.isChecking}
          <LoaderCircle class="animate-spin" />
          Checking
        {:else if !updates.hasUpdate}
          Check for updates
        {:else}
          Update
        {/if}
      </Button>
    </li>
    <hr />
  {/if}
  <li>
    <span>Select theme</span>
    <Select.Root
      type="single"
      bind:value={currentTheme}
      onValueChange={(value) => setMode(value as "light" | "dark" | "system")}
    >
      <Select.Trigger class="w-[180px] capitalize">
        {currentTheme || "Select theme"}
      </Select.Trigger>
      <Select.Content>
        <Select.Item value="light">Light</Select.Item>
        <Select.Item value="dark">Dark</Select.Item>
        <Select.Item value="system">System</Select.Item>
      </Select.Content>
    </Select.Root>
  </li>
  <hr />
  <li>
    <span>Feedback</span>
    <div class="flex gap-2">
      <a href="https://github.com/blackkittylabs/kittynode/discussions/new?category=feedback" target="_blank">
        <Button>GitHub</Button>
      </a>
      <a href="https://discord.kittynode.io" target="_blank">
        <Button>Discord</Button>
      </a>
    </div>
  </li>
  <hr />
  <li>
    <span>Delete all Kittynode data</span>
    <Button onclick={deleteKittynode} disabled={serverUrlStore.serverUrl !== ""} variant="destructive">Delete data</Button>
  </li>
</ul>

<style>
  hr {
    margin: 16px 0px 16px 0px;
  }

  .settings-list li {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }
</style>
