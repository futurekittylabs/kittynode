import { packagesStore } from "$states/packages.svelte";
import { operationalStateStore } from "$states/operationalState.svelte";
import { notifyError, notifySuccess } from "$utils/notify";

export function usePackageInstaller() {
  let installingPackages = $state<Set<string>>(new Set());

  function isInstalling(packageName: string): boolean {
    return installingPackages.has(packageName);
  }

  async function installPackage(
    packageName: string,
    network?: string,
  ): Promise<boolean> {
    if (!operationalStateStore.canInstall) {
      notifyError("Cannot install packages in the current operational state");
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

    installingPackages = new Set([...installingPackages, packageName]);
    try {
      await packagesStore.installPackage(packageName, network);
      notifySuccess(`Successfully installed ${packageName}`);
      return true;
    } catch (error) {
      notifyError(`Failed to install ${packageName}`, error);
      return false;
    } finally {
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
