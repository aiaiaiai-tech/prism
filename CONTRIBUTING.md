# Contributing

Prism is contract-first. A change is complete only when its ownership boundary,
failure behavior, tests, and public contract are clear.

The [engineering principles](docs/engineering-principles.md) are mandatory.
Reviewers must reject changes that violate SOLID, hide dependencies, replace
cohesive object boundaries with procedural orchestration, or introduce
speculative support for out-of-scope native panel clients. Clean Architecture
governs application boundaries; MVVM may organize panel presentation state but
does not replace those boundaries.

## Local checks

```bash
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets --all-features -- -D warnings
cargo test --locked --workspace --all-features
cargo run --locked -p xtask -- check
```

Run `cargo run -p xtask -- write` only when a deliberate protocol change should
update committed JSON Schemas. Review the resulting diff and add a migration
note for every breaking pre-1.0 contract change.

Full CI also checks all targets and features with the declared Rust 1.85 MSRV.
The same `xtask check` command validates repository copyright notices. New
authored Rust, TOML, and YAML files must use the canonical aiaiaiai signature
and the repository-approved SPDX identifier. Generated schemas, JSON fixtures,
lockfiles, and third-party material follow the exceptions in
[`docs/licensing.md`](docs/licensing.md).

## Pull requests

- branch from `master` using `feature/` or `fix/`;
- keep one coherent task per branch and pull request;
- open as Draft and require Full CI to pass;
- do not add live provider calls to required CI;
- do not mix merge with deployment;
- do not commit tokens, provider payloads containing personal data, or captured
  live responses that have not been reduced to safe fixtures.

Provider adapters must keep HTTP, OAuth, and provider-native error semantics
inside the adapter. Core types may grow only when the concept is genuinely
provider-neutral.

Every material pull request must identify the owning object/module, its focused
port, the injected implementations, and the contract tests that prove
substitutability. Prefer composition over inheritance and reject central
provider, channel, or vendor switches.

<!-- © 2026 aiaiaiai · aiaiaiai.org -->
