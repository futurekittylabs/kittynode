import { coreClient } from "$lib/client";
import type { KittynodeConfig } from "$lib/types";
import { serverUrlStore, normalizeServerUrl } from "./serverUrl.svelte";

let config = $state<KittynodeConfig | null>(null);
let loading = $state(false);
let initialized = $state(false);
let loadPromise: Promise<void> | null = null;

async function loadConfig(): Promise<void> {
  loading = true;
  try {
    config = await coreClient.getConfig();
    serverUrlStore.setFromConfig(config.serverUrl, config.lastServerUrl);
    initialized = true;
  } catch (e) {
    console.error(`Failed to load Kittynode config: ${e}`);
    throw e;
  } finally {
    loading = false;
    loadPromise = null;
  }
}

export const appConfigStore = {
  get config() {
    return config;
  },
  get loading() {
    return loading;
  },
  get initialized() {
    return initialized;
  },
  get autoStartDocker() {
    return config?.autoStartDocker ?? false;
  },
  async load() {
    if (initialized) {
      return;
    }

    if (!loadPromise) {
      loadPromise = loadConfig();
    }

    return loadPromise;
  },
  async reload() {
    if (loadPromise) {
      await loadPromise;
    }
    loadPromise = loadConfig();
    return loadPromise;
  },
  async setAutoStartDocker(enabled: boolean) {
    try {
      await coreClient.setAutoStartDocker(enabled);
      if (config) {
        config = { ...config, autoStartDocker: enabled };
      }
    } catch (e) {
      console.error(`Failed to update Docker auto-start preference: ${e}`);
      throw e;
    }
  },
  async setServerUrl(endpoint: string) {
    const normalizedEndpoint = normalizeServerUrl(endpoint);
    const previousLast = normalizeServerUrl(
      config?.lastServerUrl ?? serverUrlStore.lastServerUrl ?? "",
    );

    try {
      await coreClient.setServerUrl(normalizedEndpoint);
      const nextLast =
        normalizedEndpoint !== "" ? normalizedEndpoint : previousLast;

      if (config) {
        config = {
          ...config,
          serverUrl: normalizedEndpoint,
          lastServerUrl: nextLast,
          hasRemoteServer: normalizedEndpoint !== "",
        };
      }

      serverUrlStore.setFromConfig(normalizedEndpoint, nextLast);
    } catch (e) {
      console.error(`Failed to update server URL: ${e}`);
      serverUrlStore.setFromConfig(
        config?.serverUrl ?? serverUrlStore.serverUrl,
        config?.lastServerUrl ?? previousLast,
      );
      throw e;
    }
  },
};
