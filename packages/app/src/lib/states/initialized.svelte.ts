import { coreClient } from "$lib/client";

type InitState = "idle" | "initializing" | "initialized";

let state = $state<InitState>("idle");

export const initializedState = {
  get state() {
    return state;
  },
  get initialized() {
    return state === "initialized";
  },
  get initializing() {
    return state === "initializing";
  },
  async initialize() {
    if (state !== "idle") {
      return;
    }
    state = "initializing";
    try {
      await coreClient.initKittynode();
      state = "initialized";
    } catch (error) {
      state = "idle";
      throw error;
    }
  },
  async fakeInitialize() {
    if (state !== "idle") {
      return;
    }
    state = "initialized";
  },
  async uninitialize() {
    state = "idle";
  },
};
