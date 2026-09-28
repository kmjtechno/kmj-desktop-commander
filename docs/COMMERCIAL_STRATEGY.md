# KMJ Desktop Commander — Commercial Strategy

## Positioning

KMJ Desktop Commander is a local-first engineering command center for operating projects and customer-controlled servers safely. The product should win on breadth, low infrastructure cost, predictable privacy, and governed automation rather than on hosting customer workloads.

## Infrastructure rule

Project files, terminals, tests, builds, Git operations, AI context and remote execution stay on the customer device or customer-controlled server by default. KMJ Main Platform handles only account identity, signed entitlements, release metadata, billing state, abuse controls and coarse anonymous product metrics when the user has opted in.

The desktop app must never call KMJ infrastructure per shell command. Commercial entitlements are cached locally. The target entitlement refresh interval is seven days, with revocation and critical-security refreshes allowed to shorten that window.

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
