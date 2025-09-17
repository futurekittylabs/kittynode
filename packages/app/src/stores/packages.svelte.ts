import { invoke } from "@tauri-apps/api/core";
import type { Package } from "$lib/types";
import { dockerStatus } from "./dockerStatus.svelte";
import { serverUrlStore } from "./serverUrl.svelte";

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

let lastDockerRunning: boolean | null = null;

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
      const result = await invoke<Record<string, Package>>("get_packages");
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
    const running = dockerStatus.isRunning;

    if (running === false) {
      setInstalledUnavailable();
      return;
    }

    if (running !== true) {
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
      const result = await invoke<Package[]>("get_installed_packages", {
        serverUrl: serverUrlStore.serverUrl,
      });

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
      installedState = {
        status: dockerStatus.isRunning === true ? "error" : "unavailable",
        packages: {},
        error: message,
      };
    }
  },

  async installPackage(name: string) {
    try {
      await invoke("install_package", {
        name,
        serverUrl: serverUrlStore.serverUrl,
      });
      await this.loadInstalledPackages({ force: true });
    } catch (e) {
      console.error(`Failed to install ${name}: ${e}`);
      throw e;
    }
  },

  async deletePackage(name: string) {
    try {
      await invoke("delete_package", {
        name,
        includeImages: false,
        serverUrl: serverUrlStore.serverUrl,
      });
      await this.loadInstalledPackages({ force: true });
    } catch (e) {
      console.error(`Failed to delete ${name}: ${e}`);
      throw e;
    }
  },

  handleDockerStateChange(running: boolean | null) {
    if (running === lastDockerRunning) {
      return;
    }

    lastDockerRunning = running;

    if (running === true) {
      void this.loadInstalledPackages({ force: true });
      return;
    }

    if (running === false) {
      setInstalledUnavailable();
    }
  },
};
