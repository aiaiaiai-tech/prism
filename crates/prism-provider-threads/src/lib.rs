// © 2026 aiaiaiai · aiaiaiai.org
// SPDX-License-Identifier: Apache-2.0

//! Text publishing adapter for the official Threads API.
//!
//! OAuth lifecycle and persistent channel configuration remain outside this
//! crate. A caller injects a [`ThreadsBindingResolver`] that resolves Prism's
//! opaque channel and credential references only inside the adapter boundary.

mod http;

use std::{collections::BTreeSet, fmt, sync::Arc};

use async_trait::async_trait;
use prism_core::{
    AltTextCapabilities, ChannelRef, CredentialRef, DeliveryError, DeliveryErrorClass, Extensions,
    MediaCapabilities, NamespacedKey, ProviderCapabilities, ProviderId, ProviderReceipt,
    PublicationFormat, TextCapabilities, ValidationIssue,
};
use prism_provider::{ProviderAdapter, ProviderPublishRequest, ProviderTargetContext};
use serde_json::json;
use thiserror::Error;

pub use http::{ReqwestThreadsTransport, ThreadsTransportConfigError};

/// Stable Prism provider identifier for Threads.
pub const THREADS_PROVIDER_ID: &str = "meta.threads";

/// Secret Threads user access token with intentionally redacted formatting.
pub struct ThreadsAccessToken(String);

impl ThreadsAccessToken {
    /// Wraps a non-empty access token without exposing it through formatting.
    pub fn new(value: impl Into<String>) -> Result<Self, ThreadsBindingError> {
        let value = value.into();
        if value.is_empty() || value.len() > 8_192 || value.chars().any(char::is_whitespace) {
            return Err(ThreadsBindingError::InvalidAccessToken);
        }
        Ok(Self(value))
    }

    pub(crate) fn expose_secret(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for ThreadsAccessToken {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("ThreadsAccessToken([REDACTED])")
    }
}

/// Validated Threads object identifier returned by the provider.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ThreadsObjectId(String);

impl ThreadsObjectId {
    /// Creates an identifier. Threads object IDs are decimal strings.
    pub fn new(value: impl Into<String>) -> Result<Self, ThreadsBindingError> {
        let value = value.into();
        if value.is_empty()
            || value.len() > 64
            || !value.chars().all(|character| character.is_ascii_digit())
        {
            return Err(ThreadsBindingError::InvalidObjectId);
        }
        Ok(Self(value))
    }

    /// Returns the provider identifier as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Resolved provider binding kept entirely inside the Threads adapter.
pub struct ThreadsBinding {
    user_id: ThreadsObjectId,
    access_token: ThreadsAccessToken,
}

impl ThreadsBinding {
    /// Creates a resolved Threads user/token binding.
    #[must_use]
    pub const fn new(user_id: ThreadsObjectId, access_token: ThreadsAccessToken) -> Self {
        Self {
            user_id,
            access_token,
        }
    }

    /// Returns the resolved Threads user ID.
    #[must_use]
    pub const fn user_id(&self) -> &ThreadsObjectId {
        &self.user_id
    }
}

impl fmt::Debug for ThreadsBinding {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ThreadsBinding")
            .field("user_id", &self.user_id)
            .field("access_token", &self.access_token)
            .finish()
    }
}

/// Invalid provider binding or object identifier.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum ThreadsBindingError {
    /// Token is empty, unreasonably large, or contains control characters.
    #[error("invalid Threads access token")]
    InvalidAccessToken,
    /// Provider object identifier is not a bounded decimal string.
    #[error("invalid Threads object ID")]
    InvalidObjectId,
}

