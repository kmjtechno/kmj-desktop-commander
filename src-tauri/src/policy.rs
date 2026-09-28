use serde::Serialize;

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RiskLevel { ReadOnly, Reversible, Privileged, Destructive }

#[derive(Serialize)]
pub struct PolicyDecision {
    operation: String,
    risk: RiskLevel,
    allowed: bool,
    approval_required: bool,
    reason: String,
}

pub fn classify_operation(operation: &str) -> PolicyDecision {
    match operation {
        "project.inspect" | "git.status" | "quality.read" | "remote.probe" => PolicyDecision {
            operation: operation.into(), risk: RiskLevel::ReadOnly, allowed: true,
            approval_required: false, reason: "Read-only inspection is permitted by the base policy.".into(),
        },
        "format.apply" | "test.run" => PolicyDecision {
            operation: operation.into(), risk: RiskLevel::Reversible, allowed: true,
            approval_required: false, reason: "Reversible development operation is permitted in a non-production workspace.".into(),
        },
        "deploy.production" | "privilege.escalate" | "firewall.change" => PolicyDecision {
            operation: operation.into(), risk: RiskLevel::Privileged, allowed: false,
            approval_required: true, reason: "Privileged or production-impacting operation requires explicit approval.".into(),
        },
        "filesystem.recursive_delete" | "git.force_push" | "database.production_mutate" => PolicyDecision {
            operation: operation.into(), risk: RiskLevel::Destructive, allowed: false,
            approval_required: true, reason: "Destructive operation is denied without a dedicated approval policy.".into(),
        },
        _ => PolicyDecision {
            operation: operation.into(), risk: RiskLevel::Privileged, allowed: false,
            approval_required: true, reason: "Unknown operations are denied by default.".into(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn inspection_is_allowed(){let d=classify_operation("project.inspect");assert!(d.allowed);assert!(!d.approval_required);}
    #[test] fn unknown_is_denied(){let d=classify_operation("unknown.action");assert!(!d.allowed);assert!(d.approval_required);}
    #[test] fn destructive_requires_approval(){let d=classify_operation("git.force_push");assert!(!d.allowed);assert!(d.approval_required);}
    #[test] fn remote_probe_is_read_only(){let d=classify_operation("remote.probe");assert!(d.allowed);assert!(!d.approval_required);}
}
