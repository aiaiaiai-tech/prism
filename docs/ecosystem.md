# Ecosystem boundaries

Prism is one product in a larger family. Repository boundaries are ownership boundaries, not deployment choices.

| Component | Owns | Integrates through |
| --- | --- | --- |
| `prism` | Deterministic publishing, provider adapters, execution protocol | Rust APIs and `prism-execution.v1` |
| `prism-hub` | Workspaces, identities, channels, OAuth lifecycle, scheduling, persistence, approvals, external API | `prism-execution.v1` |
| `prism-bot` | Telegram and future messaging clients | Generated hub API client |
| `prism-panel` | Planned web administration client | Generated hub API client |
| `prism-ai` | Optional content-variant generation | Hub-owned generation port |
| HQBase | Independent mail product | HQBase public Mail API |
| infrastructure | Hosts, secrets wiring, databases, storage, DNS, TLS, backups | Versioned application artifacts |

## Dependency direction

- Clients depend on `prism-hub`, never on Prism, `prism-ai`, or provider APIs.
- `prism-hub` invokes Prism through the versioned execution boundary.
- `prism-hub` may invoke `prism-ai` through an optional generation port.
- `prism-ai` returns explicit `ContentVariant` values with provenance and never publishes directly.
- Infrastructure consumes versioned artifacts and documented runtime contracts.
- HQBase integration belongs in a hub adapter using the public HQBase API.

## Human identity notation

Prism does not own human-identifier syntax or allocation. If a Hub or client uses nilx.one-style `0x` public identity notation, its canonical meaning is defined by [`0x1` Identity — `0x` notation](https://github.com/nilx-one/0x1/blob/master/documents/04-identity.md#0x-notation). Prism must not hard-code a concrete identity such as `0x0sky` or reinterpret the `0x` prefix locally.

## Forbidden coupling

- Prism importing hub, client, ai, HQBase, or infrastructure internals;
- shared mutable application databases between products;
- direct access to another product's tables or browser cookies;
- provider tokens in bot or panel clients;
- client-domain concepts such as animals, donations, expenses, or campaigns in `prism-core`;
- infrastructure patching application source during deployment.

## Content variants

Locale, voice profile, audience, provider target, and publication format are independent dimensions. Prism has no canonical language and never silently infers one dimension from another.

See [`content-variants.md`](content-variants.md).

## Persistence

Persistence is a `prism-hub` policy, not a Prism requirement. A personal deployment may keep only delivery metadata. An organization may keep drafts, media, receipts, metrics, and reporting. Both use the same Prism publishing semantics.

## Deployment profiles

- **direct stateless** — Prism runtime without a database;
- **self-hosted hub** — hub, runtime, PostgreSQL, and optional object storage;
- **aiaiaiai-managed** — the same versioned artifacts with managed infrastructure;
- **dedicated client** — the same artifacts in isolated infrastructure.

No deployment profile may require aiaiaiai infrastructure for Prism correctness.

Implementation state for peer products is tracked in their own repositories. Prism's own implemented/planned boundary lives in [`status.md`](status.md).

Native panel applications, including `prism-panel-ios`, remain outside the current scope. They require a separate product and architecture decision.

<!-- © 2026 aiaiaiai · aiaiaiai.org -->
