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
| `prism-protocol` | `prism-execution.v1` request/response envelopes and JSON Schema |
| `prism-runtime` | Stateless preflight and dispatch engine plus JSON/NDJSON process boundary |
| `prism-testkit` | Scriptable fake provider and conformance helpers |
| `xtask` | Reproducible contract generation and drift checks |

The first milestone is a working vertical slice rather than an empty crate
skeleton. Real network adapters deliberately come later, after the contract is
proven by the fake provider and golden protocol tests.

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
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo run -p xtask -- check
```

The canonical boundaries are documented in
[`docs/architecture.md`](docs/architecture.md). Contract examples and generated
schemas live in [`contracts/`](contracts/).

## Ecosystem boundary

`prism-hub`, `prism-web`, `prism-telegram`, `prism-cli`, and `prism-ai` are
consumers or orchestrators. They do not redefine delivery truth. HQBase and the
aiaiaiai infrastructure remain independent peers connected through public,
versioned contracts.

## Status

Pre-1.0 foundation. Wire and crate APIs follow semantic versioning; a protocol
version is never inferred from a binary version.

Licensed under Apache-2.0.
