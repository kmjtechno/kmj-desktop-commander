# KMJ Desktop Commander — Commercial Strategy

## Positioning

KMJ Desktop Commander is a local-first engineering command center for operating projects and customer-controlled servers safely. The product should win on breadth, low infrastructure cost, predictable privacy, and governed automation rather than on hosting customer workloads.

## Infrastructure rule

Project files, terminals, tests, builds, Git operations, AI context and remote execution stay on the customer device or customer-controlled server by default. KMJ Main Platform handles only account identity, signed entitlements, release metadata, billing state, abuse controls and coarse anonymous product metrics when the user has opted in.

The desktop app must never call KMJ infrastructure per shell command. Commercial entitlements are signed by KMJ Main Platform and verified locally with Ed25519. The public bootstrap is cached client-side and refreshes no more than roughly once per 24–30 hours under normal use; refresh jitter prevents synchronized client bursts. Signed commercial entitlements target a seven-day refresh cadence, with a bounded offline grace state so temporary KMJ Platform/network outages do not stop normal recovery work. Revocation and critical-security refreshes may shorten that window. There is no idle heartbeat or command-by-command licensing telemetry.

## Freemium conversion

Free is a real product, not a crippled demo:
- 1 device, 3 server profiles.
- Unlimited manual fixed operations.
- 50 governed autopilot workflows per month.
- Local/BYOK models allowed.
- Community support and stable updates.

Pro target after paid launch:
- 3 devices, 25 server profiles.
- 2,000 governed autopilot workflows per month.
- Advanced policies, reusable workflows, encrypted sync, priority updates and richer audit history.
- Initial pricing hypothesis: India ₹499/month or ₹4,999/year; international USD 9/month or USD 90/year. Validate conversion before locking prices.

Team target:
- Seat-based licensing, shared policy packs, organization audit, approval workflows and centrally managed entitlements.
- No hosted build compute included by default; cloud execution is a separately metered add-on if introduced.

Do not charge for ordinary manual SSH/Git/test operations. Monetize automation depth, governance, collaboration, managed policy and optional cloud services.

## Distribution

Desktop: Windows, Linux and macOS are first-class targets. Android and iOS are companion surfaces for approvals, monitoring and remote job control rather than unrestricted local shell execution.

Use platform-native installers and signed releases. Auto-update metadata can be static/CDN-backed so update checks do not create application-server load.

## Marketing loop

1. Free public beta with no card required.
2. Publish reproducible security and performance evidence, not unverified “best in the world” claims.
3. SEO pages around safe server automation, SSH project operations, local-first DevOps and governed AI engineering.
4. GitHub public repository, changelog, release notes and issue-driven community.
5. Short product videos showing one-click inspect → test → diff → report workflows.
6. Referral rewards should grant temporary Pro entitlement, not cash, until fraud controls are mature.
7. In-product upgrade prompts appear only at an actual paid-feature boundary or quota boundary.
8. Regional pricing and UPI for India; localized checkout globally.
9. Team/enterprise motion begins only after individual retention and reliability are measured.
10. Measure activation, weekly retained users, successful workflows, free-to-paid conversion, churn and support burden. Do not optimize vanity download counts.

## Safety and trust

Never upload source code, SSH private keys, secrets or terminal output to KMJ servers by default. Production/destructive operations remain approval-gated. Telemetry is opt-in and coarse. License enforcement must fail gracefully for temporary network loss and must not disable basic recovery/export functions.


## Scale and acquisition contract

The low-cost growth loop is product-led rather than server-compute-led:

- Free activation requires no payment card.
- Free remains useful enough to demonstrate trust: one device, three server profiles, unlimited manual operations and a meaningful governed-autopilot allowance.
- Upgrade prompts appear only when a user reaches a real scale/automation/team boundary.
- Main Platform serves cacheable static bootstrap/release metadata; Commander performs execution and entitlement checks locally.
- Signed entitlement renewal requests are small and infrequent. Clients add refresh jitter so a large installed base cannot create a synchronized renewal spike.
- Referral rewards, when implemented with abuse controls, grant time-limited Pro entitlement rather than cash.
- GitHub releases, reproducible security evidence, fast onboarding, SEO product pages and short workflow demos are the primary acquisition channels before paid advertising.
- Measure activation → first successful operation → weekly retained use → quota/feature boundary → paid conversion. Do not optimize raw downloads independently of retained successful users.
- Enterprise revenue comes from governance, SSO, fleet policy, audit/export, support and larger limits—not from routing ordinary customer commands through KMJ servers.
