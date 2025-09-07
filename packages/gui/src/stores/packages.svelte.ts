import { invoke } from "@tauri-apps/api/core";
import type { Package } from "$lib/types";
import { serverUrlStore } from "./serverUrl.svelte";

let packages = $state<{ [name: string]: Package }>({});
let installedPackages = $state<Package[]>([]);
let isLoading = $state(false);

export const packagesStore = {
  get packages() {
    return packages;
  },

  get installedPackages() {
    return installedPackages;
  },

  get isLoading() {
    return isLoading;
  },

  isInstalled(packageName: string | undefined): boolean {
    if (!packageName) return false;
    return installedPackages.some((p) => p.name === packageName);
  },

  async loadPackages() {
    try {
      packages = await invoke("get_packages");
    } catch (e) {
      console.error(`Failed to load packages: ${e}`);
    }
  },

  async loadInstalledPackages() {
    isLoading = true;
    try {
      installedPackages = await invoke("get_installed_packages", {
        serverUrl: serverUrlStore.serverUrl,
      });
    } catch (e) {
      console.error(`Failed to load installed packages: ${e}`);
    } finally {
      isLoading = false;
    }
  },

  async installPackage(name: string) {
    try {
      await invoke("install_package", {
        name,
        serverUrl: serverUrlStore.serverUrl,
      });
      await this.loadInstalledPackages();
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
      await this.loadInstalledPackages();
    } catch (e) {
      console.error(`Failed to delete ${name}: ${e}`);
      throw e;
    }
  },
};
