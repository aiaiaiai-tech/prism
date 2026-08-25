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
live Threads call. `.github/workflows/threads-live.yml` is a separate manual
workflow because publishing is an irreversible external action. It runs the
same adapter through the complete Prism executor and emits only the redacted
execution report.

### Controlled live proof

The repository must have a protected `threads-live` GitHub Environment with:

- environment variable `THREADS_USER_ID` containing the controlled Threads
  user ID;
- environment secret `THREADS_ACCESS_TOKEN` containing a short-lived user
  token with `threads_basic` and `threads_content_publish`;
- a required reviewer when the repository plan supports environment reviewers.

From the Actions tab, select **Threads Live Validation**, choose **Run
workflow**, provide the public test text, and type the exact confirmation
`PUBLISH_THREADS_TEST`. The workflow cannot run from pull requests, pushes, or
schedules. A successful report must contain one `published` target, the public
post ID, and the safe `meta.threads.container_id`; it must not contain the token
or a raw Meta response.

After the proof, record the workflow URL and public post URL in the pull request.
Remove the short-lived secret from the environment when no further validation
is planned. Token acquisition, refresh, and storage remain outside Prism core.

## Provider references

- [Threads API overview](https://developers.facebook.com/documentation/threads)
- [Threads posts](https://developers.facebook.com/documentation/threads/posts)
- [Threads publishing reference](https://developers.facebook.com/documentation/threads/reference/publishing)
- [Official Meta Postman collection](https://www.postman.com/meta/threads/documentation/dht3nzz/threads-api)

<!-- © 2026 aiaiaiai · aiaiaiai.org -->
