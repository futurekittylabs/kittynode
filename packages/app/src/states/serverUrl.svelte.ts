let serverUrl = $state("");
let lastServerUrl = $state("");

export function normalizeServerUrl(url: string) {
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
    const normalizedCurrent = normalizeServerUrl(currentUrl);
    const normalizedLast = normalizeServerUrl(lastUrl) || normalizedCurrent;

    serverUrl = normalizedCurrent;
    lastServerUrl = normalizedLast;
  },
};
