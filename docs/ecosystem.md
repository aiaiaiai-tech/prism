# Ecosystem boundaries

Prism is one repository in a larger product family. Repository boundaries are
semantic ownership boundaries, not merely deployment choices.

| Component | Owns | Integrates through |
| --- | --- | --- |
| `prism` | Deterministic execution, provider contract, wire protocol | Rust APIs and `prism-execution.v1` |
| `prism-hub` | Workspaces, identities, channels, OAuth lifecycle, scheduling, persistence policy, approvals, API | `prism-execution.v1` |
| `prism-cli` | Direct operator/developer UX and hub API UX | Prism public APIs and hub OpenAPI |
| `prism-web` | Browser interaction | Generated hub API client |
| `prism-telegram` | Telegram bot and Mini App interaction | Hub API |
| `prism-ai` | Optional explicit variant production | Public content contracts and a hub extension boundary |
| HQBase | Independent mail product | HQBase public Mail API |
| infrastructure authority | Runtime hosts, secrets wiring, databases, storage, DNS, TLS, backups | Immutable application artifacts |

## Allowed direction

- clients depend on `prism-hub`, never on provider APIs;
- `prism-hub` invokes Prism, never the reverse;
- `prism-ai` returns explicit `ContentVariant` values before delivery;
- infrastructure consumes versioned artifacts and documented runtime contracts;
- HQBase integration belongs to a hub adapter using its public API.

## Forbidden coupling

- Prism importing client, hub, AI, HQBase, or infrastructure internals;
- a shared mutable application database between products;
- raw access to another product's tables or browser cookies;
- provider tokens in web or Telegram clients;
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

Only the `prism` execution foundation exists in this repository today. The hub,
clients, AI component, peer integrations, and deployment profiles in this
document are architectural boundaries and TODO targets, not shipped services.
See [`status.md`](status.md) for implementation state and exit criteria.

<!-- © 2026 aiaiaiai · aiaiaiai.org -->
