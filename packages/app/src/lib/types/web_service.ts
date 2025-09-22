export type WebServiceState =
  | "started"
  | "already_running"
  | "stopped"
  | "not_running";

export interface WebServiceStatus {
  status: WebServiceState;
  port: number | null;
}
