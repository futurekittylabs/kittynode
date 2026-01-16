import { coreClient } from "$lib/client";
import type { KittynodeConfig } from "$lib/types";
import { normalizeServerUrl, serverUrlState } from "./server-url.svelte";

let config = $state<KittynodeConfig | null>(null);
let loading = $state(false);
let initialized = $state(false);
let loadPromise: Promise<void> | null = null;

async function loadConfig(): Promise<void> {
  loading = true;
  try {
    config = await coreClient.getConfig();
    serverUrlState.setFromConfig(config.serverUrl, config.lastServerUrl);
    initialized = true;
  } catch (e) {
    console.error(`Failed to load Kittynode config: ${e}`);
    throw e;
  } finally {
    loading = false;
    loadPromise = null;
  }
}

export const appConfigState = {
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
  get showTrayIcon() {
    return config?.showTrayIcon ?? true;
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
  async setShowTrayIcon(enabled: boolean) {
    try {
      await coreClient.setShowTrayIcon(enabled);
      if (config) {
        config = { ...config, showTrayIcon: enabled };
      }
    } catch (e) {
      console.error(`Failed to update tray icon preference: ${e}`);
      throw e;
    }
  },
  async setServerUrl(endpoint: string) {
    const normalizedEndpoint = normalizeServerUrl(endpoint);
    const previousLast = normalizeServerUrl(
      config?.lastServerUrl ?? serverUrlState.lastServerUrl ?? ""
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

      serverUrlState.setFromConfig(normalizedEndpoint, nextLast);
    } catch (e) {
      console.error(`Failed to update server URL: ${e}`);
      serverUrlState.setFromConfig(
        config?.serverUrl ?? serverUrlState.serverUrl,
        config?.lastServerUrl ?? previousLast
      );
      throw e;
    }
  },
};
