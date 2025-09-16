import { invoke } from "@tauri-apps/api/core";
import { platform } from "@tauri-apps/plugin-os";

const mobilePlatforms = ["ios", "android"] as const;

export type DockerStatusValue =
  | "unknown"
  | "not_installed"
  | "starting"
  | "running"
  | "not_running";

let status = $state<DockerStatusValue>("unknown");
let interval: number | null = $state(null);

export const dockerStatus = {
  get status() {
    return status;
  },

  get isRunning() {
    return status === "running";
  },

  get isStarting() {
    return status === "starting";
  },

  get isInstalled() {
    return status !== "not_installed";
  },

  async checkDocker() {
    try {
      const currentPlatform = platform();
      const isMobile = mobilePlatforms.some(
        (value) => value === currentPlatform,
      );

      status = isMobile
        ? "running"
        : await invoke<DockerStatusValue>("get_docker_status");
    } catch (e) {
      console.error(`Failed to check Docker status: ${e}`);
      status = "not_running";
    }
  },

  startPolling(intervalMs = 5000) {
    const currentPlatform = platform();
    const isMobile = mobilePlatforms.some((value) => value === currentPlatform);

    if (!isMobile && status === "unknown") {
      status = "starting";
    }

    this.checkDocker();
    interval = window.setInterval(() => this.checkDocker(), intervalMs);
  },

  stopPolling() {
    if (interval !== null) {
      window.clearInterval(interval);
      interval = null;
    }
  },
};
