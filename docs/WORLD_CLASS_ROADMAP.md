# KMJ Desktop Commander — World-Class Autonomous Development Roadmap

Status: canonical product direction
Principle: continuous autonomous work without giving an AI an unrestricted root shell.

## Locked speed upgrades
1. Persistent VPS Agent — avoid CI queue latency; durable reconnect/recovery.
2. Parallel Worktree Engine — isolated concurrent frontend/backend/test/fix workers.
3. Smart Test Selection — changed-code/dependency-aware tests first, full gates before promotion.
4. Project Auto-Detection — Laravel/PHP, Node/React/TS, Rust, Python, Go, Java, .NET, Docker and extensible adapters.
5. Failure -> Diagnose -> Fix -> Retest loop with bounded retries and evidence.
6. Checkpoints + Automatic Rollback before risky edits/deployments.
7. Durable Task Queue — blocked work is isolated; independent work continues.
8. Dependency/Build Cache — Composer, pnpm/npm, Cargo and other toolchain caches.
9. Controlled Privileged Broker — narrowly scoped approved sudo operations; never expose a permanent unrestricted root shell to AI.
10. Evidence Ledger — task, diff, commit SHA, tests, build, deploy, rollback and approval evidence.
11. Resource-Aware Scheduler — CPU/RAM/disk/load-aware concurrency and backpressure.
12. Kristi Autopilot Orchestrator — goal -> plan -> parallel execution -> verification -> report.

## World-class platform requirements

### A. Autonomous engineering core
- Persistent state machine: DISCOVER -> PLAN -> POLICY_CHECK -> EXECUTE -> VERIFY -> DIFF -> APPROVAL -> COMMIT/PUSH -> DEPLOY -> OBSERVE.
- Resumable jobs after app, network, VPS or runner restart.
- Dependency-aware DAG scheduler with priorities, retries, timeouts and cancellation.
- Multi-agent/worktree isolation with deterministic merge/conflict handling.
- Semantic repository index, symbol/dependency graph, LSP integration and incremental context.
- Change-impact analysis to select tests/builds and estimate blast radius.
- Automatic root-cause loop using compiler/test/runtime/log evidence rather than blind retries.
- Definition-of-done gates per project and per environment.

### B. Performance
- Direct persistent agent transport for low-latency execution; CI remains an independent verification/fallback path.
- Streaming stdout/stderr, structured events and incremental results.
- Content-addressed dependency/build/test cache with safe invalidation.
- Incremental builds/tests and changed-file pipelines.
- Adaptive parallelism based on server resources and workload.
- Benchmarks for task latency, repair success, token/model cost, test time and deployment recovery time.

### C. Safety and control
- Deny-by-default policy-as-code with risk classes: READ, TEST, EDIT, GIT_WRITE, SERVICE, DEPLOY, PRIVILEGED, DESTRUCTIVE.
- Least-privilege service identity; capability-scoped sudo broker for explicitly approved system operations.
- Workspace sandboxing/container isolation for untrusted builds.
- Path traversal, command injection, symlink and secret-exfiltration defenses.
- Secret vault integration: OS keychain/agent/environment references; never store raw SSH private keys/passwords in project profiles.
- Automatic secret redaction from logs/evidence.
- Immutable audit trail for commands, approvals and state transitions.
- Signed releases, reproducible builds, SBOM, dependency/provenance verification and update signature validation.
- Production protections: environment identity, maintenance/approval gates, backup/checkpoint validation and rollback rehearsal.

### D. Reliability
- Heartbeats, watchdog, reconnect and self-recovery for persistent agents.
- Idempotent operations and operation IDs to prevent duplicate execution.
- Transactional/atomic project file writes.
- Snapshot/checkpoint before risky changes.
- Health verification after deploy plus automatic rollback on failed health gates.
- Offline queue and eventual sync when connectivity returns.
- Disaster-recovery export/import of non-secret configuration and evidence.

### E. AI/model architecture
- Provider-neutral model router: local/free models first when appropriate, optional cloud models when configured.
- Task-specific routing for planning, coding, review, debugging and summarization.
- Context budget manager and repository retrieval instead of sending whole repositories.
- Deterministic tool APIs: AI chooses typed operations, not arbitrary root shell strings.
- Independent reviewer/verifier stage for high-risk changes.
- Model/tool quality benchmarks with regression suites before upgrades.

### F. Developer experience
- One-click project onboarding: repo + SSH profile + project root -> detect -> validate -> ready.
- Terminal-quality streaming UI plus structured task timeline.
- Human-readable plan/diff/test evidence before high-risk approval.
- Multi-project dashboard with status, blockers, resources, branches and deployments.
- Searchable job history and reproducible task bundles.
- Desktop notifications and mobile companion approvals for privileged actions.
- Fast keyboard workflow, accessibility and internationalization.

### G. Extensibility
- Stable plugin/adapter SDK for stacks, CI, cloud providers, databases and deployment systems.
- MCP integration with versioned schemas and capability discovery.
- Project-level commander config checked into Git for reproducible policies/gates.
- Hooks for GitHub/GitLab/Bitbucket and generic Git remotes without locking core architecture to one provider.
- API/CLI for automation while preserving the same policy engine as desktop.

### H. Enterprise readiness
- Local-first architecture: source code, SSH keys, secrets and raw terminal output stay on customer-controlled systems by default.
- RBAC/team approvals and environment-specific policies when team mode is enabled.
- Fleet inventory for many VPS/project agents with per-project isolation.
- SSO/audit export optional for enterprise; core development remains usable without KMJ cloud dependency.
- Licensing/account/update metadata separated from customer execution data.
- Clear data-retention controls and privacy boundaries.

