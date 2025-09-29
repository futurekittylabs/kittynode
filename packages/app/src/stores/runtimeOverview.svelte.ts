import {
  type RuntimeStatus,
  fetchRuntimeStatuses,
} from "$lib/runtime/packageRuntime.svelte";

type RuntimeStatusMap = Record<string, RuntimeStatus>;

let statuses = $state<RuntimeStatusMap>({});
let loading = $state(false);
let pollHandle: number | null = null;
let pollIntervalMs = 5000;
let activeNames: string[] = [];
let lastKey = "";

function clearPolling() {
  if (pollHandle !== null) {
    window.clearInterval(pollHandle);
    pollHandle = null;
  }
}

async function refresh(names: string[], { force = false } = {}) {
  if (names.length === 0) {
    statuses = {};
    loading = false;
    return;
  }

  if (loading && !force) {
    return;
  }

  loading = true;
  try {
    const result = await fetchRuntimeStatuses(names);
    statuses = buildStatusMap(names, result);
  } catch (error) {
    console.error("Failed to refresh runtime statuses", error);
    statuses = buildStatusMap(names);
  } finally {
    loading = false;
  }
}

function startPolling() {
  if (activeNames.length === 0) {
    return;
  }
  clearPolling();
  pollHandle = window.setInterval(() => {
    void refresh([...activeNames]);
  }, pollIntervalMs);
}

function stop() {
  clearPolling();
  statuses = {};
  activeNames = [];
  lastKey = "";
  loading = false;
}

function buildStatusMap(
  names: string[],
  source?: Partial<RuntimeStatusMap>,
): RuntimeStatusMap {
  const result: RuntimeStatusMap = {};
  for (const name of names) {
    result[name] = source?.[name] ?? "unknown";
  }
  return result;
}

function sync(
  names: string[],
  options: { enabled: boolean; pollInterval?: number },
) {
  if (!options.enabled) {
    stop();
    return;
  }

  const sortedKey = [...names].sort().join("|");
  const intervalChanged =
    typeof options.pollInterval === "number" &&
    options.pollInterval > 0 &&
    options.pollInterval !== pollIntervalMs;

  if (intervalChanged && typeof options.pollInterval === "number") {
    pollIntervalMs = options.pollInterval;
  }

  if (sortedKey !== lastKey) {
    activeNames = [...names];
    lastKey = sortedKey;
    void refresh(activeNames, { force: true });
  } else if (
    statuses &&
    Object.keys(statuses).length === 0 &&
    names.length > 0
  ) {
    activeNames = [...names];
    void refresh(activeNames, { force: true });
  }

  if (intervalChanged || pollHandle === null) {
    startPolling();
  }
}

export const runtimeOverviewStore = {
  get statuses() {
    return statuses;
  },
  get loading() {
    return loading;
  },
  statusOf(name: string): RuntimeStatus {
    return statuses[name] ?? "unknown";
  },
  sync,
  stop,
};
