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

## 2 — external operator (`planned`)

Create `prism-cli` as the first consumer outside this workspace. It proves both
direct mode and stable machine-readable output before hub orchestration exists.

## 3 — control plane (`planned`)

Create `prism-hub` after the execution protocol and one real provider are proven.
The recommended starting point remains a conventional Ruby/Rails API with
PostgreSQL and database-backed jobs. The hub owns accounts, OAuth lifecycle,
scheduling, persistence policy, approvals, and OpenAPI — never provider HTTP
semantics.

## 4 — clients and autonomy (`planned`)

Add Telegram and web clients against generated hub contracts. Prove equivalent
self-hosted, aiaiaiai-managed, and dedicated-client profiles using immutable
artifacts.

## 5 — reporting and peer integrations (`planned`)

Add optional archive/metrics modules and HQBase mail notifications through the
public HQBase API. Client-domain modules remain outside Prism core.

## 6 — intelligence (`planned`)

Create `prism-ai` only after manual localization and delivery contracts are
stable. AI produces explicit variants with provenance and cannot bypass
permissions, preflight, or deterministic dispatch.

## Deferred decisions

- canonical aiaiaiai infrastructure repository;
- exact `prism-hub` license;
- unattended HQBase service authorization;
- crates.io names and publication policy;
- daemon transport, justified by measurement rather than assumed early.

Detailed implementation state and exit criteria live in
[`status.md`](status.md).

<!-- © 2026 aiaiaiai · aiaiaiai.org -->
