# Ecosystem boundaries

Prism is one repository in a larger product family. Repository boundaries are
semantic ownership boundaries, not merely deployment choices.

| Component | Owns | Integrates through |
| --- | --- | --- |
| `prism` | Deterministic execution, provider contract, wire protocol | Rust APIs and `prism-execution.v1` |
| `prism-hub` | Workspaces, identities, channels, OAuth lifecycle, scheduling, persistence policy, approvals, API | `prism-execution.v1` |
| `prism-bot` | Modular Telegram bot/Mini App and future messaging surfaces | Generated hub API client |
| `prism-panel` | Web administration, onboarding, configuration, and operational UX | Generated hub API client |
| `prism-ai` | Optional explicit variant production | Public content contracts and a hub extension boundary |
| HQBase | Independent mail product | HQBase public Mail API |
| infrastructure authority | Runtime hosts, secrets wiring, databases, storage, DNS, TLS, backups | Immutable application artifacts |

## Allowed direction

- bot and panel clients depend on `prism-hub`, never on Prism, AI, or provider
  APIs;
- `prism-hub` invokes Prism, never the reverse;
- `prism-hub` invokes `prism-ai` through an optional generation port;
- `prism-ai` returns explicit `ContentVariant` values with provenance before
  delivery and never invokes Prism;
- infrastructure consumes versioned artifacts and documented runtime contracts;
- HQBase integration belongs to a hub adapter using its public API.

## Forbidden coupling

- Prism importing client, hub, AI, HQBase, or infrastructure internals;
- a shared mutable application database between products;
- raw access to another product's tables or browser cookies;
- provider tokens in bot or panel clients;
- product-specific concepts such as animals, donations, expenses, or campaigns
  in `prism-core`;
- infrastructure patching application source during deployment.

## Localization

Prism has no canonical language. A Ukrainian variant is not derived from an
English source unless an external producer explicitly creates it that way.

`uk-UA` is a BCP-47 locale. `uk_SP` is a voice/style profile and therefore a
separate reference. Audience geography is separate again. This lets one locale
have several voices or audiences without inventing invalid language tags.

## Persistence

Persistence is a `prism-hub` policy, not a prerequisite for publishing. A
personal installation may discard content after delivery while retaining only
minimal delivery metadata. An organization may opt into durable drafts, media,
receipts, metrics, and reporting. Both use identical Prism semantics.

## Deployment profiles

- **direct stateless:** Prism runtime with no database;
- **self-hosted hub:** hub, runtime, PostgreSQL, and optional object storage;
- **aiaiaiai-managed:** the same versioned artifacts with managed wiring;
- **dedicated client:** the same artifacts in isolated infrastructure.

No profile may require aiaiaiai infrastructure for correctness.

Only the `prism` execution foundation and first Threads provider proof are
implemented today. Public repository shells for `prism-hub`, `prism-ai`, and
`prism-bot` do not yet constitute shipped services. `prism-panel` is planned;
native variants including `prism-panel-ios` are outside the current scope. See
[`status.md`](status.md) for implementation state and exit criteria, and
[`engineering-principles.md`](engineering-principles.md) for the normative
SOLID/OOP boundary.

<!-- © 2026 aiaiaiai · aiaiaiai.org -->
