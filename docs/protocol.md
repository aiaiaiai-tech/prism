# `prism-execution.v1`

The execution protocol lets Ruby and other runtimes invoke Prism without reimplementing Prism domain or provider behavior.

## Envelope

Every request contains:

- `protocol_version` — exactly `prism-execution.v1`;
- `request_id` — caller-owned correlation reference;
- `operation` — `capabilities`, `validate`, or `publish`;
- `payload` — operation-specific data.

Every response repeats `protocol_version` and `request_id`, then contains either an `ok` result or a typed protocol error. Errors never echo the request payload.

## Transport

`prism-runtime --json` reads one JSON envelope to EOF and emits one response.

The default NDJSON mode reads and emits one envelope per line for a long-lived local process. Blank lines are ignored.

stdout is protocol-only. Diagnostics use stderr and contain request metadata, never content or provider secrets.

## Compatibility

Protocol and binary versions are independent. During pre-1.0 development:

- additive fields need deterministic defaults;
- removed, renamed, or semantically changed fields require a changelog and migration note;
- unknown protocol versions fail before execution;
- committed JSON Schemas are generated from Rust wire types and checked for drift in CI.

See `contracts/examples/` for canonical envelopes.

## Current boundary

Only local JSON/NDJSON process transport is implemented. Remote transport, daemon lifecycle, retries, and durable idempotency are planned, not implied by the protocol version.

Delivery failures may include `outcome_unknown` when Prism cannot safely determine whether an external action succeeded. Consumers must reconcile provider state before retry. Namespaced `details` may carry redacted recovery metadata.

<!-- © 2026 aiaiaiai · aiaiaiai.org -->
