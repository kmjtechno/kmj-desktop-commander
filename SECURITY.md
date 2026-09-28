# Security Policy

KMJ Desktop Commander follows a deny-by-default execution model.

## Security rules

- No unrestricted shell API is exposed to the webview.
- Native operations require stable operation IDs and risk classification.
- Unknown operations are denied by default.
- Destructive and production-impacting actions require explicit approval policy.
- Secrets belong in OS-backed credential storage, never repository files or logs.
- Remote connections must use host-key verification and key-based authentication.
- Audit records must redact secrets.
- Force push, recursive deletion, production database mutation, privilege escalation, and firewall changes are never silently approved.

## Reporting

Please do not publish exploitable security issues in a public issue. Contact KMJ TECHNO privately with reproduction details and impact.
