import { invoke } from "@tauri-apps/api/core";

export type RiskLevel = "read_only" | "reversible" | "privileged" | "destructive";
export type JobStatus = "running" | "succeeded" | "failed";

export interface SystemProbe {
  app_version: string;
  platform: string;
  architecture: string;
  policy_mode: string;
}

export interface PolicyDecision {
  operation: string;
  risk: RiskLevel;
  allowed: boolean;
  approval_required: boolean;
  reason: string;
}

export interface RemoteProfile {
  host: string;
  username: string;
  port: number;
}

export interface RemoteProbeResult {
  target: string;
  success: boolean;
  exit_code: number | null;
  output: string;
}

export interface JobRecord {
  id: string;
  operation: string;
  target: string;
  status: JobStatus;
  started_ms: number;
  finished_ms: number | null;
  summary: string | null;
}

export const commander = {
  probe: () => invoke<SystemProbe>("system_probe"),
  evaluate: (operation: string) =>
    invoke<PolicyDecision>("evaluate_operation", { operation }),
  remoteProbe: (profile: RemoteProfile) =>
    invoke<RemoteProbeResult>("remote_probe", { profile }),
  jobs: () => invoke<JobRecord[]>("list_jobs"),
};
