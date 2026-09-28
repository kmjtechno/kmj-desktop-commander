# KMJ Desktop Commander — Commercial Model

Status: implementation contract; billing remains disabled until payment and entitlement gates are verified.

## Principles
- Apache-2.0 core rights remain intact.
- Manual core development remains useful without a subscription.
- Paid value comes from scale, orchestration, automation, fleet/team controls and managed commercial capabilities.
- Customer source code, SSH keys, secrets and raw execution data remain customer-controlled by default.
- Billing state never grants raw root access; execution policy remains independent from commercial entitlement.

## Product ladder
### Free
- 3 server profiles
- 1 device
- unlimited manual operations
- 50 autopilot runs/month when autopilot accounting ships
- local/customer execution

### Pro
- 25 server profiles
- 3 devices
- unlimited manual operations
- 2,000 autopilot runs/month
- advanced parallel/autopilot workflows, higher automation limits and priority release capabilities as implemented

### Team
- 100 server profiles
- 10 devices
- unlimited manual operations
- 10,000 autopilot runs/month
- team/fleet policy, shared approvals, audit/export and collaboration capabilities as implemented

## Revenue streams
1. Pro subscriptions for individual power users and small businesses.
2. Team subscriptions for shared servers, approvals, fleet policy and audit workflows.
3. Enterprise contracts for SSO, governance, support, private distribution and larger fleet limits.
4. Optional paid support/onboarding for complex customer infrastructure.
5. Optional commercial add-ons only where they provide real operational value; no artificial disabling of Apache core functionality.

## Commercial authority chain
KMJ Main Platform is the authority for product catalog, account/license/entitlement and future billing metadata.
Commander enforces signed/cached entitlements locally. Normal project execution must not depend on a round trip to KMJ Platform.

## Activation gates
Billing MUST remain off until all are true:
- commercial control-plane CI is green;
- authenticated account linking exists;
- signed entitlement issuance and offline verification exist;
- device binding/revocation and clock/expiry rules are tested;
- quota accounting for paid automation exists;
- checkout provider integration is end-to-end tested in sandbox;
- refund/cancel/renewal state transitions are handled;
- pricing/tax/currency presentation is approved;
- privacy/terms/refund/support surfaces are published;
- upgrade/downgrade cannot lose customer project data;
- billing outage cannot block entitled manual development unexpectedly.

## Current implementation
- Main Platform public bootstrap defines Free/Pro/Team limits with billing disabled.
- Commander reads the bootstrap.
- Commander displays current public-beta plan state.
- Commander enforces the active bootstrap server-profile limit.
- Manual operations remain unlimited.
- Checkout/payment and signed authenticated entitlements are NOT yet active.

## Promotion/distribution loop
After release gates are green:
1. Release-quality README with verified screenshots, supported platforms, security model, install paths and honest current capabilities.
2. Signed GitHub release/installers and checksums.
3. KMJ Main Platform product page + SEO/schema/download funnel.
4. Launch posts and demo media for LinkedIn, X, Facebook, Instagram and relevant developer communities.
5. Changelog/release posts for meaningful versions, not spam.
6. Measure repository visits -> installer downloads -> activated devices -> retained users -> Free-to-Pro conversion.
7. Improve onboarding/product based on measured drop-off; never optimize promotion independently of product quality.

## Pricing
Exact prices are intentionally not hard-coded until payment economics, taxes, target-market validation and billing provider costs are verified. Entitlement tiers are designed now so pricing can change without rebuilding the execution architecture.
