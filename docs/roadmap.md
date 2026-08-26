# Roadmap

The roadmap is evidence-driven. Repository names describe ownership, but a new
repository is created only when its public boundary is executable.

## 0 — execution foundation (`complete`)

One vertical slice in `prism`:

- canonical content and localization types;
- capability-driven preflight;
- explicit dispatch policies and idempotency derivation;
- provider registry and scriptable fake provider;
- JSON/NDJSON execution protocol;
- schema and golden contract tests;
- CI and architecture contract.

This intentionally replaces three mostly structural bootstrap PRs. Empty crates
would freeze names without proving their boundaries.

## 1 — provider proof (`current`)

The first increment implements text-only Threads publishing behind injected
binding and HTTP boundaries, including safe error mapping and ambiguous-outcome
semantics. Next, prove controlled live validation, then add Threads media.
Instagram may later share private Meta transport/auth helpers while keeping its
product semantics separate. X remains independent. Required CI uses reduced
fixtures; live publish checks are explicit and use controlled accounts.

## 2 — control plane (`planned`)

Implement the existing `prism-hub` repository after the execution protocol and
one real provider are proven. The recommended starting point remains a
conventional Ruby/Rails API with PostgreSQL and database-backed jobs, structured
as Clean Architecture with SOLID object boundaries. The hub owns accounts,
OAuth lifecycle, scheduling, persistence policy, approvals, and OpenAPI — never
provider HTTP semantics.

## 3 — explicit intelligence (`planned`)

Implement `prism-ai` as an optional variant producer behind a hub-owned port.
AI output becomes explicit `ContentVariant` values with provenance and cannot
bypass permissions, approval, preflight, or deterministic dispatch.

## 4 — clients and autonomy (`planned`)

Implement modular Telegram surfaces in `prism-bot` and the web administration
experience in `prism-panel`, both against generated hub contracts. Future
WhatsApp, Viber, and other messaging integrations belong in isolated
`prism-bot` adapters. Prove equivalent self-hosted, aiaiaiai-managed, and
dedicated-instance profiles using immutable artifacts.

## 5 — reporting and peer integrations (`planned`)

Add optional archive/metrics modules and HQBase mail notifications through the
public HQBase API. Client-domain modules remain outside Prism core.

## Deferred decisions

- optional developer CLI, when a concrete non-browser workflow justifies it;
- canonical aiaiaiai infrastructure repository;
- exact `prism-hub` license;
- unattended HQBase service authorization;
- crates.io names and publication policy;
- daemon transport, justified by measurement rather than assumed early.

Native panel applications, including `prism-panel-ios`, are outside this
roadmap. They require a separate product and architecture decision before any
supporting abstraction is added.

Detailed implementation state and exit criteria live in
[`status.md`](status.md).

<!-- © 2026 aiaiaiai · aiaiaiai.org -->