/// Resolves opaque Prism references to a Threads user/token binding.
///
/// Implementations may use environment variables, a secret store, or a hub
/// adapter. They must verify that the channel and credential belong to the same
/// Threads user, preserve that user binding throughout one execution, and
/// return only redacted [`DeliveryError`] values. Token rotation is allowed as
/// long as it cannot change the resolved user.
#[async_trait]
pub trait ThreadsBindingResolver: Send + Sync + 'static {
    /// Resolves one configured channel and credential pair.
    async fn resolve(
        &self,
        channel: &ChannelRef,
        credential: &CredentialRef,
    ) -> Result<ThreadsBinding, DeliveryError>;
}

/// Safe provider transport failure category.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ThreadsTransportErrorKind {
    /// Token is missing, expired, or lacks required scopes.
    Authentication,
    /// Provider rejected the call because a quota is exhausted.
    RateLimited,
    /// Provider returned a deterministic request rejection.
    Rejected,
    /// Transport or provider failure that may be transient.
    Transient,
    /// A successful response could not be interpreted safely.
    InvalidResponse,
}

/// Redacted error returned by a Threads transport implementation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ThreadsTransportError {
    /// Stable category used by the adapter's stage-aware mapping.
    pub kind: ThreadsTransportErrorKind,
    /// Safe stable code without raw provider content.
    pub code: String,
    /// Optional retry guidance from the provider.
    pub retry_after_seconds: Option<u64>,
}

impl ThreadsTransportError {
    /// Creates a redacted transport error.
    #[must_use]
    pub fn new(kind: ThreadsTransportErrorKind, code: impl Into<String>) -> Self {
        Self {
            kind,
            code: code.into(),
            retry_after_seconds: None,
        }
    }

    /// Adds provider retry guidance.
    #[must_use]
    pub const fn with_retry_after(mut self, seconds: u64) -> Self {
        self.retry_after_seconds = Some(seconds);
        self
    }
}

/// Threads publishing transport. Implementations keep raw HTTP and tokens
/// outside provider-neutral Prism contracts.
#[async_trait]
pub trait ThreadsTransport: Send + Sync + 'static {
    /// Creates an unpublished text container.
    async fn create_text_container(
        &self,
        binding: &ThreadsBinding,
        text: &str,
    ) -> Result<ThreadsObjectId, ThreadsTransportError>;

    /// Publishes a previously created container.
    async fn publish_container(
        &self,
        binding: &ThreadsBinding,
        container_id: &ThreadsObjectId,
    ) -> Result<ThreadsObjectId, ThreadsTransportError>;
}

/// Official Threads text publishing adapter.
pub struct ThreadsAdapter {
    provider_id: ProviderId,
    bindings: Arc<dyn ThreadsBindingResolver>,
    transport: Arc<dyn ThreadsTransport>,
}

impl ThreadsAdapter {
    /// Creates an adapter with injected binding and HTTP boundaries.
    #[must_use]
    pub fn new(
        bindings: Arc<dyn ThreadsBindingResolver>,
        transport: Arc<dyn ThreadsTransport>,
    ) -> Self {
        Self {
            provider_id: ProviderId::new(THREADS_PROVIDER_ID)
                .expect("the static Threads provider ID is valid"),
            bindings,
            transport,
        }
    }

    async fn resolve_binding(
        &self,
        request: &ProviderPublishRequest,
    ) -> Result<ThreadsBinding, DeliveryError> {
        let credential = request.target.credential.as_ref().ok_or_else(|| {
            DeliveryError::new(
                DeliveryErrorClass::AuthRequired,
                "threads.credential.required",
                "Threads publishing requires a configured credential reference",
            )
        })?;
        self.bindings
            .resolve(&request.target.channel, credential)
            .await
    }
}

#[async_trait]
impl ProviderAdapter for ThreadsAdapter {
    fn provider_id(&self) -> &ProviderId {
        &self.provider_id
    }

