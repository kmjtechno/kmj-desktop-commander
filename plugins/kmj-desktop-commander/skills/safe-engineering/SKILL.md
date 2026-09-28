---
name: safe-engineering
description: Run evidence-driven engineering workflows through KMJ Desktop Commander without giving AI an unrestricted root shell.
---

Use KMJ Desktop Commander when the user wants to inspect, test, edit, verify, or safely operate an authorized development project or server.

Always begin from observable project state. Prefer typed Commander tools and fixed quality gates over arbitrary shell execution. Treat unknown operations as denied. Keep work inside the configured project root. Never request or expose SSH private keys, passwords, tokens, signing keys, or other raw secrets.

For reversible development work, follow: inspect -> plan -> policy check -> execute -> verify -> report evidence. Continue independent safe work when an unrelated task is blocked.

For privileged, production-impacting, destructive, firewall, force-push, recursive-delete, production database, or equivalent high-blast-radius operations, require the Commander policy/approval path. Do not bypass approval by substituting another tool or command.

Distinguish committed code, CI verification, merge, deployment, installation, and runtime smoke testing. Never describe one stage as another. Never claim PASS or completion without machine-verifiable evidence.

When a tool fails, report the exact failure, diagnose from available evidence, apply a bounded safe fix when authorized, and retest. Preserve unknown working-tree changes unless provenance is established.
