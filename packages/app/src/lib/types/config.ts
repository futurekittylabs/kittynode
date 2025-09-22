export interface KittynodeConfig {
  capabilities: string[];
  serverUrl: string;
  lastServerUrl: string;
  hasRemoteServer: boolean;
  onboardingCompleted: boolean;
  autoStartDocker: boolean;
}
