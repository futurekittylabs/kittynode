import { coreClient } from "$lib/client";
import type { Package } from "$lib/types";
import { notifyError } from "$utils/notify";

type CatalogStatus = "idle" | "loading" | "ready" | "error";
type InstalledStatus = "idle" | "loading" | "ready" | "error";

type CatalogState = {
  status: CatalogStatus;
  packages: Record<string, Package>;
  error?: string;
};

type InstalledState = {
  status: InstalledStatus;
  packages: Record<string, Package>;
  error?: string;
};

let catalogState = $state<CatalogState>({
  status: "idle",
  packages: {},
});

let installedState = $state<InstalledState>({
  status: "idle",
  packages: {},
});

let installedRequestToken = 0;

function toPackageRecord(list: Package[]): Record<string, Package> {
  return Object.fromEntries(list.map((pkg) => [pkg.name, pkg]));
}

async function loadCatalog({ force = false }: { force?: boolean } = {}) {
  if (!force && catalogState.status === "loading") return;
  catalogState = {
    status: "loading",
    packages: force ? {} : { ...catalogState.packages },
  };
  try {
    const result = await coreClient.getPackageCatalog();
    catalogState = { status: "ready", packages: result };
  } catch (e) {
    notifyError("Failed to load packages", e);
    catalogState = {
      status: "error",
      packages: {},
      error: "Failed to load packages",
    };
  }
}

async function loadInstalled({ force = false }: { force?: boolean } = {}) {
  const requestId = ++installedRequestToken;
  installedState = {
    status: "loading",
    packages: force ? {} : { ...installedState.packages },
  };
  try {
    const result = await coreClient.getInstalledPackages();
    if (requestId !== installedRequestToken) return;
    installedState = { status: "ready", packages: toPackageRecord(result) };
  } catch (e) {
    if (requestId !== installedRequestToken) return;
    notifyError("Failed to load installed packages", e);
    installedState = {
      status: "error",
      packages: {},
      error: "Failed to load installed packages",
    };
  }
}

export const packagesStore = {
  get packages() {
    return catalogState.packages;
  },

  get catalogState() {
    return catalogState;
  },

  get installedState() {
    return installedState;
  },

  get installedPackages(): Package[] {
    return installedState.status === "ready"
      ? Object.values(installedState.packages)
      : [];
  },

  installationStatus(
    packageName: string | undefined,
  ): "unknown" | "installed" | "available" {
    if (!packageName || installedState.status !== "ready") return "unknown";
    return installedState.packages[packageName] ? "installed" : "available";
  },

  isInstalled(packageName: string | undefined): boolean {
    return !!(packageName && installedState.packages[packageName]);
  },

  loadPackages: loadCatalog,
  loadInstalledPackages: loadInstalled,

  async installPackage(name: string, network?: string) {
    try {
      await coreClient.installPackage(name, network);
      await loadInstalled({ force: true });
    } catch (e) {
      notifyError(`Failed to install ${name}`, e);
      throw e;
    }
  },

  async deletePackage(name: string) {
    try {
      await coreClient.deletePackage(name, false);
      await loadInstalled({ force: true });
    } catch (e) {
      notifyError(`Failed to delete ${name}`, e);
      throw e;
    }
  },

  async stopPackage(name: string) {
    try {
      await coreClient.stopPackage(name);
    } catch (e) {
      notifyError(`Failed to stop ${name}`, e);
      throw e;
    }
  },

  async startPackage(name: string) {
    try {
      await coreClient.startPackage(name);
    } catch (e) {
      notifyError(`Failed to start ${name}`, e);
      throw e;
    }
  },
};
