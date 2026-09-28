# Architecture

## Trust zones

`React UI -> typed Tauri IPC -> Rust policy engine -> scoped providers -> verification/audit`

The webview expresses intent. It does not own shell authority.

## Planned providers

- RemoteRunner: SSH transport with strict host-key verification and key-only authentication
- LocalRunner: constrained development operations
- GitProvider: status, diff, branch, commit and PR abstractions
- SecretProvider: OS keychain abstraction
- AuditProvider: append-only, redacted operation ledger
- AgentPlanner: Kristi planning interface; proposes operations but never bypasses policy

## Job lifecycle

`Proposed -> Classified -> Approved/Denied -> Running -> Verified -> Audited`

Every execution result should include exit status, timing, redacted output summary, changed paths, and verification evidence.

## Production principle

Autonomy increases inside a safe, reversible development sandbox. Authority decreases as blast radius increases.
