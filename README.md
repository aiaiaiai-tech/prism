# Prism

Prism is the deterministic publishing foundation of the aiaiaiai ecosystem.
It turns explicit localized content into independently traceable deliveries
without owning accounts, databases, user interfaces, infrastructure, or AI.

The repository is intentionally useful on its own:

- provider capabilities are runtime data, not compile-time assumptions;
- every target selects an explicit content variant;
- unsupported content fails during preflight, before an external action;
- multi-target behavior is governed by an explicit dispatch policy;
- persistence and AI are optional consumers of the public contracts;
- non-Rust processes can use the versioned JSON/NDJSON execution protocol.

## Workspace

| Package | Owns |
| --- | --- |
| `prism-core` | Canonical content, localization, capability, request, result, and error types |
| `prism-provider` | Object-safe asynchronous provider adapter contract and registry |
| `prism-provider-threads` | Official Threads text adapter with injected channel/credential and HTTP boundaries |
| `prism-protocol` | `prism-execution.v1` request/response envelopes and JSON Schema |
| `prism-runtime` | Stateless preflight and dispatch engine plus JSON/NDJSON process boundary |
| `prism-testkit` | Scriptable fake provider and conformance helpers |
| `xtask` | Reproducible contract generation and drift checks |

The first milestone is a working vertical slice rather than an empty crate
skeleton. The first provider proof adds text publishing through the official
Threads API; media and additional providers remain explicit later increments.

## Try the process boundary

The test provider is opt-in and performs no network action:

```bash
cargo run -p prism-runtime --features test-provider -- --json \
  < contracts/examples/publish.request.json
```

For a long-lived process, omit `--json`; stdin and stdout then use one JSON
envelope per line. Protocol output is written only to stdout. Diagnostics use
stderr and never log request payloads.

## Development

```bash
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets --all-features -- -D warnings
cargo test --locked --workspace --all-features
cargo run --locked -p xtask -- check
```

The canonical boundaries are documented in
[`docs/architecture.md`](docs/architecture.md). Contract examples and generated
schemas live in [`contracts/`](contracts/).

## Documentation

| Document | Purpose |
| --- | --- |
| [`implementation status`](docs/status.md) | Exact current behavior, explicit non-features, and TODO exit criteria |
| [`engineering principles`](docs/engineering-principles.md) | Normative SOLID, OOP, Clean Architecture, scope, and review invariants |
| [`architecture`](docs/architecture.md) | Ownership, dependency, determinism, and provider boundaries |
| [`protocol`](docs/protocol.md) | `prism-execution.v1` envelope and transport contract |
| [`ecosystem`](docs/ecosystem.md) | Boundaries between Prism, hub, clients, AI, HQBase, and infrastructure |
| [`Threads provider`](docs/providers/threads.md) | Current live-adapter boundary, capabilities, and retry safety |
| [`roadmap`](docs/roadmap.md) | Evidence-driven implementation order |
| [`copyright and licensing`](docs/licensing.md) | aiaiaiai signature, Apache-2.0 policy, exclusions, and automation |

## Ecosystem boundary

`prism-hub` orchestrates Prism and optional `prism-ai` generation.
`prism-bot` and the planned `prism-panel` are clients of the hub; they never
depend on Prism, AI, or provider APIs directly. Native panel clients, including
`prism-panel-ios`, are outside the current scope. HQBase and the aiaiaiai
infrastructure remain independent peers connected through public, versioned
contracts.

## Status

Pre-1.0 provider proof. The repository has a fake provider and an injectable
Threads text adapter, but no configured credentials, live CI calls, control
plane, client, database, or deployment. See the [implementation
status](docs/status.md) for the precise current/TODO split.

Wire and crate APIs follow semantic versioning; a protocol version is never
inferred from a binary version.

Licensed under Apache-2.0.

<!-- © 2026 aiaiaiai · aiaiaiai.org -->
