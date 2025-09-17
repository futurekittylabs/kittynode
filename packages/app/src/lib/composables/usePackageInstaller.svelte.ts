import { packagesStore } from "$stores/packages.svelte";
import { dockerStatus } from "$stores/dockerStatus.svelte";
import { notifyError, notifySuccess } from "$utils/notify";

export function usePackageInstaller() {
  let installingPackages = $state<Set<string>>(new Set());

  function isInstalling(packageName: string): boolean {
    return installingPackages.has(packageName);
  }

  async function installPackage(packageName: string): Promise<boolean> {
    if (!dockerStatus.isRunning) {
      notifyError("Docker must be running to install packages");
      return false;
    }

    const status = packagesStore.installationStatus(packageName);

    if (status === "unknown") {
      notifyError(
        "Package status is still loading. Try again once it finishes.",
      );
      return false;
    }

    if (status === "installed") {
      notifyError(`${packageName} is already installed`);
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
