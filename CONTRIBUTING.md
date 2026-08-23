# Contributing

Prism is contract-first. A change is complete only when its ownership boundary,
failure behavior, tests, and public contract are clear.

## Local checks

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo run -p xtask -- check
```

Run `cargo run -p xtask -- write` only when a deliberate protocol change should
update committed JSON Schemas. Review the resulting diff and add a migration
note for every breaking pre-1.0 contract change.

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

<!-- © 2026 aiaiaiai · aiaiaiai.org -->
