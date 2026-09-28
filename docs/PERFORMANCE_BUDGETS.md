# KMJ Desktop Commander performance budgets

Status: normative release gate.

These budgets protect Commander's local-first, near-zero-server-load architecture. A release candidate MUST NOT be merged or published when a required performance gate exceeds a hard budget without an explicit, reviewed budget change in this file and `performance-budgets.json`.

## Release budgets

| Metric | Target | Hard failure |
|---|---:|---:|
| Idle CPU after warm-up | p95 <= 0.50% | any sampled 1 s interval > 1.00% |
| Idle resident memory (RSS) | p95 <= 160 MiB | > 180 MiB |
| Idle network | 0 application HTTP requests / 10 min | any unsolicited request |
| Warm startup to usable main window | p95 <= 1,500 ms | > 3,500 ms |
| Cold startup to usable main window | p95 <= 2,500 ms | > 3,500 ms |
| Stripped Commander executable | <= 20 MiB | > 20 MiB |
| Installer/package, excluding separately installed system WebView runtime | <= 35 MiB | > 35 MiB |
| Frontend `dist/` raw total | <= 2 MiB | > 2 MiB |

All MiB values are binary MiB (1,048,576 bytes).

## Measurement contract

Runtime measurements use release builds only. Debug/dev-server measurements are invalid.

Idle measurements begin after a 30-second warm-up and continue for 10 minutes with no user input, no active SSH session, no build/test task and no explicitly requested refresh. CPU is sampled every second. Report p50, p95 and maximum. RSS is the full Commander process tree, including its owned WebView processes where the platform exposes them.

Startup is measured for 20 launches. **Usable** means the main window has rendered its first interactive Commander UI and accepts input; process-spawn time alone is not sufficient. Warm startup uses populated OS caches. Cold startup uses a fresh application process and cleared Commander UI/bootstrap cache but MUST NOT deliberately flush the entire host OS page cache on shared CI.

Network accounting counts application-originated HTTP(S)/WebSocket requests from Commander. OS/WebView update traffic outside the Commander process is excluded. During the 10-minute idle window the allowed count is exactly zero.

An expired/missing bootstrap may cause at most one cacheable public bootstrap GET during startup. Activation and entitlement refresh are explicit lifecycle events and are measured separately; they MUST NOT become idle polling. There is no allowance for telemetry heartbeat, account polling, billing polling, revocation polling, per-command licensing or always-on KMJ sockets.

## CI enforcement

`scripts/check-performance-budgets.mjs` is the single budget evaluator. It exits non-zero on malformed budgets or a metric violation.

Fast PR CI MUST:
1. validate the budget configuration;
2. build the frontend and fail if raw `dist/` exceeds 2 MiB;
3. keep Rust/frontend/MCP/headless correctness gates green.

Release/performance CI MUST additionally feed a measured JSON result to the evaluator containing:
- `idle_cpu_p95_percent`
- `idle_cpu_max_percent`
- `idle_rss_p95_mib`
- `idle_rss_max_mib`
- `idle_network_requests_10m`
- `startup_warm_p95_ms`
- `startup_cold_p95_ms`
- `startup_max_ms`
- `stripped_executable_mib`
- `installer_mib`

A missing required metric is a failure, not a pass.

## Change control

Performance budgets are product contracts, not moving baselines. CI MUST NOT automatically raise them after a regression. Any increase requires a dedicated reviewed commit explaining the measured reason and user benefit. Feature work should instead reduce or stay within the budget.

Platform-specific stricter budgets may be added. A platform-specific override may never silently weaken the global hard ceiling.
