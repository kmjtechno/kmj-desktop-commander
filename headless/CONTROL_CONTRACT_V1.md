# KMJ Commander Control Contract v1

Security baseline for the ChatGPT-to-VPS channel.

- Executor is deny-by-default and exposes no arbitrary shell.
- CloudOS workspace is pinned to `/home/info/kmj-cloudos` and canonicalized.
- Initial operations: inspect, git-status, diff-check, py-compile, provider-tests.
- Every operation must pass the existing Commander policy engine.
- Future network gateway must bind only to loopback/Unix socket until authenticated transport is configured.
- Authentication must use short-lived credentials; authorization must require both scope and operation allowlist.
- Replayed, expired, malformed, unknown, privileged, destructive, and production-mutating requests fail closed.
- Audit records must redact credentials and secrets.
