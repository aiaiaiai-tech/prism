# Security policy

## Supported versions

Prism is pre-1.0. Security fixes target the latest `master` and the latest published pre-release when one exists.

## Reporting

Report vulnerabilities privately to `security@aiaiaiai.org`. Do not open a public issue before impact and remediation have been assessed.

## Security invariants

- Domain and protocol values carry credential references, never raw credentials.
- `prism-runtime` logs metadata only; it never logs or echoes request payloads.
- Provider adapters own secret resolution and redact upstream responses.
- Publishing is an explicit external-action boundary with an idempotency key.
- `outcome_unknown` must be reconciled before retry so an ambiguous provider response cannot become an accidental duplicate.
- Required CI never performs live provider calls.

<!-- © 2026 aiaiaiai · aiaiaiai.org -->
