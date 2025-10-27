import { packagesStore } from "$states/packages.svelte";

export type RuntimeStatus = "unknown" | "checking" | "running" | "stopped";
export type LifecyclePhase = "idle" | "stopping" | "starting";

const statusCache = new Map<string, RuntimeStatus>();
const pendingRequests = new Map<string, Promise<RuntimeStatus>>();
async function requestStatus(name: string): Promise<RuntimeStatus> {
  if (pendingRequests.has(name)) {
    const pending = pendingRequests.get(name);
    if (pending) {
      return pending;
    }
  }

  const request = (async () => {
    try {
      const result = await packagesStore.getPackages([name]);
      const state = result[name];
      const runtime = state?.runtime;
      const status: RuntimeStatus =
        runtime === "running" ? "running" : runtime ? "stopped" : "unknown";
      statusCache.set(name, status);
      return status;
    } catch (error) {
      statusCache.set(name, "unknown");
      throw error;
    } finally {
      pendingRequests.delete(name);
    }
  })();

  pendingRequests.set(name, request);
  return request;
}

export async function fetchRuntimeStatuses(
  packageNames: string[],
): Promise<Record<string, RuntimeStatus>> {
  if (packageNames.length === 0) {
    return {};
  }

  const runtimeMap = await packagesStore.getPackages(packageNames);
  const result: Record<string, RuntimeStatus> = {};

  for (const name of packageNames) {
    const state = runtimeMap[name];
    const runtime = state?.runtime;
    const status: RuntimeStatus =
      runtime === "running" ? "running" : runtime ? "stopped" : "unknown";
    statusCache.set(name, status);
    result[name] = status;
  }

  return result;
}

export function getCachedRuntimeStatus(
  name: string,
): RuntimeStatus | undefined {
  return statusCache.get(name);
}

interface RefreshOptions {
  force?: boolean;
  withSpinner?: boolean;
}

interface AttachOptions {
  name: string | null;
  enabled: boolean;
  pollInterval?: number;
}

export function createPackageRuntimeController(initialInterval = 5000) {
  let target: string | null = null;
  let enabled = false;
  let pollHandle: number | null = null;
  let pollInterval = initialInterval;
  let inFlight: Promise<RuntimeStatus> | null = null;
  let destroyed = false;

  let status = $state<RuntimeStatus>("unknown");
  let lifecycle = $state<LifecyclePhase>("idle");
  let loading = $state(false);

  function clearPolling() {
    if (pollHandle !== null) {
      window.clearInterval(pollHandle);
      pollHandle = null;
    }
  }

  function startPolling() {
    if (!target || !enabled || lifecycle !== "idle" || destroyed) {
      return;
    }
    clearPolling();
    pollHandle = window.setInterval(() => {
      void refresh();
    }, pollInterval);
  }

  async function refresh(options: RefreshOptions = {}) {
    const { force = false, withSpinner = false } = options;

    if (!target || !enabled || destroyed) {
      status = "unknown";
      loading = false;
      return;
    }

    if (inFlight) {
      if (!force) {
        return inFlight;
      }

      try {
        await inFlight;
      } catch (error) {
        console.error(`Runtime refresh failed for ${target}:`, error);
      }
    }

    if (withSpinner) {
      loading = true;
      status = "checking";
    }

    const request = requestStatus(target)
      .then((result) => {
        if (target) {
          status = result;
        }
        return result;
      })
      .catch((error) => {
        status = "unknown";
        throw error;
      })
      .finally(() => {
        if (withSpinner) {
          loading = false;
        }
        if (inFlight === request) {
          inFlight = null;
        }
      });

    inFlight = request;
    return request;
  }

  function resetState() {
    status = "unknown";
    loading = false;
    lifecycle = "idle";
  }

  function attach({
    name,
    enabled: nextEnabled,
    pollInterval: nextInterval,
  }: AttachOptions) {
    if (destroyed) {
      return;
    }

    const nameChanged = target !== name;
    const enabledChanged = enabled !== nextEnabled;
    const intervalChanged =
      typeof nextInterval === "number" &&
      nextInterval > 0 &&
      nextInterval !== pollInterval;

    if (!name && !nextEnabled) {
      clearPolling();
      target = null;
      enabled = false;
      resetState();
      return;
    }

    target = name;
    enabled = nextEnabled;
    if (intervalChanged && typeof nextInterval === "number") {
      pollInterval = nextInterval;
    }

    if (!target || !enabled) {
      clearPolling();
      resetState();
      return;
    }

    const cachedStatus = statusCache.get(target);
    if (cachedStatus) {
      status = cachedStatus;
    }

    if (nameChanged || enabledChanged) {
      clearPolling();
      loading = true;
      status = "checking";
      void refresh({ force: true, withSpinner: true })
        .catch((error) => {
          console.error(`Failed to refresh runtime state for ${name}:`, error);
        })
        .finally(() => {
          if (!destroyed) {
            startPolling();
          }
        });
      return;
    }

    if (intervalChanged) {
      startPolling();
    } else if (pollHandle === null) {
      startPolling();
    }
  }

  async function performLifecycle(
    phase: Exclude<LifecyclePhase, "idle">,
    action: () => Promise<void>,
  ) {
    if (!target || lifecycle !== "idle" || destroyed) {
      return false;
    }

    lifecycle = phase;
    clearPolling();

    try {
      await action();
      await refresh({ force: true, withSpinner: true }).catch((error) => {
        console.error(`Failed to refresh runtime state for ${target}:`, error);
      });
      return true;
    } finally {
      lifecycle = "idle";
      if (!destroyed) {
        startPolling();
      }
    }
  }

  function stop() {
    destroyed = true;
    clearPolling();
    resetState();
  }

  return {
    get status() {
      return status;
    },
    get lifecycle() {
      return lifecycle;
    },
    get loading() {
      return loading;
    },
    attach,
    refresh,
    performLifecycle,
    stop,
  };
}
