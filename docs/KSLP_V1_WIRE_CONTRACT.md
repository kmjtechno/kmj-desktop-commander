# KSLP-v1 Commander Wire Contract

Status: normative API contract for KMJ Desktop Commander and KMJ Main Platform.

All timestamps are unsigned Unix seconds in UTC. All identifiers are opaque ASCII strings. JSON requests use `Content-Type: application/json`. Unknown request fields MUST be rejected unless a future protocol version explicitly permits them.

## Encoding and signatures

KSLP-v1 entitlement artifacts use:

```json
{
  "payload": "<base64url-no-padding UTF-8 canonical entitlement JSON>",
  "signature": "<base64url-no-padding Ed25519 signature>"
}
```

The Ed25519 signature is computed over the exact ASCII bytes of the `payload` string, not over decoded JSON. Commander first verifies the signature using `kid`, then decodes `payload`, parses JSON, and validates every claim.

Canonical entitlement JSON MUST use UTF-8, no duplicate object keys, integer timestamps/counters, and deterministic serialization on the signer. The client MUST NOT accept `alg` values other than `Ed25519` for KSLP-v1.

## Common limits

- request body: maximum 16 KiB
- response body: maximum 16 KiB
- `Idempotency-Key`: 16–128 printable ASCII characters
- nonce: 16–128 base64url characters and single-use within its validity window
- client version: SemVer, maximum 64 characters
- identifiers: maximum 128 characters
- device public-key fingerprint: `sha256:<64 lowercase hex characters>`
- device proof public key: Ed25519
- server MUST rate-limit by account/license/device/IP using bounded rules
- secrets, payment instruments, source, SSH keys and execution data are forbidden

## Entitlement claims

Decoded `payload` has this exact logical schema:

```json
{
  "protocol": "KSLP-v1",
  "license_id": "lic_...",
  "customer_id": "cus_...",
  "product": "KMJ_DESKTOP_COMMANDER",
  "plan": "free|pro|team|enterprise|custom",
  "activation_id": "act_...",
  "device_id": "dev_...",
  "device_public_key_fingerprint": "sha256:...",
  "enabled_features": ["manual_operations"],
  "limits": {
    "server_profiles": 3,
    "devices": 1,
    "autopilot_runs_per_month": 50
  },
  "commercial_state": "ACTIVE|RENEWAL_DUE|GRACE|RESTRICTED",
  "issued_at": 1790627000,
  "not_before": 1790627000,
  "lease_expiry": 1791231800,
  "sequence": 42,
  "kid": "kslp-ent-2026-01",
  "token_id": "tok_...",
  "nonce": "..."
}
```

Normative validation:
- `protocol` and `product` MUST match exactly.
- `license_id`, `customer_id`, `activation_id`, `device_id`, `kid`, `token_id` and `nonce` MUST be non-empty.
- `device_id` and fingerprint MUST match the local activation.
- `enabled_features` MUST contain unique registered feature identifiers.
- every `limits` value is an unsigned integer; zero means no allowance, never unlimited.
- an unlimited capability MUST be represented by a feature whose semantics are explicitly unlimited, not by a magic integer.
- `sequence` MUST be greater than zero and MUST increase for each newly issued entitlement for an activation.
- `token_id` MUST be globally unique.
- `not_before <= lease_expiry`; future clock tolerance is at most 300 seconds.
- `commercial_state=RESTRICTED` cannot authorize paid/restricted features even if stale feature names are present.
- Commander MUST reject a validly signed entitlement if semantic validation fails.

## POST /api/v1/commander/entitlements/activate

Authentication: existing KMJ Main Platform user authentication. A bearer/session credential authenticates the account but is never embedded in the entitlement.

Required headers:

```http
Content-Type: application/json
Accept: application/json
Idempotency-Key: <16-128 printable ASCII>
```

Request:

```json
{
  "protocol": "KSLP-v1",
  "product": "KMJ_DESKTOP_COMMANDER",
  "device": {
    "device_id": "dev_...",
    "install_id": "ins_...",
    "public_key": "<base64url-no-padding 32-byte Ed25519 public key>",
    "public_key_fingerprint": "sha256:..."
  },
  "client": {
    "version": "0.1.0",
    "platform": "windows|linux|macos",
    "architecture": "x86_64|aarch64",
    "channel": "stable|beta"
  },
  "nonce": "<cryptographically random base64url value>"
}
```

`install_id` is a privacy-preserving random identifier generated at install time; it MUST NOT be a raw hardware serial, IMEI, MAC address, hostname or other unnecessary hardware identifier.

Successful response: HTTP 201 for a newly created activation; HTTP 200 for an idempotent replay that resolves to the same activation.

```json
{
  "ok": true,
  "activation_id": "act_...",
  "entitlement": {
    "payload": "<base64url>",
    "signature": "<base64url>"
  },
  "lease_expiry": 1791231800,
  "next_refresh_not_before": 1791145400,
  "server_time": 1790627000
}
```

The duplicated `lease_expiry` is scheduling convenience only. The signed claim is authoritative.

## POST /api/v1/commander/entitlements/refresh

No Main Platform interactive user session is required during routine refresh. Device possession is proven with the activation's Ed25519 key.

Required headers:

```http
Content-Type: application/json
Accept: application/json
Idempotency-Key: <16-128 printable ASCII>
```

Request:

```json
{
  "protocol": "KSLP-v1",
  "product": "KMJ_DESKTOP_COMMANDER",
  "license_id": "lic_...",
  "activation_id": "act_...",
  "device_id": "dev_...",
  "current_token_id": "tok_...",
  "current_sequence": 41,
  "client_version": "0.1.0",
  "nonce": "<single-use random base64url value>",
  "device_proof": "<base64url Ed25519 signature>"
}
```

