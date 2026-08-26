# ADR 0001: Instagram capability boundaries

Status: accepted
Date: 2026-08-26

## Context

The first personal Prism client needs Instagram publishing and short publication metrics without allowing provider credentials, provider SDKs, or transport details to escape the Prism provider boundary.

Prism is the deterministic publishing engine. Prism Hub owns identities, authorisation, provider connections, credential persistence, publication history, and client-facing APIs. Telegram and other clients depend only on the versioned Hub API.

## Decision

### Capability model

Instagram support is exposed as focused provider capabilities rather than one broad Instagram adapter contract:

- `publication.instagram.post`
- `publication.instagram.story`
- `metrics.instagram.publication`
- `credential.resolve`
- `media.resolve`

A provider binding advertises only capabilities that are actually composed and available. Shared publication policy must not branch on provider names to discover or execute these capabilities.

### Credential boundary

Provider access and refresh tokens never appear in client requests, Hub client responses, Hub-to-Prism calls, Prism execution envelopes, publication records, fixtures, or logs.

Prism execution receives an opaque `credential_ref`. A focused `CredentialResolver` port resolves that reference at the provider-adapter boundary. The production resolver is supplied explicitly at the composition root. This boundary applies even when Hub and Prism are composed in-process; transport choice must not weaken the contract.

Secret-bearing values must redact `Debug` and error output. Resolver failures are typed and must not reveal credential material.

### Media boundary

Instagram publication execution receives opaque `media_ref` values rather than raw storage credentials or storage URLs.

A focused `MediaResolver` port resolves media only at the provider-adapter boundary. The resolver owns access to the underlying media source and returns only the data required by the publishing adapter.

### Publishing ownership

`InstagramPostPublisher` and `InstagramStoryPublisher` are separate substitutable adapters behind focused publication capability ports.

They own Instagram-specific validation, transport sequencing, provider response interpretation, and outcome mapping. They do not own workspace policy, OAuth sessions, provider credential persistence, or publication history.

An ambiguous provider result maps to `outcome_unknown`. Automatic retry must never turn an unknown outcome into a fresh publication attempt.

### Metrics ownership

Publication metrics are exposed through a focused `PublicationMetricsReader` capability.

Metric values preserve provider provenance, provider-specific meaning, and observation time. Missing permissions or unsupported metrics map to capability-unavailable states; unavailable values are never represented as zero.

Prism does not persist metric snapshots. Prism Hub owns durable collection, scheduling, and history projection.

## Consequences

- Clients and Hub-facing contracts remain free of raw provider tokens.
- Provider adapters can evolve independently while preserving deterministic Prism contracts.
- Instagram post, story, and metrics support can be composed and tested independently.
- New providers extend registries and adapters instead of central provider conditionals.
- Hub remains the authority for OAuth, encrypted credential storage, media ownership, history, and lifecycle policy.

## Rejected alternatives

### Put Meta tokens in the execution envelope

Rejected because it expands the secret-bearing surface across Hub, transports, logs, fixtures, and clients of the Prism execution contract.

### Put Instagram OAuth in Prism

Rejected because OAuth transactions, user identity, workspace authorisation, and credential persistence belong to Prism Hub rather than the provider execution engine.

### One provider switch for publishing and metrics

Rejected because it violates the open/closed and interface-segregation invariants and couples unrelated capabilities.

## Engineering review gate

1. **Owner** — Instagram transport behaviour is owned by the Instagram provider adapters; credential and media resolution are owned by focused resolver ports.
2. **Boundary** — publication, metrics, `CredentialResolver`, and `MediaResolver` ports separate domain policy from transport and secret-bearing infrastructure.
3. **Substitution** — adapters must preserve validation, failure, redaction, and outcome semantics; contract tests prove those guarantees without live services.
4. **Extension** — providers and capabilities register through composition and registries, not central provider switches.
5. **Dependencies** — provider transports and resolvers are explicit dependencies supplied at the composition root and point inward through ports.
6. **Tests** — unit and contract tests use injected transports and deterministic fake resolvers; required CI performs no live provider calls.

<!-- © 2026 aiaiaiai · aiaiaiai.org -->
