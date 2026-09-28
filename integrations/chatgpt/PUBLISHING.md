# KMJ Desktop Commander — ChatGPT Plugin Publication

## Canonical identity

**Product:** KMJ Desktop Commander  
**Publisher:** KMJ TECHNO  
**Short name:** KMJ Commander  
**Positioning:** Autonomous engineering control plane for customer-controlled computers and servers.

Use the canonical Commander mark from `public/commander.svg` and the KMJ Main Platform visual language: near-black surfaces, white hierarchy, restrained translucent borders, and minimal accent color.

Do not publish or promote the product under the generic name "Remote Desktop Commander". The public identity must remain clearly attributable to KMJ TECHNO.

## Public promise

KMJ Desktop Commander connects an authorized development machine or server to AI-assisted engineering workflows without exposing an unrestricted AI root shell. Tools use typed inputs, validated paths, fixed/allow-listed operations, policy checks, explicit approval where required, and auditable evidence.

## Publication architecture

The current MCP integration is stdio/local. Public ChatGPT distribution requires a separately authenticated remote transport rather than exposing SSH credentials to ChatGPT.

Target flow:

```text
ChatGPT
   │ authenticated HTTPS MCP
   ▼
KMJ Commander Remote Gateway
   │ short-lived device-scoped session
   ▼
Authorized Commander Agent / Desktop
   │ native policy + approval boundary
   ▼
Customer-controlled project / VPS
```

The gateway must relay typed intents and bounded results. It must not become a general shell proxy or permanent root credential store.

## Required gates before directory submission

- authenticated HTTPS MCP endpoint;
- explicit user/device pairing and disconnect/revocation;
- short-lived scoped session credentials;
- per-device and per-account rate limiting;
- replay/idempotency protection;
- secret and credential redaction;
- bounded tool output and request sizes;
- immutable/auditable operation evidence;
- deny-by-default unknown tool/operation behavior;
- approval challenge for privileged/destructive actions;
- public privacy, security, support and terms surfaces;
- end-to-end tests for authentication, authorization, traversal, injection, replay and disconnect;
- signed/versioned server and desktop release identity;
- verified onboarding from install -> pair -> inspect -> safe edit -> test -> disconnect.

## Promotion gate

Promotion starts only after the public plugin is installable and the end-to-end onboarding path is verified. Launch material must link to the official KMJ TECHNO product/repository surfaces and must accurately distinguish shipped capabilities from roadmap items.

Recommended launch assets:
- branded product page;
- GitHub release + checksums;
- 60–90 second safe-development demo;
- security architecture graphic;
- quick-start guide;
- LinkedIn/X/Facebook/Instagram launch variants;
- developer-community announcement focused on the open Apache-2.0 core and policy-controlled automation.

## Success metrics

Measure install -> successful pairing -> first verified project inspection -> first verified development task -> 7/30-day retention -> Free-to-Pro conversion. Do not optimize raw impressions at the expense of successful onboarding or trust.