`device_proof` signs the UTF-8 bytes of this unambiguous message:

```text
KSLP-v1\nKMJ_DESKTOP_COMMANDER\n<license_id>\n<activation_id>\n<device_id>\n<current_token_id>\n<current_sequence decimal>\n<client_version>\n<nonce>\n<Idempotency-Key>
```

Successful response: HTTP 200.

```json
{
  "ok": true,
  "activation_id": "act_...",
  "entitlement": {
    "payload": "<base64url>",
    "signature": "<base64url>"
  },
  "lease_expiry": 1791836600,
  "next_refresh_not_before": 1791750200,
  "server_time": 1791231800
}
```

Main Platform MUST evaluate account state, effective billing state, plan, device binding, revocation, replay/sequence state and effective feature limits in this single request.

## Error envelope

All expected API errors use:

```json
{
  "ok": false,
  "error": {
    "code": "KSLP_AUTH_REQUIRED",
    "message": "Authentication required.",
    "retryable": false,
    "retry_after_seconds": null
  },
  "server_time": 1790627000
}
```

`message` is safe human-readable text and MUST NOT contain secrets or internal exception details. Commander logic MUST branch on `code`, never on `message`.

### Error codes

| HTTP | Code | Retryable | Meaning |
|---:|---|:---:|---|
| 400 | KSLP_INVALID_REQUEST | no | malformed/unknown/missing field |
| 400 | KSLP_PROTOCOL_UNSUPPORTED | no | unsupported protocol |
| 400 | KSLP_PRODUCT_MISMATCH | no | wrong product |
| 400 | KSLP_INVALID_DEVICE_PROOF | no | proof/key/fingerprint invalid |
| 401 | KSLP_AUTH_REQUIRED | no | activation needs valid account authentication |
| 403 | KSLP_ACCOUNT_RESTRICTED | no | account cannot activate/renew |
| 403 | KSLP_LICENSE_REVOKED | no | license revoked |
| 403 | KSLP_ACTIVATION_REVOKED | no | this activation revoked |
| 403 | KSLP_DEVICE_MISMATCH | no | activation is bound to another device key |
| 403 | KSLP_DEVICE_LIMIT_REACHED | no | plan device allowance reached |
| 404 | KSLP_LICENSE_NOT_FOUND | no | license unavailable to this principal |
| 404 | KSLP_ACTIVATION_NOT_FOUND | no | activation unavailable |
| 409 | KSLP_REPLAY_DETECTED | no | nonce/token/sequence replay or rollback |
| 409 | KSLP_IDEMPOTENCY_CONFLICT | no | same idempotency key used with different request |
| 409 | KSLP_TRANSFER_REQUIRED | no | explicit device transfer required |
| 426 | KSLP_CLIENT_UPDATE_REQUIRED | no | client below security minimum |
| 429 | KSLP_RATE_LIMITED | yes | bounded rate limit exceeded |
| 500 | KSLP_INTERNAL_ERROR | yes | safe generic server failure |
| 503 | KSLP_TEMPORARILY_UNAVAILABLE | yes | control plane temporarily unavailable |

For 429/503, `Retry-After` MUST be returned when known and must match `retry_after_seconds`. Commander applies exponential backoff plus jitter and MUST NOT busy-loop.

For security/privacy, 404 MAY intentionally replace a more specific ownership error to avoid identifier enumeration.

## HTTP cache contract

Dynamic activation and refresh responses, including errors:

```http
Cache-Control: no-store, private
Pragma: no-cache
Vary: Authorization
X-Content-Type-Options: nosniff
```

Refresh responses that do not use `Authorization` MAY omit `Vary: Authorization`. Intermediaries MUST NOT cache dynamic entitlement responses. Server-side idempotency records are not HTTP caches.

Public bootstrap:

```http
Cache-Control: public, max-age=86400, stale-if-error=604800
ETag: "<content-hash>"
Content-Type: application/json
X-Content-Type-Options: nosniff
```

Public KSLP verification keys:

```http
Cache-Control: public, max-age=21600, stale-if-error=604800
ETag: "<content-hash>"
Content-Type: application/json
X-Content-Type-Options: nosniff
```

Release channel metadata:

```http
Cache-Control: public, max-age=300, stale-if-error=86400
ETag: "<content-hash>"
Content-Type: application/json
X-Content-Type-Options: nosniff
```

Versioned release artifacts/manifests whose URL contains an immutable version/content digest:

```http
Cache-Control: public, max-age=31536000, immutable
```

Conditional public GETs SHOULD return HTTP 304 with no body when `If-None-Match` matches.

## Idempotency and replay storage

Activation and refresh idempotency records SHOULD store only: principal/activation identifier, hashed idempotency key, request digest, resulting status/result reference, and expiry.

Recommended retention:
- activation idempotency: 24 hours
- refresh idempotency: 24 hours
- consumed nonce/replay markers: at least the maximum accepted replay window

Repeated identical requests with the same idempotency key return the original semantic result without issuing duplicate activations or incrementing sequence twice. Reuse with a different request digest returns `KSLP_IDEMPOTENCY_CONFLICT`.

## Low-load scheduling

A normal ACTIVE entitlement targets a seven-day lease/refresh cycle. Main Platform chooses `next_refresh_not_before`; Commander adds bounded random jitter and never refreshes earlier except for explicit recovery/security instructions.

No activation or refresh response instructs a continuous heartbeat. No normal Commander operation depends on a live Main Platform round trip.
