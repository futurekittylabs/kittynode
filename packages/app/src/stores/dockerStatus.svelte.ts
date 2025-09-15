import { invoke } from "@tauri-apps/api/core";
import { platform } from "@tauri-apps/plugin-os";

let isRunning = $state<boolean | null>(null);
let isStarting = $state<boolean>(false);
let interval: number | null = $state(null);
let wasAutoStarted = $state<boolean>(false);
let startingTimeout: number | null = $state(null);

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
        this.clearStartingTimeout();
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

        // Set a timeout to stop showing "starting" after 30 seconds
        this.setStartingTimeout(30000);
      }

      // Check if Docker was auto-started
      wasAutoStarted = await invoke<boolean>("was_docker_auto_started");

      return result;
    } catch (e) {
      console.error(`Failed to start Docker: ${e}`);
      return "error";
    }
  },

  setStartingTimeout(duration: number) {
    this.clearStartingTimeout();
    startingTimeout = window.setTimeout(() => {
      if (isStarting) {
        console.info("Docker startup timeout reached, clearing starting state");
        isStarting = false;
        // Return to normal polling interval
        this.startPolling(5000);
      }
    }, duration);
  },

  clearStartingTimeout() {
    if (startingTimeout !== null) {
      window.clearTimeout(startingTimeout);
      startingTimeout = null;
    }
  },

  startPolling(intervalMs = 5000) {
    // Clear any existing interval before starting a new one
    if (interval !== null) {
      window.clearInterval(interval);
    }
    this.checkDocker(); // Initial check
    interval = window.setInterval(() => this.checkDocker(), intervalMs);
  },

  stopPolling() {
    if (interval !== null) {
      window.clearInterval(interval);
      interval = null;
    }
    this.clearStartingTimeout();
  },
};
