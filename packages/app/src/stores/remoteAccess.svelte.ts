import { coreClient } from "$lib/client";
import type { WebServiceStatus, WebServiceState } from "$lib/types";

let status = $state<WebServiceStatus | null>(null);
let loading = $state(false);
let lastError = $state<string | null>(null);
let pollHandle: number | null = null;

function isRunning(state: WebServiceState | null) {
  return state === "started" || state === "already_running";
}

function applyStatus(next: WebServiceStatus) {
  status = next;
  lastError = null;
}

function recordError(error: unknown) {
  const message = error instanceof Error ? error.message : String(error);
  lastError = message;
  console.error(`Remote access error: ${message}`);
}

async function refresh(): Promise<WebServiceStatus | null> {
  loading = true;
  try {
    const result = await coreClient.getWebServiceStatus();
    applyStatus(result);
    return result;
  } catch (error) {
    recordError(error);
    return null;
  } finally {
    loading = false;
  }
}

async function enable(port?: number): Promise<WebServiceStatus> {
  loading = true;
  try {
    const result = await coreClient.startWebService(port);
    applyStatus(result);
    return result;
  } catch (error) {
    recordError(error);
    throw error;
  } finally {
    loading = false;
  }
}

async function disable(): Promise<WebServiceStatus> {
  loading = true;
  try {
    const result = await coreClient.stopWebService();
    applyStatus(result);
    return result;
  } catch (error) {
    recordError(error);
    throw error;
  } finally {
    loading = false;
  }
}

function startPolling(interval = DEFAULT_POLL_INTERVAL_MS) {
  if (typeof window === "undefined") {
    return;
  }
  stopPolling();
  void refresh();
  pollHandle = window.setInterval(() => {
    void refresh();
  }, interval);
}

function stopPolling() {
  if (pollHandle !== null && typeof window !== "undefined") {
    window.clearInterval(pollHandle);
    pollHandle = null;
  }
}

export const remoteAccessStore = {
  get remoteAccess() {
    return isRunning(status?.status ?? null);
  },
  get status() {
    return status;
  },
  get port() {
    return status?.port ?? null;
  },
  get loading() {
    return loading;
  },
  get lastError() {
    return lastError;
  },
  async refresh() {
    return refresh();
  },
  async enable(port?: number) {
    return enable(port);
  },
  async disable() {
    return disable();
  },
  startPolling,
  stopPolling,
};

const DEFAULT_POLL_INTERVAL_MS = 5_000;
