export type OperationalMode = "local" | "remote";

export interface OperationalState {
  canInstall: boolean;
  canManage: boolean;
  diagnostics: string[];
  dockerRunning: boolean;
  mode: OperationalMode;
}
