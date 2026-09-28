# KMJ Desktop Commander — Minimal Main Platform API Boundary

Status: architecture contract for KSLP-v1 integration.\n\nNormative request/response, entitlement-claim, error-code and HTTP-cache schemas are defined in `docs/KSLP_V1_WIRE_CONTRACT.md`.

## Hard boundary

KMJ Desktop Commander does not require a separate KMJ server. The existing KMJ Main Platform on the primary KMJ server/VPS is the sole commercial authority.

Commander execution remains local/customer-controlled. KMJ Main Platform MUST NOT proxy ordinary SSH, terminal, Git, test, build, AI-context, file, or remote-operation traffic.

The public network boundary is deliberately small. Account state, billing state and revocation are evaluated by Main Platform while issuing or refreshing an entitlement; Commander MUST NOT poll separate account, billing or revocation APIs.

## Public/static surfaces

### GET /commander/bootstrap.json

Purpose: cacheable product/bootstrap policy only.

Contains:
- schema/version
- product identifier
- KSLP protocol version
- current public key-set URL
- entitlement refresh policy
- bootstrap refresh interval and jitter
- minimum supported Commander version
- release metadata URL
- public plan capability hints only

Rules:
- no user-specific data
- no secrets
- CDN/static-file cacheable
- ETag + Cache-Control
- normal client refresh no more than every 24–30 hours with jitter
- stale cached copy remains usable during temporary network failure

### GET /.well-known/kmj/kslp-v1/keys.json

Purpose: Ed25519 verification public keys and key rotation metadata.

Contains only public verification material:
- kid
- algorithm
- public key
- not-before
- optional retirement timestamp

Rules:
- private signing keys never appear here or in Commander
- long-lived cache with ETag
- key rollover supports overlap
- separate release-signing and entitlement-signing keys are preferred

### GET /commander/releases/<channel>.json

Purpose: signed release/update metadata.

Contains:
- version
- channel
- artifact URLs
- checksums
- artifact signature metadata
- minimum supported version
- publication timestamp

Rules:
- static/CDN-cacheable
- conditional GET/ETag
- no account lookup
- no per-device response
- binaries are served as static artifacts, not streamed through the application process

## Dynamic Commander API

Only two normal dynamic calls are required.

### POST /api/v1/commander/entitlements/activate

Called on first activation, explicit device transfer/reactivation, or after local entitlement loss.

Request:
- protocol = KSLP-v1
- product = KMJ_DESKTOP_COMMANDER
- authenticated Main Platform account/session proof
- device public-key fingerprint
- stable privacy-preserving install/device identifier
- client version
- one-time nonce/idempotency key

Server evaluates in one transaction:
- account state
- effective billing/subscription state
- plan
- device allowance
- existing activation/transfer state
- abuse/rate-limit state
- revocation state

Response:
- signed entitlement artifact
- activation id
- lease expiry
- next-refresh-not-before
- deterministic status/error code

The signed entitlement carries the effective commercial snapshot needed by Commander. No separate account-state or billing-status polling endpoint is exposed to Commander.

### POST /api/v1/commander/entitlements/refresh

Called only when the locally verified entitlement reaches its refresh window, after an explicit user recovery action, or when a security policy requires refresh.

Request:
- protocol/product
- license id
- activation id
- device proof/signature over server challenge or request nonce
- current entitlement token id
- current sequence
- client version
- idempotency key

Server performs one compact lookup/evaluation:
- license/account status
- billing-derived effective plan
- activation/device binding
- revocation
- sequence/replay protection
- entitlement limits/features
- required minimum client version

Response when unchanged:
- a newly signed short payload/lease with incremented sequence and token id

Response when changed:
- newly signed entitlement reflecting plan/account/billing/revocation state

Rules:
- target normal refresh cadence: approximately 7 days
- client obeys server next-refresh-not-before
- refreshes use randomized jitter
- exponential backoff on transient failure
- no retry storm
- offline grace is evaluated locally
- temporary Main Platform outage does not disable basic recovery/export/manual work
- expired/revoked state becomes restricted, never destructive
- secrets, source, terminal output and customer workload data are never included

## Revocation model

There is no continuously polled per-device revocation endpoint.

Normal revocation is enforced at the next entitlement refresh. High-risk revocation may shorten the next lease/refresh policy for affected future entitlements.

Administrative revoke/suspend/transfer actions live inside KMJ Main Platform's existing authenticated admin/customer surfaces; they are not a separate Commander backend service.

A revoked response uses deterministic reason codes and Commander enters RESTRICTED without deleting customer data.

## Account and billing model

Commander does not need direct billing-provider access and does not receive payment details.

Main Platform converts billing/account state into effective entitlement state:
- ACTIVE
- RENEWAL_DUE
- GRACE
- RESTRICTED

Billing webhooks update Main Platform's existing commercial records asynchronously. They do not call Commander devices. Commander observes the result only on activation/refresh.

This keeps payment-provider traffic and Commander client traffic decoupled.

## Local/offline behavior

Commander verifies Ed25519 signatures locally and caches the last valid entitlement.

State machine:
ACTIVE -> RENEWAL_DUE -> GRACE -> RESTRICTED
RESTRICTED -> ACTIVE after successful valid refresh/reactivation.

Local checks enforce:
- product/protocol
- device binding
- signature/kid
- not-before/issued-at/lease-expiry
- sequence/token id
- feature/resource limits
- basic clock rollback detection

No network round trip is required for normal commands.

## Server-load contract

For a healthy active Commander installation:
- ordinary command/execution requests to KMJ Main Platform: 0
- idle heartbeat requests: 0
- account polling requests: 0
- billing polling requests: 0
- revocation polling requests: 0
- bootstrap: about 1 cacheable request per 24–30 hours, usually served statically
- key metadata: cache/rotation driven
- release metadata: cacheable conditional checks
- dynamic entitlement traffic: normally about 1 small refresh per device per ~7 days

All refresh scheduling MUST include jitter. Responses SHOULD be compact JSON, gzip/brotli-capable, indexed by license/activation identifiers, and avoid expensive joins on the request path.

## Data-minimization rule

Main Platform receives only the minimum commercial/security metadata required to issue and protect an entitlement. It does not receive Commander project files, source code, SSH keys, shell history, terminal output, AI prompts/context, build logs, or ordinary remote-operation payloads by default.

## Explicitly out of scope

Do not add these unless a future measured requirement justifies them:
- dedicated Commander application server
- dedicated Commander database cluster
- websocket connection to every idle client
- per-command authorization API
- continuous license heartbeat
- continuous account/billing polling
- hosted SSH relay by default
- hosted build/test execution by default
- raw telemetry/event firehose
- source-code or terminal-output ingestion

This boundary is intentionally small so Commander can scale primarily with customer-side compute while KMJ Main Platform remains the single lightweight commercial control plane.
