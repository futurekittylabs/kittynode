import { coreClient } from "$lib/client";
import type { Package } from "$lib/types";
import type { OperationalState } from "$lib/types/operational_state";
import { operationalStateStore } from "./operationalState.svelte";

type CatalogStatus = "idle" | "loading" | "ready" | "error";
type InstalledStatus = "idle" | "loading" | "ready" | "unavailable" | "error";

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
let lastOperationalSnapshot: { canManage: boolean } | null = null;

function toPackageRecord(list: Package[]): Record<string, Package> {
  return Object.fromEntries(list.map((pkg) => [pkg.name, pkg]));
}

function setInstalledUnavailable() {
  installedRequestToken += 1;
  installedState = {
    status: "unavailable",
    packages: {},
  };
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
    if (installedState.status !== "ready") {
      return [];
    }
    return Object.values(installedState.packages);
  },

  installationStatus(
    packageName: string | undefined,
  ): "unknown" | "installed" | "available" {
    if (!packageName) return "unknown";
    if (installedState.status !== "ready") {
      return "unknown";
    }
    return installedState.packages[packageName] ? "installed" : "available";
  },

  isInstalled(packageName: string | undefined): boolean {
    if (!packageName) return false;
    return Boolean(installedState.packages[packageName]);
  },

  async loadPackages({ force = false }: { force?: boolean } = {}) {
    if (!force && catalogState.status === "loading") {
      return;
    }

    if (!force && catalogState.status === "ready") {
      return;
    }

    const previous = catalogState.packages;
    catalogState = {
      status: "loading",
      packages: force ? {} : { ...previous },
    };

    try {
      const result = await coreClient.getPackageCatalog();
      catalogState = {
        status: "ready",
        packages: result,
      };
    } catch (e) {
      console.error(`Failed to load packages: ${e}`);
      catalogState = {
        status: "error",
        packages: {},
        error: e instanceof Error ? e.message : String(e),
      };
    }
  },

  async loadInstalledPackages({ force = false }: { force?: boolean } = {}) {
    const state = operationalStateStore.state;

    if (!state) {
      return;
    }

    if (!state.canManage) {
      setInstalledUnavailable();
      return;
    }

    if (!force && installedState.status === "loading") {
      return;
    }

    const requestId = ++installedRequestToken;
    const previous = installedState.packages;
    installedState = {
      status: "loading",
      packages: force ? {} : { ...previous },
    };

    try {
      const result = await coreClient.getInstalledPackages();

      if (requestId !== installedRequestToken) {
        return;
      }

      const packages = toPackageRecord(result);
      installedState = {
        status: "ready",
        packages,
      };
    } catch (e) {
      if (requestId !== installedRequestToken) {
        return;
      }

      const message = e instanceof Error ? e.message : String(e);
      console.error(`Failed to load installed packages: ${message}`);
      const fallbackStatus = operationalStateStore.canManage
        ? "error"
        : "unavailable";
      installedState = {
        status: fallbackStatus,
        packages: {},
        error: message,
      };
    }
  },

  async installPackage(name: string, network?: string) {
    try {
      await coreClient.installPackage(name, network);
      await this.loadInstalledPackages({ force: true });
    } catch (e) {
      console.error(`Failed to install ${name}: ${e}`);
      throw e;
    }
  },

  async deletePackage(name: string) {
    try {
      await coreClient.deletePackage(name, false);
      await this.loadInstalledPackages({ force: true });
    } catch (e) {
      console.error(`Failed to delete ${name}: ${e}`);
      throw e;
    }
  },

  async stopPackage(name: string) {
    try {
      await coreClient.stopPackage(name);
    } catch (e) {
      console.error(`Failed to stop ${name}: ${e}`);
      throw e;
    }
  },

  async startPackage(name: string) {
    try {
      await coreClient.startPackage(name);
    } catch (e) {
      console.error(`Failed to start ${name}: ${e}`);
      throw e;
    }
  },

  async getPackage(name: string) {
    try {
      return await coreClient.getPackage(name);
    } catch (e) {
      console.error(`Failed to fetch package state for ${name}: ${e}`);
      throw e;
    }
  },

  async getPackages(names: string[]) {
    try {
      return await coreClient.getPackages(names);
    } catch (e) {
      console.error(`Failed to fetch states for [${names.join(", ")}] : ${e}`);
      throw e;
    }
  },

  handleOperationalStateChange(state: OperationalState | null) {
    if (!state) {
      return;
    }

    const previous = lastOperationalSnapshot;
    lastOperationalSnapshot = { canManage: state.canManage };

    if (state.canManage) {
      if (!previous || !previous.canManage) {
        void this.loadInstalledPackages({ force: true });
      }
      return;
    }

    if (!previous || previous.canManage) {
      setInstalledUnavailable();
    }
  },
};
