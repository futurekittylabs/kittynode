export interface KittynodeConfig {
  capabilities: string[];
  serverUrl: string;
  lastServerUrl: string;
  remoteConnected: boolean;
  onboardingCompleted: boolean;
  autoStartDocker: boolean;
}
