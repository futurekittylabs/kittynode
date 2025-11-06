export type OperationalMode = "local" | "remote";

export interface OperationalState {
  mode: OperationalMode;
  dockerRunning: boolean;
  canInstall: boolean;
  canManage: boolean;
  diagnostics: string[];
}
