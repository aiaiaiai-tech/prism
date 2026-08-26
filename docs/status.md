# Implementation status

This document separates the behavior implemented in this repository from the
product capabilities planned for the wider Prism ecosystem. A roadmap entry is
not a claim that the capability exists.

The status below describes `0.2.0-alpha.1` on the Threads provider-proof branch.

## Current: first provider proof

### Public surfaces

| Surface | Implemented behavior |
| --- | --- |
| `prism-core` | Provider-neutral content, localization, audience, capability, request, validation, result, receipt, and event types |
| `prism-provider` | Object-safe async adapter contract and deterministic registry |
| `prism-provider-threads` | Text-only official Threads adapter with injected binding resolver and HTTPS transport |
| `prism-protocol` | Exact `prism-execution.v1` request and response envelopes plus generated JSON Schema |
| `prism-runtime` | Stateless `capabilities`, `validate`, and `publish` operations over JSON or NDJSON |
| `prism-testkit` | Scriptable fake provider, safe call recording, and adapter conformance probe |
| `xtask` | Contract generation, schema-drift validation, and repository copyright checks |

The optional test provider performs no external action. The Threads adapter can
call the official production API only when an application explicitly constructs
it with a credential resolver and transport; required CI never supplies either
live credentials or a live publishing target.

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
  target ID using collision-free, versioned `prism-idempotency.v1` material;
- adapters may map or hash this material to provider limits;
- Prism currently has no durable deduplication store or retry scheduler.

Threads does not expose native idempotency for the implemented publishing path.
An ambiguous response from its final publish call is therefore classified as
`outcome_unknown`, with the safe container ID attached for reconciliation.
Callers must not retry that outcome automatically.

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
- the Threads access-token wrapper never implements display or serialization
  and always renders as `[REDACTED]` in debug output;
- Threads requests use HTTPS and bearer authorization rather than embedding a
  token in Prism domain or protocol payloads.

### Current verification

Full CI runs formatting, Clippy with warnings denied, all workspace tests,
generated-contract drift checks, copyright/license checks, rustdoc with
warnings denied, and a separate all-targets compatibility check on the declared
Rust 1.85 MSRV. Dependency resolution is locked. Golden request/response
fixtures protect the wire contract. Required CI contains no live publish
operation.

## Not implemented

The following capabilities are intentionally absent from the current code:

- Threads image, video, carousel, poll, reply, or provider-option publishing;
- live Instagram, X, Telegram, Mastodon, or other provider calls;
- OAuth flows, token storage, or a concrete credential resolver;
- media fetching, transformation, upload, or durable storage;
- retries, backoff, scheduling, queues, or long-term idempotency records;
- accounts, workspaces, approvals, persistence, audit history, or reporting;
- an HTTP service, daemon lifecycle, or remote transport;
- implemented `prism-hub`, `prism-ai`, `prism-bot`, or `prism-panel`
  applications; the first three currently have repository shells only;
- deployment artifacts or hosted infrastructure.

## TODO: implementation sequence

### 1. Complete the Threads provider proof

The text adapter and real HTTPS transport exist without changing core ownership
boundaries. Complete the milestone with controlled account evidence and media
support.

Exit criteria:

- [x] an injected credential-resolution boundary returns secrets only inside
  the adapter;
- [x] text capability discovery, preflight, two-step publish, error
  classification, receipt redaction, and rate-limit behavior are tested;
- [x] required CI performs no live publish;
- [x] ambiguous external-action outcomes are not marked safe to retry;
- [ ] a separate opt-in validation path exercises a controlled provider
  account;
- [ ] image, video, and carousel media resolution is proven;
- [ ] provider-specific options are implemented only through namespaced fields.

Instagram may reuse private Meta transport/auth helpers later while retaining a
distinct product adapter. X and other providers remain independent adapters.

### 2. Introduce the control plane

Implement the existing `prism-hub` repository after the execution protocol and
one real adapter have been proven.

The hub will own accounts, workspaces, OAuth lifecycle, scheduling, persistence
policy, approvals, audit history, jobs, and its external API. It will invoke
Prism through a versioned boundary and will not reimplement provider HTTP
semantics. The current preferred direction is a conventional Ruby/Rails API,
PostgreSQL, and database-backed jobs, with Clean Architecture and SOLID object
boundaries; this remains a design target, not shipped code.

### 3. Add explicit intelligence

Implement `prism-ai` as an optional internal service behind a hub-owned
generation port. AI output must become explicit `ContentVariant` values with
provenance. It cannot bypass permissions, approval, preflight, dispatch policy,
or idempotency behavior.

### 4. Add clients and deployment profiles

- generate hub clients from its versioned API contract;
- build Telegram bot and Mini App modules in `prism-bot` against the hub, not
  provider APIs;
- build the web administration experience in `prism-panel` against the hub;
- keep future messaging networks in isolated `prism-bot` channel adapters;
- support direct stateless, self-hosted hub, aiaiaiai-managed, and isolated
  dedicated-client profiles from identical versioned artifacts;
- keep application build/release ownership separate from infrastructure wiring.

### 5. Add reporting and peer integrations

- define optional archive, delivery-history, metrics, and reporting modules;
- integrate HQBase through its public Mail API from the hub boundary;
- keep client-domain concepts outside `prism-core`;
- document retention and data-minimization policies per deployment profile.

## Deferred decisions

These require separate evidence and repository-level decisions:

- which provider follows the first Meta proof;
- public crate names and crates.io publication policy;
- daemon, Unix-socket, or HTTP runtime transport;
- exact `prism-hub` license;
- canonical aiaiaiai infrastructure repository;
- unattended HQBase service authorization;
- delivery concurrency and retry semantics;
- optional developer CLI and its concrete operator workflow.

Native panel applications, including `prism-panel-ios`, are outside the current
scope and roadmap. No current component may introduce speculative native-client
abstractions for them.

See [`architecture.md`](architecture.md) for ownership invariants,
[`engineering-principles.md`](engineering-principles.md) for mandatory SOLID,
OOP, and scope rules,
[`protocol.md`](protocol.md) for the wire contract, and
[`roadmap.md`](roadmap.md) for the ecosystem-level sequence.

<!-- © 2026 aiaiaiai · aiaiaiai.org -->
