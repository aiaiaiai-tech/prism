# Security policy

## Supported versions

Prism is pre-1.0. Security fixes target the latest commit on `master` and the
latest published pre-release when one exists.

## Reporting

Please report vulnerabilities privately to `security@aiaiaiai.org`. Do not open
a public issue before the impact and remediation path have been assessed.

## Security invariants

- Domain and protocol types carry credential references, never raw credentials.
- Runtime logs metadata only; request payloads are neither logged nor echoed.
- Provider adapters own secret resolution and must redact upstream responses.
- Publishing is an explicit external-action boundary with an idempotency key.
- `outcome_unknown` failures must be reconciled before retrying, preventing an
  ambiguous provider response from becoming an accidental duplicate publish.
- Required CI never performs live provider calls.

<!-- © 2026 aiaiaiai · aiaiaiai.org -->
