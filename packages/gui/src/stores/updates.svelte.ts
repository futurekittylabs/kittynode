import { check } from "@tauri-apps/plugin-updater";
import type { Update } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";
import { notifyError } from "$utils/notify";

const TWENTY_FOUR_HOURS = 24 * 60 * 60 * 1000;

let currentUpdate = $state<Update | null>(null);
let dismissedTime = $state<number | null>(null);
let lastChecked = $state(0);
let processingUpdate = $state(false);
let checkingForUpdate = $state(false);

export const updates = {
  async getUpdate(forceCheck = false) {
    const now = Date.now();
    if (forceCheck || now > lastChecked + TWENTY_FOUR_HOURS) {
      checkingForUpdate = true;
      try {
        currentUpdate = await check();
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
    return currentUpdate !== null;
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
      await relaunch();
    } catch (e) {
      notifyError("Failed to update Kittynode", e);
    }
    processingUpdate = false;
  },
};
