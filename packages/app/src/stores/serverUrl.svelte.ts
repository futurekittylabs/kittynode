let serverUrl = $state("");
let lastServerUrl = $state("");

function normalize(url: string) {
  return url.trim();
}

export const serverUrlStore = {
  get serverUrl() {
    return serverUrl;
  },
  get lastServerUrl() {
    return lastServerUrl;
  },
  setFromConfig(currentUrl: string, lastUrl: string) {
    const normalizedCurrent = normalize(currentUrl);
    const normalizedLast = normalize(lastUrl) || normalizedCurrent;

    serverUrl = normalizedCurrent;
    lastServerUrl = normalizedLast;
  },
};
