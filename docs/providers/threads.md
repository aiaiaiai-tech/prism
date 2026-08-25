# Threads provider

`prism-provider-threads` is Prism's first real provider proof. Its current scope
is deliberately narrow: a non-empty text post, up to 500 Unicode scalar values,
published through Meta's official Threads API.

## Boundary

The adapter receives only opaque `ChannelRef` and `CredentialRef` values from
Prism. An injected `ThreadsBindingResolver` maps that pair to a Threads user ID
and access token inside the adapter. OAuth authorization, refresh, storage, and
workspace ownership remain application/control-plane responsibilities.

`ReqwestThreadsTransport`:

- accepts only HTTPS API bases;
- sends the access token as bearer authorization;
- creates a text container, then publishes that exact container;
- parses only typed IDs and provider error metadata;
- never carries Meta's raw error message into Prism results.

## Capability snapshot

| Capability | Current adapter value |
| --- | --- |
| Format | `post` |
| Text | supported, maximum 500 characters |
| Media | not yet supported |
| Provider options | rejected until individually implemented |
| Native idempotency | unavailable for the implemented path |

The snapshot is intentionally narrower than the full Threads platform. The
official API also supports image, video, and carousel publishing, but Prism will
advertise those capabilities only after media resolution, processing-state, and
recovery behavior are implemented and tested.

## External-action safety

Container creation does not make content public. A transient failure during
that stage is `retryable`. The final publish call does cross the public-action
boundary; if its response is transient or unreadable, Prism returns
`outcome_unknown` with the safe `meta.threads.container_id` detail. A caller must
query/reconcile that container before retrying, otherwise it could publish a
duplicate.

Authentication errors, rate limits, and deterministic provider rejections keep
their stable cross-provider classes. Raw tokens and upstream response bodies are
never returned or logged.

## Verification policy

Required CI uses injected transports and reduced error fixtures. It performs no
live Threads call. A future controlled-account workflow must remain explicit and
manual because publishing is an irreversible external action.

## Provider references

- [Threads API overview](https://developers.facebook.com/documentation/threads)
- [Threads posts](https://developers.facebook.com/documentation/threads/posts)
- [Threads publishing reference](https://developers.facebook.com/documentation/threads/reference/publishing)
- [Official Meta Postman collection](https://www.postman.com/meta/threads/documentation/dht3nzz/threads-api)

<!-- © 2026 aiaiaiai · aiaiaiai.org -->
