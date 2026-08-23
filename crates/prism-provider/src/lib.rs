//! Object-safe provider adapter contract and deterministic provider registry.

use std::{collections::BTreeMap, sync::Arc};

use async_trait::async_trait;
use prism_core::{
    ChannelRef, ContentVariant, CredentialRef, DeliveryError, Extensions, IdempotencyScope,
    ProviderCapabilities, ProviderId, ProviderReceipt, PublishTarget, RequestId, ValidationIssue,
};
use thiserror::Error;

/// Fully resolved, owned input passed to one provider adapter.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderPublishRequest {
    /// Correlation identifier for observability.
    pub request_id: RequestId,
    /// Stable logical publication and target idempotency material.
    pub idempotency: IdempotencyScope,
    /// Provider target including opaque channel and credential references.
    pub target: PublishTarget,
    /// Explicit selected variant.
    pub variant: ContentVariant,
}

/// Provider/channel context used for capability discovery.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderTargetContext {
    /// Provider adapter identifier.
    pub provider_id: ProviderId,
    /// Opaque configured channel reference.
    pub channel: ChannelRef,
    /// Optional credential reference.
    pub credential: Option<CredentialRef>,
    /// Provider-scoped options that may affect capabilities.
    pub options: Extensions,
}

impl From<&PublishTarget> for ProviderTargetContext {
    fn from(target: &PublishTarget) -> Self {
        Self {
            provider_id: target.provider_id.clone(),
            channel: target.channel.clone(),
            credential: target.credential.clone(),
            options: target.options.clone(),
        }
    }
}

/// Provider boundary. Implementations own HTTP, OAuth, secret/media resolution,
/// provider-native validation, and safe upstream error mapping.
#[async_trait]
pub trait ProviderAdapter: Send + Sync + 'static {
    /// Stable identifier used by targets and the registry.
    fn provider_id(&self) -> &ProviderId;

    /// Discovers current capabilities for the target context.
    async fn capabilities(
        &self,
        target: &ProviderTargetContext,
    ) -> Result<ProviderCapabilities, DeliveryError>;

    /// Applies provider-native preflight without performing a publish action.
    async fn validate_publish(
        &self,
        request: &ProviderPublishRequest,
    ) -> Result<Vec<ValidationIssue>, DeliveryError>;

    /// Performs the explicit external action and returns a redacted receipt.
    async fn publish(
        &self,
        request: &ProviderPublishRequest,
    ) -> Result<ProviderReceipt, DeliveryError>;
}

/// Provider registry mutation failure.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum RegistryError {
    /// The identifier is already registered.
    #[error("provider `{0}` is already registered")]
    Duplicate(ProviderId),
}

/// Deterministically ordered provider adapter registry.
#[derive(Clone, Default)]
pub struct ProviderRegistry {
    providers: BTreeMap<ProviderId, Arc<dyn ProviderAdapter>>,
}

impl ProviderRegistry {
    /// Creates an empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers an adapter, rejecting accidental replacement.
    pub fn register(&mut self, provider: Arc<dyn ProviderAdapter>) -> Result<(), RegistryError> {
        let provider_id = provider.provider_id().clone();
        if self.providers.contains_key(&provider_id) {
            return Err(RegistryError::Duplicate(provider_id));
        }
        self.providers.insert(provider_id, provider);
        Ok(())
    }

    /// Returns a shared adapter by stable identifier.
    #[must_use]
    pub fn get(&self, provider_id: &ProviderId) -> Option<Arc<dyn ProviderAdapter>> {
        self.providers.get(provider_id).cloned()
    }

    /// Returns registered identifiers in stable order.
    pub fn provider_ids(&self) -> impl Iterator<Item = &ProviderId> {
        self.providers.keys()
    }

    /// Returns whether the registry has no adapters.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.providers.is_empty()
    }
}
