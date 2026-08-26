# Threads provider

`prism-provider-threads` is Prism's first real provider adapter. Its current scope is intentionally small: non-empty text posts, up to 500 Unicode scalar values, through Meta's official Threads API.

## Boundary

Prism passes only opaque `ChannelRef` and `CredentialRef` values. An injected `ThreadsBindingResolver` resolves them to a Threads user ID and access token inside the adapter.

OAuth authorization, refresh, storage, workspace ownership, and durable account state belong to the control plane, not Prism core.

`ReqwestThreadsTransport`:

- accepts HTTPS API bases only;
- sends the token as bearer authorization;
- creates a text container, then publishes that exact container;
- parses typed IDs and provider error metadata only;
- never exposes Meta's raw error message in Prism results.

## Capabilities

| Capability | Current value |
| --- | --- |
| Format | `post` |
| Text | supported, maximum 500 characters |
| Media | not implemented |
| Provider-specific options | not implemented |
| Native idempotency | unavailable for this publish path |

Threads also supports image, video, and carousel publishing. Prism will advertise those capabilities only after media resolution, processing state, and recovery behavior are implemented and tested.

## Retry safety

Container creation does not publish content. A transient failure at that stage is `retryable`.

The final publish call is different: the post may already be public even if the response is lost or unreadable. In that case Prism returns `outcome_unknown` with the safe `meta.threads.container_id` detail.

The caller must reconcile provider state before retry. Otherwise a retry may create a duplicate post.

Authentication errors, rate limits, and deterministic provider rejections keep stable cross-provider error classes. Raw tokens and upstream response bodies are never returned or logged.

## Verification

Required CI uses injected transports and reduced fixtures. It never performs a live Threads call.

`.github/workflows/threads-live.yml` is a separate manual workflow for controlled live validation. It runs the same adapter through the Prism executor and emits only the redacted execution report.

### Controlled live validation

The protected `threads-live` GitHub Environment needs:

- `THREADS_USER_ID` — controlled Threads user ID;
- `THREADS_ACCESS_TOKEN` — short-lived token with `threads_basic` and `threads_content_publish`;
- a required reviewer when environment reviewers are available.

From GitHub Actions, run **Threads Live Validation**, provide the public test text, and enter `PUBLISH_THREADS_TEST` exactly.

A successful report must contain one `published` target, the public post ID, and `meta.threads.container_id`. It must not contain the token or a raw Meta response.

Record the workflow URL and public post URL as validation evidence. Remove the short-lived secret when no further validation is planned.

## References

- [Threads API overview](https://developers.facebook.com/documentation/threads)
- [Threads posts](https://developers.facebook.com/documentation/threads/posts)
- [Threads publishing reference](https://developers.facebook.com/documentation/threads/reference/publishing)
- [Official Meta Postman collection](https://www.postman.com/meta/threads/documentation/dht3nzz/threads-api)

<!-- © 2026 aiaiaiai · aiaiaiai.org -->
