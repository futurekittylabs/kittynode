let serverUrl = $state("");

export const serverUrlStore = {
  get serverUrl() {
    return serverUrl;
  },
  setFromConfig(url: string) {
    serverUrl = url;
  },
};
