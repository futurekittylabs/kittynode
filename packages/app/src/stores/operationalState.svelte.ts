import { coreClient } from "$lib/client";
import type {
  OperationalMode,
  OperationalState,
} from "$lib/types/operational_state";

let state = $state<OperationalState | null>(null);
let loading = $state(false);
let error = $state<string | null>(null);
let isStarting = $state(false);
let pollHandle: number | null = $state(null);
let startingTimeout: number | null = $state(null);

async function refresh() {
  loading = true;
  try {
    state = await coreClient.getOperationalState();
    error = null;
    if (state.dockerRunning) {
      isStarting = false;
      clearStartingTimeout();
    }
  } catch (err) {
    const message = err instanceof Error ? err.message : String(err);
    console.error(`Failed to refresh operational state: ${message}`);
    error = message;
  } finally {
    loading = false;
  }
}

function startPolling(intervalMs = DEFAULT_POLL_INTERVAL) {
  void refresh();
  if (pollHandle !== null) {
    window.clearInterval(pollHandle);
  }
  pollHandle = window.setInterval(() => {
    void refresh();
  }, intervalMs);
}

function stopPolling() {
  if (pollHandle !== null) {
    window.clearInterval(pollHandle);
    pollHandle = null;
  }
  clearStartingTimeout();
}

async function startDockerIfNeeded() {
  try {
    const status = await coreClient.startDockerIfNeeded();
    if (status === "starting") {
      isStarting = true;
      startPolling(STARTING_POLL_INTERVAL);
      scheduleStartingTimeout();
    }
    await refresh();
    return { status };
  } catch (err) {
    const message = err instanceof Error ? err.message : String(err);
    console.error(`Failed to start Docker: ${message}`);
    await refresh();
    return {
      status: "error",
      error: message,
    } as const;
  }
}

function scheduleStartingTimeout(duration = STARTING_TIMEOUT_MS) {
  clearStartingTimeout();
  startingTimeout = window.setTimeout(() => {
    if (isStarting) {
      console.info("Docker startup timeout reached, clearing starting state");
      isStarting = false;
      startPolling(DEFAULT_POLL_INTERVAL);
    }
  }, duration);
}

function clearStartingTimeout() {
  if (startingTimeout !== null) {
    window.clearTimeout(startingTimeout);
    startingTimeout = null;
  }
}

export const operationalStateStore = {
  get state() {
    return state;
  },
  get loading() {
    return loading;
  },
  get error() {
    return error;
  },
  get isStarting() {
    return isStarting;
  },
  get mode(): OperationalMode | null {
    return state?.mode ?? null;
  },
  get dockerRunning(): boolean | null {
    if (!state) return null;
    return state.dockerRunning;
  },
  get canInstall(): boolean {
    return state?.canInstall ?? false;
  },
  get canManage(): boolean {
    return state?.canManage ?? false;
  },
  async refresh() {
    await refresh();
  },
  startPolling,
  stopPolling,
  async startDockerIfNeeded() {
    return startDockerIfNeeded();
  },
};
const DEFAULT_POLL_INTERVAL = 5000;
const STARTING_POLL_INTERVAL = 2000;
const STARTING_TIMEOUT_MS = 30_000;
