import { packagesStore } from "$stores/packages.svelte";
import {
  dockerStatus,
  type DockerStatusValue,
} from "$stores/dockerStatus.svelte";
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
    const status = dockerStatus.status;
    if (status !== "running") {
      const messageByStatus: Record<DockerStatusValue, string> = {
        running: "",
        starting: "Docker Desktop is still starting. Try again in a moment.",
        not_installed: "Install Docker Desktop to manage packages.",
        not_running: "Docker must be running to delete packages.",
        unknown: "Docker must be running to delete packages.",
      };

      const message =
        messageByStatus[status] || "Docker must be running to delete packages.";
      if (message) {
        notifyError(message);
      }
      return false;
    }

    if (deletingPackages.has(packageName)) {
      return false;
    }

    // Create a new Set to trigger reactivity
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
      // Create a new Set to trigger reactivity
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
