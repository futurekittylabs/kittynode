let serverUrl = $state("");

export const serverUrlStore = {
  get serverUrl() {
    return serverUrl;
  },
  setServerUrl(url: string) {
    serverUrl = url;
  },
};
