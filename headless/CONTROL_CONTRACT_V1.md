# KMJ Commander Control Contract v1

Security baseline for the ChatGPT-to-VPS execution channel.

## Transport and identity

- Protocol: `KMJ-COMMANDER/1`.
- Gateway binds to loopback only. Public exposure must terminate authenticated TLS in front of the loopback service.
- Credentials are signed, server-bound and short-lived: maximum lifetime 300 seconds.
- Required token claims: `iss`, `sub`, `aud`, `server`, `iat`, `nbf`, `exp`, unique `jti`, and `scopes`.
- Every execution request carries a UUID `request_id`, UTC timestamp, nonce, principal, operation ID and optional pinned root.
- Expired, malformed, wrong-server, wrong-scope, replayed and unknown requests fail closed.

## Authorization

Initial scopes:

- `commander:read`
- `cloudos:read`
- `cloudos:test`
- `audit:read`

Initial operation allowlist:

- `commander.probe` → `commander:read`
- `cloudos.inspect` → `cloudos:read`
- `cloudos.git_status` → `cloudos:read`
- `cloudos.diff_check` → `cloudos:read`
- `cloudos.py_compile` → `cloudos:test`
- `cloudos.provider_tests` → `cloudos:test`
- `cloudos.full_tests` → `cloudos:test`
- audit read endpoint → `audit:read`

The executor exposes no arbitrary shell. CloudOS workspace operations are pinned and canonicalized to `/home/info/kmj-cloudos`. Every operation must also pass the existing Commander policy engine.

## Audit and bounds

- Execution output is capped at 256 KiB.
- Audit records never contain bearer credentials or signing secrets.
- Every execution records principal, server, request/event IDs, operation, outcome, exit code and output SHA-256.
- Audit records are append-only and hash chained through `previous_hash` / `record_hash`.
- The audit API returns a bounded recent window only.

## Deployment gate

`ops/install-headless-gateway.sh` performs the one-click build and local verification:

1. release build,
2. protected local signing secret creation,
3. hardened systemd user-service install,
4. loopback health check,
5. unauthenticated request must return 401,
6. five-minute scoped token mint,
7. authenticated `commander.probe`,
8. auditable execution evidence.

KVM/QGA operations remain a separate allowlisted extension. No unrestricted root/sudo or production mutation is introduced by this contract.
