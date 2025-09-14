let remoteAccess = $state(false);

export const remoteAccessStore = {
  get remoteAccess() {
    return remoteAccess;
  },
  async enable() {
    remoteAccess = true;
  },
  async disable() {
    remoteAccess = false;
  },
};
