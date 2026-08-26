# Implementation status

This document separates behavior implemented in Prism from behavior that is planned or owned by other products.

## Implemented now

| Surface | Current behavior |
| --- | --- |
| `prism-core` | Content variants, capabilities, requests, validation, outcomes, receipts, and events |
| `prism-provider` | Async provider adapter contract and deterministic registry |
| `prism-provider-threads` | Text-only official Threads adapter with injected binding resolution and HTTPS transport |
| `prism-protocol` | `prism-execution.v1` request/response envelopes and generated JSON Schema |
| `prism-runtime` | Stateless `capabilities`, `validate`, and `publish` operations over JSON/NDJSON |
| `prism-testkit` | Test provider, safe call recording, and adapter conformance helpers |
| `xtask` | Contract generation, schema drift checks, and repository policy checks |

The Threads adapter can call the official production API only when an application supplies a binding resolver and transport. Required CI supplies neither live credentials nor a live target.

## Execution

For `validate` and `publish`, Prism:

1. validates request structure;
2. resolves one explicit eligible content variant per target;
3. resolves the provider adapter and capability snapshot;
4. validates provider-neutral limits and namespaced options;
5. runs provider preflight without publishing;
6. finishes preflight for every target;
7. applies the dispatch policy;
8. publishes eligible targets and returns ordered outcomes and events.

`validate` stops after preflight. `capabilities` returns one provider capability snapshot.

### Dispatch policies

| Policy | Guarantee |
| --- | --- |
| `require_all_valid` | If any target fails preflight, Prism performs no external action. Valid targets become `skipped`. |
| `independent` | Every valid target is attempted. Invalid and provider-failed targets remain isolated. |

Prism does not promise cross-provider atomicity.

## Determinism and idempotency

- target, outcome, and event order is deterministic;
- variant selection is explicit;
- target idempotency material uses versioned `prism-idempotency.v1` scope;
- adapters may map or hash that material to provider limits;
- Prism has no durable deduplication store or retry scheduler.

Threads does not expose native idempotency for the implemented publish path. If the final publish result is ambiguous, Prism returns `outcome_unknown` with safe recovery details. The caller must reconcile provider state before retry.

## Security

- protocol and domain values carry opaque credential references, never raw provider tokens;
- provider options must use the selected provider namespace;
- secret-bearing extension keys are rejected;
- stdout is reserved for protocol envelopes;
- diagnostics use stderr without request payloads;
- provider adapters own secret resolution and upstream redaction;
- the Threads token wrapper is non-serializable and redacted in debug output;
- Threads requests use HTTPS bearer authorization.

## Verification

Full CI runs formatting, Clippy with warnings denied, workspace tests, generated-contract drift checks, copyright/license checks, rustdoc with warnings denied, and a Rust 1.85 minimum-supported-version check. Dependencies are locked. Golden fixtures protect the wire contract. Required CI never performs a live publish.

## Not implemented

Prism does not currently provide:

- Threads image, video, carousel, poll, reply, or provider-specific option publishing;
- Instagram, X, Telegram, Mastodon, or other live provider adapters;
- OAuth flows, token storage, or a production credential resolver;
- media fetching, transformation, upload, or durable storage;
- retries, backoff, scheduling, queues, or durable idempotency records;
- accounts, workspaces, approvals, audit history, or reporting;
- a remote HTTP service or daemon lifecycle;
- deployment artifacts or hosted infrastructure.

The control plane, ai generation, and clients belong to separate Prism repositories. Their implementation state is tracked there, not here.

## Next Prism increments

1. Run the controlled Threads live-validation workflow and record evidence.
2. Add Threads image, video, and carousel support with media resolution and recovery behavior.
3. Add provider-specific options only through namespaced fields with proven semantics.
4. Add another provider as a separate adapter once its boundary is proven.

Instagram may reuse private Meta transport/auth helpers, but its provider adapter and product semantics remain separate from Threads.

## Deferred decisions

- next provider after the Meta proof;
- public crate names and crates.io publication policy;
- daemon, Unix socket, or HTTP runtime transport;
- delivery concurrency and retry semantics;
- optional developer CLI.

See [`architecture.md`](architecture.md), [`content-variants.md`](content-variants.md), [`engineering-principles.md`](engineering-principles.md), [`protocol.md`](protocol.md), and [`roadmap.md`](roadmap.md).

<!-- © 2026 aiaiaiai · aiaiaiai.org -->
