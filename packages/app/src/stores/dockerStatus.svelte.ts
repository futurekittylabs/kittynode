import { invoke } from "@tauri-apps/api/core";
import { platform } from "@tauri-apps/plugin-os";

let isRunning = $state<boolean | null>(null);
let isStarting = $state<boolean>(false);
let interval: number | null = $state(null);
let wasAutoStarted = $state<boolean>(false);

export const dockerStatus = {
  get isRunning() {
    return isRunning;
  },

  get isStarting() {
    return isStarting;
  },

  get wasAutoStarted() {
    return wasAutoStarted;
  },

  async checkDocker() {
    try {
      isRunning = ["ios", "android"].includes(platform())
        ? true
        : await invoke("is_docker_running");

      // If Docker is running and we were starting, clear the starting state
      if (isRunning && isStarting) {
        isStarting = false;
      }
    } catch (e) {
      console.error(`Failed to check Docker status: ${e}`);
      isRunning = false;
    }
  },

  async startDockerIfNeeded() {
    try {
      const result = await invoke<string>("start_docker_if_needed");

      if (result === "starting") {
        isStarting = true;
        // Start more aggressive polling while Docker starts
        this.startPolling(2000);
      }

      // Check if Docker was auto-started
      wasAutoStarted = await invoke<boolean>("was_docker_auto_started");

      return result;
    } catch (e) {
      console.error(`Failed to start Docker: ${e}`);
      return "error";
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
