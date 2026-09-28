import { invoke } from "@tauri-apps/api/core";

export type RiskLevel = "read_only" | "reversible" | "privileged" | "destructive";

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

export const commander = {
  probe: () => invoke<SystemProbe>("system_probe"),
  evaluate: (operation: string) =>
    invoke<PolicyDecision>("evaluate_operation", { operation }),
};
