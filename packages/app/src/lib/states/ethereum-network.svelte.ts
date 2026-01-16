import { coreClient } from "$lib/client";
import { getEthereumNetworkLabel } from "$lib/constants/ethereum-networks";
import type { Package } from "$lib/types";

let label = $state<string | null>(null);
let loading = $state(false);
let error = $state<string | null>(null);
let hasEthereumInstalled = false;
let pendingRefresh = false;
let refreshToken = 0;

function resetRequestToken() {
  refreshToken += 1;
}

async function refreshLabel() {
  if (!hasEthereumInstalled) {
    label = null;
    error = null;
    loading = false;
    return;
  }

  const requestId = ++refreshToken;
  loading = true;
  try {
    const config = await coreClient.getPackageConfig("ethereum");
    if (requestId !== refreshToken) {
      return;
    }
    label = getEthereumNetworkLabel(config.values.network);
    error = null;
  } catch (err) {
    if (requestId !== refreshToken) {
      return;
    }
    label = null;
    error = err instanceof Error ? err.message : String(err);
    console.error(`Failed to load Ethereum network label: ${error}`);
  } finally {
    if (requestId === refreshToken) {
      loading = false;
    }
  }
}

function handleInstalledPackages(packages: Package[] | null) {
  const hasEthereum =
    Array.isArray(packages) && packages.some((pkg) => pkg.name === "ethereum");

  if (!hasEthereum) {
    hasEthereumInstalled = false;
    pendingRefresh = false;
    resetRequestToken();
    label = null;
    error = null;
    loading = false;
    return;
  }

  const shouldRefresh = !hasEthereumInstalled || pendingRefresh;
  hasEthereumInstalled = true;
  pendingRefresh = false;

  if (shouldRefresh) {
    void refreshLabel();
  }
}

function handleConfigUpdated(packageName: string) {
  if (packageName !== "ethereum") {
    return;
  }

  if (!hasEthereumInstalled) {
    pendingRefresh = true;
    return;
  }

  pendingRefresh = false;
  void refreshLabel();
}

export const ethereumNetworkState = {
  get label() {
    return label;
  },
  get loading() {
    return loading;
  },
  get error() {
    return error;
  },
  handleInstalledPackages,
  handleConfigUpdated,
  refresh() {
    return refreshLabel();
  },
};
