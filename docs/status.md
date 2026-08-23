# Implementation status

This document separates the behavior implemented in this repository from the
product capabilities planned for the wider Prism ecosystem. A roadmap entry is
not a claim that the capability exists.

The status below describes `0.1.0-alpha.1` on the execution-foundation branch.

## Current: executable foundation

### Public surfaces

| Surface | Implemented behavior |
| --- | --- |
| `prism-core` | Provider-neutral content, localization, audience, capability, request, validation, result, receipt, and event types |
| `prism-provider` | Object-safe async adapter contract and deterministic registry |
| `prism-protocol` | Exact `prism-execution.v1` request and response envelopes plus generated JSON Schema |
| `prism-runtime` | Stateless `capabilities`, `validate`, and `publish` operations over JSON or NDJSON |
| `prism-testkit` | Scriptable fake provider, safe call recording, and adapter conformance probe |
| `xtask` | Contract generation, schema-drift validation, and repository copyright checks |

There are no real network provider adapters in the repository yet. The optional
test provider performs no external action.

### Current execution logic

For `validate` and `publish`, the runtime performs these stages in order:

1. validate request-local structure and reject duplicate or empty identifiers;
2. resolve one explicit eligible content variant for each ordered target;
3. resolve the registered adapter and discover its capability snapshot;
4. validate canonical content limits, provider namespaces, and secret-bearing
   extension keys;
5. run provider-native preflight without publishing;
6. finish preflight for every target before crossing any publish boundary;
7. apply the request's dispatch policy;
8. publish eligible targets sequentially in input order and return ordered,
   independent outcomes and sequence-numbered events.

`validate` stops after preflight and never calls `ProviderAdapter::publish`.
`capabilities` validates one provider context and returns the adapter snapshot.

### Current dispatch semantics

| Policy | Current guarantee |
| --- | --- |
| `require_all_valid` | If any target fails preflight, no external publish call is made. Targets that passed preflight are reported as `skipped`. |
| `independent` | Every target is preflighted first. Each valid target is then attempted; invalid and provider-failed targets remain isolated. |

Prism does not promise cross-provider atomicity. Once the first external call is
made, upstream providers cannot participate in a shared transaction.

### Current determinism and idempotency boundary

- target and outcome order follows request order;
- event sequence numbers contain no wall-clock time;
- variant selection is explicit and deterministic;
- target idempotency material is derived from the logical publication key and
  target ID;
- adapters may map or hash this material to provider limits;
- Prism currently has no durable deduplication store or retry scheduler.

Given the same request, capabilities, and adapter responses, Prism produces the
same selections, error classes, outcome order, and domain event sequence.
Provider latency, upstream state, and external IDs are observations outside that
deterministic boundary.

### Current security boundary

- protocol and domain values carry opaque credential references, never raw
  provider tokens;
- provider options must use the selected provider namespace;
- compound secret-bearing keys such as `access_token` are rejected;
- stdout is reserved for protocol envelopes;
- diagnostics go to stderr without request payloads;
- real adapters are responsible for secret resolution and upstream redaction.

### Current verification

Full CI runs formatting, Clippy with warnings denied, all workspace tests,
generated-contract drift checks, copyright/license checks, and rustdoc with
warnings denied. Golden request/response fixtures protect the wire contract.
Required CI contains no live publish operation.

## Not implemented

The following capabilities are intentionally absent from the current code:

- live Threads, Instagram, X, Telegram, Mastodon, or other provider calls;
- OAuth flows, token storage, or a concrete credential resolver;
- media fetching, transformation, upload, or durable storage;
- retries, backoff, scheduling, queues, or long-term idempotency records;
- accounts, workspaces, approvals, persistence, audit history, or reporting;
- an HTTP service, daemon lifecycle, or remote transport;
- `prism-hub`, `prism-cli`, `prism-web`, `prism-telegram`, or `prism-ai`;
- deployment artifacts or hosted infrastructure.

## TODO: implementation sequence

### 1. Prove one real provider

Add the first live adapter without changing core ownership boundaries.

Exit criteria:

- an injected credential-resolution boundary returns secrets only inside the
  adapter;
- capability discovery, preflight, publish, idempotency mapping, error
  classification, receipt redaction, and rate-limit behavior are tested;
- required CI uses reduced recorded fixtures and performs no live publish;
- a separate opt-in validation path can exercise a controlled provider account;
- any provider-specific fields remain namespaced until proven portable.

Threads is the preferred first Meta proof. Instagram may reuse private
transport/auth helpers later, while retaining a distinct product adapter. X and
other providers remain independent adapters.

### 2. Prove an external operator

Create `prism-cli` as the first consumer outside the Rust workspace.

Exit criteria:

- direct JSON/NDJSON execution works without a hub;
- machine-readable output and exit behavior are stable and documented;
- credentials enter only through a supported resolver, never request payloads;
- compatibility is tested against committed protocol fixtures.

### 3. Introduce the control plane

Create `prism-hub` only after the execution protocol and one real adapter have
been proven.

The hub will own accounts, workspaces, OAuth lifecycle, scheduling, persistence
policy, approvals, audit history, jobs, and its external API. It will invoke
Prism through a versioned boundary and will not reimplement provider HTTP
semantics. The current preferred direction is a conventional Ruby/Rails API,
PostgreSQL, and database-backed jobs; this remains a design target, not shipped
code.

### 4. Add clients and deployment profiles

- generate hub clients from its versioned API contract;
- build Telegram and web interactions against the hub, not provider APIs;
- support direct stateless, self-hosted hub, aiaiaiai-managed, and isolated
  dedicated-client profiles from identical versioned artifacts;
- keep application build/release ownership separate from infrastructure wiring.

### 5. Add reporting and peer integrations

- define optional archive, delivery-history, metrics, and reporting modules;
- integrate HQBase through its public Mail API from the hub boundary;
- keep client-domain concepts outside `prism-core`;
- document retention and data-minimization policies per deployment profile.

### 6. Add explicit intelligence

Create `prism-ai` only after manual localization and delivery contracts are
stable. AI output must become explicit `ContentVariant` values with provenance.
It cannot bypass permissions, approval, preflight, dispatch policy, or
idempotency behavior.

## Deferred decisions

These require separate evidence and repository-level decisions:

- which provider follows the first Meta proof;
- public crate names and crates.io publication policy;
- daemon, Unix-socket, or HTTP runtime transport;
- exact `prism-hub` license;
- canonical aiaiaiai infrastructure repository;
- unattended HQBase service authorization;
- delivery concurrency and retry semantics.

See [`architecture.md`](architecture.md) for ownership invariants,
[`protocol.md`](protocol.md) for the wire contract, and
[`roadmap.md`](roadmap.md) for the ecosystem-level sequence.

<!-- © 2026 aiaiaiai · aiaiaiai.org -->
