import { invoke } from "@tauri-apps/api/core";
import { serverUrlStore } from "./serverUrl.svelte";

let initialized = $state(false);

export const initializedStore = {
  get initialized() {
    return initialized;
  },
  async initialize() {
    await invoke("init_kittynode", { serverUrl: serverUrlStore.serverUrl });
    initialized = true;
  },
  async fakeInitialize() {
    initialized = true;
  },
  async uninitialize() {
    initialized = false;
  },
};
