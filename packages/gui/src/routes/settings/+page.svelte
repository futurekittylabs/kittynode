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
import { error } from "$utils/error";
import { setMode, userPrefersMode } from "mode-watcher";
import * as Select from "$lib/components/ui/select";

let currentTheme = $state<"light" | "dark" | "system">($userPrefersMode);

async function enableRemoteAccess() {
  try {
    remoteAccessStore.enable();
    alert("Remote access has been enabled.");
  } catch (e) {
    error(`Failed to enable remote access: ${e}`);
  }
}

async function disableRemoteAccess() {
  try {
    remoteAccessStore.disable();
    alert("Remote access has been disabled.");
  } catch (e) {
    error(`Failed to disable remote access: ${e}`);
  }
}

async function connectRemote() {
  try {
    setRemote("http://merlin:3000");
    alert("Connected to remote.");
  } catch (e) {
    error(`Failed to connect to remote: ${e}`);
  }
}

async function disconnectRemote() {
  try {
    setRemote("");
    alert("Disconnected from remote.");
  } catch (e) {
    error(`Failed to disconnect from remote: ${e}`);
  }
}

async function deleteKittynode() {
  try {
    await invoke("delete_kittynode", { serverUrl: serverUrlStore.serverUrl });
    await initializedStore.uninitialize();
    console.info("Kittynode data has been deleted successfully.");
  } catch (e) {
    error(`Failed to delete Kittynode: ${e}`);
  }
}

async function handleUpdate() {
  await updates.installUpdate();
}

async function checkForUpdates() {
  await updates.getUpdate();
  if (!updates.hasUpdate) {
    alert("No update available, you're up to date!");
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
      <Button disabled={updates.isProcessing} onclick={updates.hasUpdate ? handleUpdate : checkForUpdates}>
        {#if updates.isProcessing}
          <LoaderCircle class="animate-spin" />
          Updating
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
      onValueChange={(value: "light" | "dark" | "system") => setMode(value)}
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
