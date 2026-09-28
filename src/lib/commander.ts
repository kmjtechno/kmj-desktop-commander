import { invoke } from "@tauri-apps/api/core";

export type RiskLevel = "read_only" | "reversible" | "privileged" | "destructive";
export type JobStatus = "running" | "succeeded" | "failed";
export type RemoteOperation =
  | "probe"
  | "project_inspect"
  | "git_status"
  | "git_diff_check"
  | "php_test"
  | "frontend_typecheck"
  | "frontend_build"
  | "rust_test";

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
  project_root: string | null;
}

export interface SavedProfile {
  id: string;
  label: string;
  host: string;
  username: string;
  port: number;
  project_root: string;
}

export interface RemoteResult {
  target: string;
  operation: string;
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
  execute: (profile: RemoteProfile, operation: RemoteOperation) =>
    invoke<RemoteResult>("remote_execute", { profile, operation }),
  jobs: () => invoke<JobRecord[]>("list_jobs"),
  profiles: () => invoke<SavedProfile[]>("list_profiles"),
  saveProfile: (profile: SavedProfile) =>
    invoke<SavedProfile[]>("save_profile", { profile }),
  deleteProfile: (id: string) =>
    invoke<SavedProfile[]>("delete_profile", { id }),
};