## 24x7 superfast execution architecture
- Always-on Agent Supervisor: the VPS agent runs as a supervised service with heartbeat, watchdog, crash restart and boot-time recovery.
- Persistent transactional queue: jobs, DAG state, retries, leases and checkpoints survive process/VPS/network restarts.
- Zero-idle scheduler: whenever runnable safe work exists and resource limits permit it, an available worker claims it automatically.
- Worker pool + work stealing: independent coding, test, review, build and documentation workers run concurrently without duplicating ownership.
- Warm workers: keep common toolchains/repository metadata ready to remove repeated startup and dependency-discovery latency.
- Test sharding: split large suites across available CPU/worker capacity, then aggregate deterministic evidence.
- Build/test cache federation: content-addressed local cache first; optional peer/remote cache without requiring KMJ cloud.
- Incremental semantic index: update only changed symbols/files instead of rescanning entire repositories.
- Speculative safe verification: run likely tests/static analysis in parallel with independent work, cancel obsolete work when a newer revision supersedes it.
- Priority lanes: production incident > failed gate > active user goal > optimization/cleanup, configurable per project.
- Resource reservations: preserve RAM/disk/CPU headroom so parallelism never makes the VPS unusable.
- Disk-pressure guardian: cache eviction, artifact retention and workspace cleanup before low disk can stop development.
- Circuit breakers: repeated identical failures are quarantined instead of burning CPU/tokens forever; unrelated DAG work continues.
- Dead-letter/blocker lane: permanently blocked nodes retain evidence and retry conditions while the main queue keeps moving.
- Lease/fencing tokens: after reconnect/failover, stale workers cannot execute the same mutation twice.
- Automatic stale-work cancellation when commits, requirements or dependencies invalidate an older task.
- Scheduled maintenance lane for dependency updates, security checks, cache warming and repository health when foreground work is idle.
- Multi-project fair scheduler so one huge project cannot starve all other projects.
- Optional multi-VPS worker federation: add customer-controlled workers later for horizontal scale and failover; single-VPS mode remains first-class.
- CI independence: direct agent is the fast execution path; GitHub/other CI provides independent release verification and fallback, not the normal bottleneck.
- Local/free-first AI routing and response caching where safe, with model fallback so one unavailable provider does not stop non-model work.
- Toolchain capsules: reproducible container/dev-environment adapters for PHP, Node, Rust, Python, Go, Java, .NET and extensible stacks.
- Observability: queue depth, worker utilization, cache hit rate, edit-to-green latency, failure/retry rate, CPU/RAM/disk and agent uptime.
- SLO target: recover queued safe work automatically after agent restart and keep unrelated safe work progressing through individual task failures.

## Universal development acceleration
- Repository bootstrap profiler learns the fastest verified install/test/build commands and stores them as project configuration.
- Dependency graph + changed-symbol analysis chooses the smallest valid verification set during iteration.
- AST/LSP-aware edits when available, falling back to validated text patches for unknown languages.
- Preflight detects missing runtimes/dependencies before assigning expensive work.
- Layered verification: syntax/type/static checks -> targeted tests -> affected integration tests -> full release gate.
- Deterministic merge queue for parallel worktrees with automatic rebase/conflict classification and retest after integration.
- Reusable golden environment definitions to reproduce a known-good project toolchain quickly.
- Benchmark history detects performance regressions in Commander itself and in configured project gates.
- Artifact/result deduplication prevents repeating an identical successful build/test for the same inputs.
- Background repository maintenance only consumes spare capacity and yields immediately to foreground goals.

## Continuous-work invariant
A blocked task MUST NOT stop unrelated safe work. Commander records the blocker, continues independent DAG nodes, retries recoverable failures, and requests approval only when the blocked operation truly requires it.

## Root-control invariant
"Full control" means enough scoped capability to complete legitimate development/operations reliably. It does NOT mean exposing a permanent unrestricted root shell to an AI. Privileged actions use a narrow audited broker and explicit policy/approval.

## Near-term implementation order
P0: Persistent Agent Supervisor + transactional durable queue + heartbeat/watchdog/recovery + streaming protocol.
P0: Zero-idle worktree/worker scheduler + work stealing + smart test selection/sharding + failure/fix/retest loop.
P0: Checkpoint/rollback + evidence ledger + resource/disk-pressure scheduler + circuit breaker/blocker lane.
P1: Stack auto-detection/adapters + semantic code index/LSP + cache.
P1: Privileged broker + service/deploy adapters + production health/rollback gates.
P1: Signed updater, SBOM/provenance, secret redaction and security regression suite.
P2: Kristi orchestration UX + multi-project dashboard + mobile approvals.
P2: Provider-neutral model router + plugin SDK + fleet/team/enterprise capabilities.

## Success metrics
- No manual command copying for normal project development.
- Safe work continues when an unrelated task is blocked.
- Every material change has machine-verifiable evidence.
- Recovery from agent/network restart without losing job state.
- No unrestricted AI root shell.
- Measurably lower median edit-to-verified-result time as features ship.
- Cross-platform release gates remain green before distribution.
- Safe queued work resumes automatically after agent/process/network recovery.
- Track agent uptime, queue idle-with-work time, cache hit rate and median edit-to-green latency.
