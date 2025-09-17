import { invoke } from "@tauri-apps/api/core";
import { serverUrlStore } from "./serverUrl.svelte";

type InitState = 'idle' | 'initializing' | 'initialized';

let state = $state<InitState>('idle');

export const initializedStore = {
  get state() {
    return state;
  },
  get initialized() {
    return state === 'initialized';
  },
  get initializing() {
    return state === 'initializing';
  },
  async initialize() {
    if (state !== 'idle') {
      return;
    }
    state = 'initializing';
    try {
      await invoke("init_kittynode", { serverUrl: serverUrlStore.serverUrl });
      state = 'initialized';
    } catch (error) {
      state = 'idle';
      throw error;
    }
  },
  async fakeInitialize() {
    if (state !== 'idle') {
      return;
    }
    state = 'initialized';
  },
  async uninitialize() {
    state = 'idle';
  },
};
