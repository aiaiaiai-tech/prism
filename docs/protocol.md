# `prism-execution.v1`

The execution protocol lets Ruby and other runtimes invoke Prism without
reimplementing its domain or provider logic.

## Envelope

Every request contains:

- `protocol_version` — exactly `prism-execution.v1`;
- `request_id` — caller-supplied correlation reference;
- `operation` — `capabilities`, `validate`, or `publish`;
- `payload` — the operation-specific value.

Every response repeats `protocol_version` and `request_id`, then contains either
an `ok` result or a typed protocol error. Request payloads are never echoed in
errors.

## Transports

`prism-runtime --json` reads one JSON envelope to EOF and emits one response.
The default NDJSON mode reads and emits one envelope per line, allowing a
long-lived local process. Blank lines are ignored.

The runtime guarantees that stdout is machine-only. Structured diagnostics use
stderr and include request metadata, never content or provider secrets.

## Compatibility

Protocol and binary versions are independent. Within pre-1.0 development:

- additive fields must have deterministic defaults;
- removed, renamed, or semantically changed fields require a changelog and
  migration note;
- unknown protocol versions fail before execution;
- committed JSON Schemas are generated from Rust wire types and checked for
  drift in CI.

See `contracts/examples/` for canonical envelopes.

The current implementation supports only the local JSON/NDJSON process
transport. Remote transports, daemon lifecycle, retries, and durable
idempotency are TODO and are not implied by the versioned envelope.

Delivery failures include `outcome_unknown` for a provider call whose external
effect cannot be determined safely. Consumers must not treat it as an ordinary
retryable transport failure. Namespaced `details` may carry redacted recovery
metadata needed for reconciliation.

<!-- © 2026 aiaiaiai · aiaiaiai.org -->
