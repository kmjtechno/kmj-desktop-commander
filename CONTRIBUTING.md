# Contributing

Thanks for helping improve KMJ Desktop Commander.

## Principles

- Security boundaries are product behavior, not optional implementation details.
- Prefer small, reviewable changes with tests.
- Never commit credentials, private keys, tokens, customer data, or private infrastructure details.
- New native operations must have a stable operation ID, risk classification, policy behavior, and tests.
- AI/planner code must not bypass the Rust policy boundary.

## Development

1. Fork or create a feature branch.
2. Install dependencies with pnpm.
3. Run frontend and Rust quality gates.
4. Open a pull request describing behavior, security impact, and verification.

## Required gates

```bash
pnpm typecheck
pnpm build
cd src-tauri
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

For security vulnerabilities, follow SECURITY.md instead of opening a public issue.
