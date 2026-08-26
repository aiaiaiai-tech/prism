# Contributing

Prism is contract-first. A change is ready only when ownership, failure behavior, tests, and public contracts are clear.

The [engineering principles](docs/engineering-principles.md) are normative.

## Local checks

```bash
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets --all-features -- -D warnings
cargo test --locked --workspace --all-features
cargo run --locked -p xtask -- check
```

Run `cargo run -p xtask -- write` only for an intentional protocol change. Review generated schema diffs and add a migration note for every breaking pre-1.0 contract change.

Full CI also checks the declared Rust 1.85 minimum supported version. `xtask check` validates generated contracts and repository copyright rules.

## Pull requests

- branch from `master` with `feature/` or `fix/`;
- keep one coherent task per branch and pull request;
- open as Draft and require Full CI to pass;
- never add live provider calls to required CI;
- keep merge and deployment separate;
- never commit tokens, unsafe provider payloads, or unreduced live responses.

Provider adapters own provider HTTP, OAuth, native validation, and native error mapping. Provider-neutral behavior belongs in core contracts.

Every material pull request must identify the owning object or module, the focused port, the injected implementations, and the tests that prove substitution.

## Documentation style

Human-facing documentation uses en_SV:

- plain English and short paragraphs;
- sentence-case headings and direct verbs;
- exact repository, type, field, and protocol identifiers;
- `ai` in prose and `prism-ai` for the repository;
- `content variant`, `voice profile`, `provider adapter`, and `control plane` as canonical terms;
- `current`, `planned`, or `not implemented` instead of ambiguous roadmap shorthand;
- current behavior and planned behavior must never be presented as the same thing.

Prefer the shortest wording that preserves the contract. Do not reintroduce retired aliases or terminology from old plans.

## Copyright

New authored Rust, TOML, and YAML files use the canonical aiaiaiai signature and repository-approved SPDX identifier. Generated schemas, JSON fixtures, lockfiles, and third-party material follow [`docs/licensing.md`](docs/licensing.md).

<!-- © 2026 aiaiaiai · aiaiaiai.org -->
