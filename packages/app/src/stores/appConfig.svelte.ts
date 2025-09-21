import { invoke } from "@tauri-apps/api/core";
import type { KittynodeConfig } from "$lib/types";
import { serverUrlStore } from "./serverUrl.svelte";

type RawKittynodeConfig = {
  capabilities?: string[];
  server_url?: string;
  onboarding_completed?: boolean;
  auto_start_docker?: boolean;
};

function normalizeConfig(
  raw: RawKittynodeConfig | null | undefined,
): KittynodeConfig {
  return {
    capabilities: raw?.capabilities ?? [],
    serverUrl: raw?.server_url ?? "",
    onboardingCompleted: raw?.onboarding_completed ?? false,
    autoStartDocker: raw?.auto_start_docker ?? false,
  };
}

let config = $state<KittynodeConfig | null>(null);
let loading = $state(false);
let initialized = $state(false);
let loadPromise: Promise<void> | null = null;

async function loadConfig(): Promise<void> {
  loading = true;
  try {
    const raw = await invoke<RawKittynodeConfig>("get_config");
    config = normalizeConfig(raw);
    serverUrlStore.setFromConfig(config.serverUrl);
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
      await invoke("set_auto_start_docker", { enabled });
      if (config) {
        config = { ...config, autoStartDocker: enabled };
      }
    } catch (e) {
      console.error(`Failed to update Docker auto-start preference: ${e}`);
      throw e;
    }
  },
  async setServerUrl(endpoint: string) {
    try {
      await invoke("set_server_url", { endpoint });
      if (config) {
        config = { ...config, serverUrl: endpoint };
      }
      serverUrlStore.setFromConfig(endpoint);
    } catch (e) {
      console.error(`Failed to update server URL: ${e}`);
      throw e;
    }
  },
};
