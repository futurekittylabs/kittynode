import { packagesStore } from "$stores/packages.svelte";
import {
  dockerStatus,
  type DockerStatusValue,
} from "$stores/dockerStatus.svelte";
import { notifyError, notifySuccess } from "$utils/notify";

export function usePackageInstaller() {
  let installingPackages = $state<Set<string>>(new Set());

  function isInstalling(packageName: string): boolean {
    return installingPackages.has(packageName);
  }

  async function installPackage(packageName: string): Promise<boolean> {
    const status = dockerStatus.status;
    if (status !== "running") {
      const messageByStatus: Record<DockerStatusValue, string> = {
        running: "",
        starting: "Docker Desktop is still starting. Try again in a moment.",
        not_installed: "Install Docker Desktop to manage packages.",
        not_running: "Docker must be running to install packages.",
        unknown: "Docker must be running to install packages.",
      };

      const message =
        messageByStatus[status] ||
        "Docker must be running to install packages.";
      if (message) {
        notifyError(message);
      }
      return false;
    }

    if (installingPackages.has(packageName)) {
      return false;
    }

    // Create a new Set to trigger reactivity
    installingPackages = new Set([...installingPackages, packageName]);
    try {
      await packagesStore.installPackage(packageName);
      notifySuccess(`Successfully installed ${packageName}`);
      return true;
    } catch (error) {
      notifyError(`Failed to install ${packageName}`, error);
      return false;
    } finally {
      // Create a new Set to trigger reactivity
      const newSet = new Set(installingPackages);
      newSet.delete(packageName);
      installingPackages = newSet;
    }
  }

  return {
    get installingPackages() {
      return installingPackages;
    },
    isInstalling,
    installPackage,
  };
}
