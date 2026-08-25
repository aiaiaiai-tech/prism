# Changelog

All notable changes to Prism are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the project uses
semantic versioning.

## [Unreleased]

### Added

- Deterministic content, localization, capability, and delivery domain model.
- Two-phase validation and dispatch with explicit `require_all_valid` and
  `independent` policies.
- Object-safe asynchronous provider contract and scriptable test provider.
- Versioned `prism-execution.v1` JSON/NDJSON process protocol.
- Generated JSON Schemas, golden fixtures, and contract drift checks.
- Canonical aiaiaiai copyright notices and automated policy validation.
- Architecture, ecosystem, current/TODO status, licensing, security, and
  contribution documentation.
- Text-only official Threads adapter with injected channel/credential
  resolution, redacted HTTP transport, capability snapshot, and fixture-based
  provider tests.
- `outcome_unknown` delivery failures for external actions that may have
  succeeded and therefore must not be retried without reconciliation.
- Committed dependency lockfile plus a Rust 1.85 MSRV compatibility gate.

### Changed

- Target idempotency material is now collision-free and explicitly versioned as
  `prism-idempotency.v1`.
- Delivery errors may carry namespaced safe recovery details, such as a Threads
  container ID after an ambiguous publish response.
- Canonical repository metadata now uses `aiaiaiai-org/prism`.

### Migration

- Consumers exhaustively matching `DeliveryErrorClass` must handle
  `outcome_unknown` and prevent automatic retries until provider state is
  reconciled.
- Consumers that persisted the previous `root:target` idempotency string must
  migrate to `prism-idempotency.v1`; protocol versions remain independent from
  crate versions.

<!-- © 2026 aiaiaiai · aiaiaiai.org -->
