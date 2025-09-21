import { check } from "@tauri-apps/plugin-updater";
import type { Update } from "@tauri-apps/plugin-updater";
import { invoke } from "@tauri-apps/api/core";
import { getVersion } from "@tauri-apps/api/app";
import { platform } from "@tauri-apps/plugin-os";
import { clean as semverClean, gt as semverGt } from "semver";
import { notifyError, notifyInfo } from "$utils/notify";

const TWENTY_FOUR_HOURS = 24 * 60 * 60 * 1000;
const LATEST_MANIFEST_URL =
  "https://github.com/futurekittylabs/kittynode/releases/latest/download/latest.json";

type LatestManifest = {
  version: string;
};

let currentUpdate = $state<Update | null>(null);
let dismissedTime = $state<number | null>(null);
let lastChecked = $state(0);
let processingUpdate = $state(false);
let checkingForUpdate = $state(false);
let linuxManualUpdateAvailable = $state(false);

export const updates = {
  async getUpdate(forceCheck = false) {
    const now = Date.now();
    if (forceCheck || now > lastChecked + TWENTY_FOUR_HOURS) {
      checkingForUpdate = true;
      try {
        const currentPlatform = await platform();
        if (currentPlatform === "linux") {
          linuxManualUpdateAvailable = await checkLinuxManifest();
          currentUpdate = null;
        } else {
          currentUpdate = await check();
          linuxManualUpdateAvailable = false;
        }
        lastChecked = now;
        console.info("Successfully checked for update.");
      } catch (e) {
        // Surface error to caller; leave UI notifications to callers
        console.error("Failed to check for update", e);
        throw e;
      } finally {
        checkingForUpdate = false;
      }
    }
    return currentUpdate;
  },

  get hasUpdate() {
    return currentUpdate !== null || linuxManualUpdateAvailable;
  },

  get requiresManualInstall() {
    return linuxManualUpdateAvailable;
  },

  get isDismissed() {
    if (!dismissedTime) return false;
    return Date.now() < dismissedTime + TWENTY_FOUR_HOURS;
  },

  get isProcessing() {
    return processingUpdate;
  },

  get isChecking() {
    return checkingForUpdate;
  },

  dismiss() {
    dismissedTime = Date.now();
  },

  async installUpdate() {
    if (linuxManualUpdateAvailable) {
      notifyInfo("Download the latest Kittynode", {
        description: "Open kittynode.com/download to install Linux updates.",
      });
      return;
    }

    if (!currentUpdate || processingUpdate) {
      return;
    }

    processingUpdate = true;
    try {
      let downloaded = 0;
      let contentLength = 0;

      await currentUpdate.downloadAndInstall((event) => {
        switch (event.event) {
          case "Started":
            contentLength = event.data.contentLength as number;
            console.info(
              `Started downloading ${event.data.contentLength} bytes.`,
            );
            break;
          case "Progress":
            downloaded += event.data.chunkLength;
            console.info(`Downloaded ${downloaded} from ${contentLength}.`);
            break;
          case "Finished":
            console.info("Download finished.");
            break;
        }
      });

      console.info("Update installed.");
      await invoke("restart_app");
    } catch (e) {
      notifyError("Failed to update Kittynode", e);
    }
    processingUpdate = false;
  },
};
async function checkLinuxManifest(): Promise<boolean> {
  const manifest = await invoke<LatestManifest>("fetch_latest_manifest", {
    url: LATEST_MANIFEST_URL,
  });

  if (!manifest.version) {
    throw new Error("Latest manifest is missing the version property");
  }

  const manifestVersion = semverClean(manifest.version);
  if (!manifestVersion) {
    throw new Error(
      `Latest manifest contains an invalid semver version: ${manifest.version}`,
    );
  }

  const appVersionRaw = await getVersion();
  const appVersion = semverClean(appVersionRaw);
  if (!appVersion) {
    throw new Error(`Unable to parse current app version: ${appVersionRaw}`);
  }

  return semverGt(manifestVersion, appVersion);
}