    async fn capabilities(
        &self,
        _target: &ProviderTargetContext,
    ) -> Result<ProviderCapabilities, DeliveryError> {
        let mut extensions = Extensions::new();
        extensions.insert(
            NamespacedKey::new("meta.threads.native_idempotency")
                .expect("static extension key is valid"),
            json!(false),
        );

        Ok(ProviderCapabilities {
            provider_id: self.provider_id.clone(),
            revision: Some("threads-posts.text-v1".to_owned()),
            formats: BTreeSet::from([PublicationFormat::Post]),
            text: TextCapabilities {
                supported: true,
                max_characters: Some(500),
            },
            media: MediaCapabilities {
                supported_kinds: BTreeSet::new(),
                max_items: Some(0),
                mixed_kinds: false,
                alt_text: AltTextCapabilities {
                    supported: false,
                    max_characters: None,
                },
            },
            extensions,
        })
    }

    async fn validate_publish(
        &self,
        request: &ProviderPublishRequest,
    ) -> Result<Vec<ValidationIssue>, DeliveryError> {
        let mut issues = Vec::new();
        if request.target.credential.is_none() {
            issues.push(ValidationIssue::error(
                "threads.credential.required",
                format!("/targets/{}/credential", request.target.id),
                "Threads publishing requires a configured credential reference",
            ));
        }
        for (key, _) in request.target.options.iter() {
            issues.push(ValidationIssue::error(
                "threads.option.unsupported",
                format!("/targets/{}/options/{key}", request.target.id),
                "this Threads adapter version does not implement provider options",
            ));
        }

        if !issues.iter().any(ValidationIssue::is_error) {
            self.resolve_binding(request).await?;
        }
        Ok(issues)
    }

    async fn publish(
        &self,
        request: &ProviderPublishRequest,
    ) -> Result<ProviderReceipt, DeliveryError> {
        let binding = self.resolve_binding(request).await?;
        let text = request
            .variant
            .body
            .text
            .as_deref()
            .filter(|text| !text.trim().is_empty())
            .ok_or_else(|| {
                DeliveryError::new(
                    DeliveryErrorClass::InvalidRequest,
                    "threads.text.required",
                    "the text-only Threads adapter requires non-empty text",
                )
            })?;

        let container_id = self
            .transport
            .create_text_container(&binding, text)
            .await
            .map_err(map_container_creation_error)?;
        let published_id = self
            .transport
            .publish_container(&binding, &container_id)
            .await
            .map_err(|error| map_publish_error(error, &container_id))?;

        let mut details = Extensions::new();
        details.insert(container_id_key(), json!(container_id.as_str()));
        Ok(ProviderReceipt {
            external_id: published_id.as_str().to_owned(),
            external_url: None,
            details,
        })
    }
}

fn map_container_creation_error(error: ThreadsTransportError) -> DeliveryError {
    let class = match error.kind {
        ThreadsTransportErrorKind::Authentication => DeliveryErrorClass::AuthRequired,
        ThreadsTransportErrorKind::RateLimited => DeliveryErrorClass::RateLimited,
        ThreadsTransportErrorKind::Rejected => DeliveryErrorClass::ProviderRejected,
        ThreadsTransportErrorKind::Transient => DeliveryErrorClass::Retryable,
        ThreadsTransportErrorKind::InvalidResponse => DeliveryErrorClass::Terminal,
    };
    mapped_delivery_error(class, error)
}

fn map_publish_error(
    error: ThreadsTransportError,
    container_id: &ThreadsObjectId,
) -> DeliveryError {
    let class = match error.kind {
        ThreadsTransportErrorKind::Authentication => DeliveryErrorClass::AuthRequired,
        ThreadsTransportErrorKind::RateLimited => DeliveryErrorClass::RateLimited,
        ThreadsTransportErrorKind::Rejected => DeliveryErrorClass::ProviderRejected,
        ThreadsTransportErrorKind::Transient | ThreadsTransportErrorKind::InvalidResponse => {
            DeliveryErrorClass::OutcomeUnknown
        }
    };
    mapped_delivery_error(class, error)
        .with_detail(container_id_key(), json!(container_id.as_str()))
}

