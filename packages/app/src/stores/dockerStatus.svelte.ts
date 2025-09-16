import { invoke } from "@tauri-apps/api/core";
import { platform } from "@tauri-apps/plugin-os";

let isRunning = $state<boolean | null>(null);
let interval: number | null = $state(null);

export const dockerStatus = {
  get isRunning() {
    return isRunning;
  },

  async checkDocker() {
    try {
      isRunning = ["ios", "android"].includes(platform())
        ? true
        : await invoke("is_docker_running");
    } catch (e) {
      console.error(`Failed to check Docker status: ${e}`);
      isRunning = false;
    }
  },

  startPolling(intervalMs = 5000) {
    this.checkDocker(); // Initial check
    interval = window.setInterval(() => this.checkDocker(), intervalMs);
  },

  stopPolling() {
    if (interval !== null) {
      window.clearInterval(interval);
      interval = null;
    }
  },
};
