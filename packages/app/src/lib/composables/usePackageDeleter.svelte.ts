import { packagesStore } from "$stores/packages.svelte";
import { operationalStateStore } from "$stores/operationalState.svelte";
import { notifyError, notifySuccess } from "$utils/notify";
import { goto } from "$app/navigation";

export function usePackageDeleter() {
  let deletingPackages = $state<Set<string>>(new Set());

  function isDeleting(packageName: string): boolean {
    return deletingPackages.has(packageName);
  }

  async function deletePackage(
    packageName: string,
    options?: { redirectToDashboard?: boolean },
  ): Promise<boolean> {
    if (!operationalStateStore.canManage) {
      notifyError("Cannot manage packages in the current operational state");
      return false;
    }

    const status = packagesStore.installationStatus(packageName);

    if (status === "unknown") {
      notifyError(
        "Package status is still loading. Try again once it finishes.",
      );
      return false;
    }

    if (status !== "installed") {
      notifyError(`${packageName} is not currently installed`);
      return false;
    }

    if (deletingPackages.has(packageName)) {
      return false;
    }

    deletingPackages = new Set([...deletingPackages, packageName]);
    try {
      await packagesStore.deletePackage(packageName);
      notifySuccess(`Successfully deleted ${packageName}`);

      if (options?.redirectToDashboard) {
        await goto("/");
      }

      return true;
    } catch (error) {
      notifyError(`Failed to delete ${packageName}`, error);
      return false;
    } finally {
      const newSet = new Set(deletingPackages);
      newSet.delete(packageName);
      deletingPackages = newSet;
    }
  }

  return {
    get deletingPackages() {
      return deletingPackages;
    },
    isDeleting,
    deletePackage,
  };
}