fn mapped_delivery_error(class: DeliveryErrorClass, error: ThreadsTransportError) -> DeliveryError {
    let mut mapped = DeliveryError::new(
        class,
        error.code,
        match class {
            DeliveryErrorClass::AuthRequired => {
                "Threads authorization is missing, expired, or insufficient"
            }
            DeliveryErrorClass::RateLimited => "Threads publishing is rate limited",
            DeliveryErrorClass::ProviderRejected => "Threads rejected the publishing request",
            DeliveryErrorClass::Retryable => {
                "Threads container creation failed before a public action"
            }
            DeliveryErrorClass::OutcomeUnknown => {
                "Threads publish outcome is unknown; reconcile the container before retrying"
            }
            DeliveryErrorClass::InvalidRequest | DeliveryErrorClass::Terminal => {
                "Threads returned an invalid or unsupported response"
            }
        },
    );
    if let Some(seconds) = error.retry_after_seconds {
        mapped = mapped.with_retry_after(seconds);
    }
    mapped
}

fn container_id_key() -> NamespacedKey {
    NamespacedKey::new("meta.threads.container_id").expect("static extension key is valid")
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use prism_core::{
        ContentBody, ContentVariant, IdempotencyKey, IdempotencyScope, LocaleTag, Provenance,
        ProvenanceKind, PublishTarget, RequestId, TargetId, VariantId, VariantSelection,
        VoiceProfileRef,
    };

    use super::*;

    struct StaticResolver;

    #[async_trait]
    impl ThreadsBindingResolver for StaticResolver {
        async fn resolve(
            &self,
            _channel: &ChannelRef,
            _credential: &CredentialRef,
        ) -> Result<ThreadsBinding, DeliveryError> {
            Ok(ThreadsBinding::new(
                ThreadsObjectId::new("123456789").expect("valid user ID"),
                ThreadsAccessToken::new("top-secret-token").expect("valid token"),
            ))
        }
    }

    struct ScriptedTransport {
        create: Mutex<Option<Result<ThreadsObjectId, ThreadsTransportError>>>,
        publish: Mutex<Option<Result<ThreadsObjectId, ThreadsTransportError>>>,
    }

    #[async_trait]
    impl ThreadsTransport for ScriptedTransport {
        async fn create_text_container(
            &self,
            _binding: &ThreadsBinding,
            _text: &str,
        ) -> Result<ThreadsObjectId, ThreadsTransportError> {
            self.create
                .lock()
                .expect("test transport lock")
                .take()
                .expect("scripted create result")
        }

        async fn publish_container(
            &self,
            _binding: &ThreadsBinding,
            _container_id: &ThreadsObjectId,
        ) -> Result<ThreadsObjectId, ThreadsTransportError> {
            self.publish
                .lock()
                .expect("test transport lock")
                .take()
                .expect("scripted publish result")
        }
    }

    fn adapter(
        create: Result<ThreadsObjectId, ThreadsTransportError>,
        publish: Result<ThreadsObjectId, ThreadsTransportError>,
    ) -> ThreadsAdapter {
        ThreadsAdapter::new(
            Arc::new(StaticResolver),
            Arc::new(ScriptedTransport {
                create: Mutex::new(Some(create)),
                publish: Mutex::new(Some(publish)),
            }),
        )
    }

    fn request() -> ProviderPublishRequest {
        ProviderPublishRequest {
            request_id: RequestId::new("request-1").expect("valid request ID"),
            idempotency: IdempotencyScope {
                root: IdempotencyKey::new("publication-1").expect("valid idempotency key"),
                target_id: TargetId::new("threads-uk").expect("valid target ID"),
            },
            target: PublishTarget {
                id: TargetId::new("threads-uk").expect("valid target ID"),
                provider_id: ProviderId::new(THREADS_PROVIDER_ID).expect("valid provider ID"),
                channel: ChannelRef::new("personal").expect("valid channel"),
                credential: Some(
                    CredentialRef::new("threads.personal").expect("valid credential ref"),
                ),
                selection: VariantSelection::Exact {
                    variant_id: VariantId::new("uk").expect("valid variant ID"),
                },
                options: Extensions::new(),
            },
            variant: ContentVariant {
                id: VariantId::new("uk").expect("valid variant ID"),
                locale: LocaleTag::new("uk-UA").expect("valid locale"),
                voice_profile: Some(
                    VoiceProfileRef::new("0x0sky.threads.uk_SP").expect("valid voice profile"),
                ),
                audience: None,
                provider_target: Some(
                    ProviderId::new(THREADS_PROVIDER_ID).expect("valid provider ID"),
                ),
                format: PublicationFormat::Post,
                body: ContentBody {
                    text: Some("привіт, Threads".to_owned()),
                    media: Vec::new(),
                },
                provenance: Provenance {
                    kind: ProvenanceKind::Human,
                    producer: None,
                    source_refs: Vec::new(),
                },
                extensions: Extensions::new(),
            },
        }
    }

    #[tokio::test]
    async fn capabilities_are_an_explicit_text_only_snapshot() {
        let adapter = adapter(
            Ok(ThreadsObjectId::new("1").expect("valid container ID")),
            Ok(ThreadsObjectId::new("2").expect("valid post ID")),
        );
        let capabilities = adapter
            .capabilities(&ProviderTargetContext::from(&request().target))
            .await
            .expect("capabilities");

        assert_eq!(capabilities.text.max_characters, Some(500));
        assert_eq!(capabilities.media.max_items, Some(0));
        assert!(capabilities.formats.contains(&PublicationFormat::Post));
    }

    #[tokio::test]
    async fn publish_returns_redacted_receipt_metadata() {
        let adapter = adapter(
            Ok(ThreadsObjectId::new("111").expect("valid container ID")),
            Ok(ThreadsObjectId::new("222").expect("valid post ID")),
        );
        let receipt = adapter.publish(&request()).await.expect("publish succeeds");

        assert_eq!(receipt.external_id, "222");
        assert_eq!(
            receipt
                .details
                .iter()
                .next()
                .map(|(_, value)| value.clone()),
            Some(json!("111"))
        );
        assert!(!format!("{receipt:?}").contains("top-secret-token"));
    }

    #[tokio::test]
    async fn ambiguous_publish_is_never_marked_safe_to_retry() {
        let adapter = adapter(
            Ok(ThreadsObjectId::new("111").expect("valid container ID")),
            Err(ThreadsTransportError::new(
                ThreadsTransportErrorKind::Transient,
                "threads.transport",
            )),
        );
        let error = adapter
            .publish(&request())
            .await
            .expect_err("publish fails");

        assert_eq!(error.class, DeliveryErrorClass::OutcomeUnknown);
        assert_eq!(
            error.details.iter().next().map(|(_, value)| value.clone()),
            Some(json!("111"))
        );
    }

    #[tokio::test]
    async fn container_rate_limit_preserves_retry_guidance() {
        let adapter = adapter(
            Err(ThreadsTransportError::new(
                ThreadsTransportErrorKind::RateLimited,
                "threads.api.4",
            )
            .with_retry_after(60)),
            Ok(ThreadsObjectId::new("222").expect("valid post ID")),
        );
        let error = adapter
            .publish(&request())
            .await
            .expect_err("publish fails");

        assert_eq!(error.class, DeliveryErrorClass::RateLimited);
        assert_eq!(error.retry_after_seconds, Some(60));
        assert!(error.details.is_empty());
    }

    #[test]
    fn token_and_binding_debug_output_are_redacted() {
        let binding = ThreadsBinding::new(
            ThreadsObjectId::new("123").expect("valid user ID"),
            ThreadsAccessToken::new("top-secret-token").expect("valid token"),
        );

        assert_eq!(
            format!("{binding:?}"),
            "ThreadsBinding { user_id: ThreadsObjectId(\"123\"), access_token: ThreadsAccessToken([REDACTED]) }"
        );
    }
}
