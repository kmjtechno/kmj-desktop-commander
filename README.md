# KMJ Desktop Commander

> A security-first, AI-ready desktop control plane for modern engineering operations.

KMJ Desktop Commander is an open-source desktop application by **KMJ TECHNO** for operating development projects, remote runners, quality gates, Git workflows, and AI-assisted engineering from one native command surface.

## Why Commander?

Engineering work is fragmented across terminals, SSH sessions, CI pages, Git tools, dashboards, and AI assistants. Commander is designed to unify those workflows without giving an AI unrestricted shell authority.

**Core principle:** autonomy increases inside safe, reversible development workflows; authority decreases as blast radius increases.

## Architecture

```text
React UI
   │ typed intent
   ▼
Tauri IPC
   │
   ▼
Rust Policy Engine ──► Approval Gate
   │
   ▼
Scoped Providers ──► Local / SSH / Git / CI
   │
   ▼
Verification + Audit
```

The webview expresses intent. Native operations are classified and authorized in Rust before execution.

## Current foundation

- Tauri 2 native desktop shell
- Rust security/policy boundary
- React + TypeScript interface
- deny-by-default operation policy
- explicit read-only, reversible, privileged, and destructive risk classes
- minimal Tauri capability surface
- CI gates for frontend and Rust
- architecture and security documentation
- saved server/project profiles without stored credentials
- policy-gated OpenSSH remote operations
- project inspection, Git status/diff checks, PHP/TypeScript/build/Rust quality gates
- persistent local job history
- Windows NSIS installer artifact workflow
- interfaces designed for future Kristi AI orchestration

## Roadmap

The project is being built in vertical slices:

1. Secure desktop shell and policy engine
2. SSH profiles, host verification, and scoped remote operations
3. Persistent jobs, live logs, cancellation, and crash recovery
4. Project discovery and quality-gate orchestration
5. Git diff/review/commit/PR workflows
6. OS-backed secret storage and signed audit history
7. Kristi AI planner with policy-controlled execution
8. Signed installers, secure updater, and release channels

## Security

Commander intentionally does **not** expose an unrestricted shell command API to its webview or AI layer. Unknown operations are denied by default. Production deployment, privilege escalation, firewall changes, force-push, recursive deletion, and production database mutation require stronger policy and explicit approval.

Never commit SSH private keys, tokens, passwords, or production secrets. See [SECURITY.md](SECURITY.md).

## Development

Prerequisites: a current Node.js LTS, pnpm, Rust toolchain, and the platform prerequisites required by Tauri.

```bash
pnpm install
pnpm tauri dev
```

Quality gates:

```bash
pnpm typecheck
pnpm build
cd src-tauri
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

## Status

**Early development / architecture foundation.** APIs and internal structure may change before the first stable release. Do not treat the current branch as production-ready until release gates and signed binaries are published.

## Contributing

Issues and pull requests are welcome. Keep changes small, testable, security-conscious, and free of credentials or private infrastructure data. See [CONTRIBUTING.md](CONTRIBUTING.md).

## License

Licensed under the Apache License 2.0. See [LICENSE](LICENSE).

## Project

Created and maintained by **KMJ TECHNO**.

Copyright © 2026 KMJ TECHNO.
