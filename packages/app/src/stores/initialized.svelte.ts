import { invoke } from "@tauri-apps/api/core";
import { serverUrlStore } from "./serverUrl.svelte";

let initialized = $state(false);
let initializing = $state(false);

export const initializedStore = {
  get initialized() {
    return initialized;
  },
  get initializing() {
    return initializing;
  },
  async initialize() {
    if (initializing || initialized) {
      return;
    }
    initializing = true;
    try {
      await invoke("init_kittynode", { serverUrl: serverUrlStore.serverUrl });
      initialized = true;
    } finally {
      initializing = false;
    }
  },
  async fakeInitialize() {
    if (initializing || initialized) {
      return;
    }
    initialized = true;
  },
  async uninitialize() {
    initialized = false;
    initializing = false;
  },
};
