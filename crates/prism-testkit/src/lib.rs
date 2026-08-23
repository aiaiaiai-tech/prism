//! Scriptable fake provider and reusable adapter conformance probes.

use std::{
    collections::VecDeque,
    sync::{Arc, Mutex, MutexGuard},
};

use async_trait::async_trait;
use prism_core::{
    DeliveryError, DeliveryErrorClass, Extensions, ProviderCapabilities, ProviderId,
    ProviderReceipt, RequestId, TargetId, ValidationIssue, VariantId,
};
use prism_provider::{ProviderAdapter, ProviderPublishRequest, ProviderTargetContext};
use thiserror::Error;

/// Snapshot of one fake publish call without content or credentials.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecordedPublish {
    /// Runtime request correlation identifier.
    pub request_id: RequestId,
    /// Target-scoped idempotency material.
    pub idempotency_key: String,
    /// Target identifier.
    pub target_id: TargetId,
    /// Selected variant identifier.
    pub variant_id: VariantId,
}

/// Testkit state access failure.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
#[error("fake provider state is unavailable")]
pub struct FakeProviderStateError;

#[derive(Debug)]
struct FakeState {
    validation_issues: Vec<ValidationIssue>,
    validation_error: Option<DeliveryError>,
    capability_error: Option<DeliveryError>,
    publish_results: VecDeque<Result<ProviderReceipt, DeliveryError>>,
    recorded: Vec<RecordedPublish>,
}

/// Deterministic, no-network provider used by tests and explicit local demos.
#[derive(Clone, Debug)]
pub struct FakeProvider {
    provider_id: ProviderId,
    capabilities: ProviderCapabilities,
    state: Arc<Mutex<FakeState>>,
}

impl FakeProvider {
    /// Creates a fake provider. The capability provider ID is normalized to the
    /// adapter ID so fixtures cannot accidentally test a mismatched snapshot.
    #[must_use]
    pub fn new(provider_id: ProviderId, mut capabilities: ProviderCapabilities) -> Self {
        capabilities.provider_id = provider_id.clone();
        Self {
            provider_id,
            capabilities,
            state: Arc::new(Mutex::new(FakeState {
                validation_issues: Vec::new(),
                validation_error: None,
                capability_error: None,
                publish_results: VecDeque::new(),
                recorded: Vec::new(),
            })),
        }
    }

    /// Replaces provider-native validation issues.
    pub fn set_validation_issues(
        &self,
        issues: Vec<ValidationIssue>,
    ) -> Result<(), FakeProviderStateError> {
        self.state()?.validation_issues = issues;
        Ok(())
    }

    /// Configures a request-level capability discovery failure.
    pub fn set_capability_error(
        &self,
        error: Option<DeliveryError>,
    ) -> Result<(), FakeProviderStateError> {
        self.state()?.capability_error = error;
        Ok(())
    }

    /// Configures a provider-native validation failure.
    pub fn set_validation_error(
        &self,
        error: Option<DeliveryError>,
    ) -> Result<(), FakeProviderStateError> {
        self.state()?.validation_error = error;
        Ok(())
    }

    /// Adds one scripted publish result. Results are consumed in FIFO order.
    pub fn enqueue_publish_result(
        &self,
        result: Result<ProviderReceipt, DeliveryError>,
    ) -> Result<(), FakeProviderStateError> {
        self.state()?.publish_results.push_back(result);
        Ok(())
    }

    /// Returns safe recorded calls.
    pub fn recorded_publishes(&self) -> Result<Vec<RecordedPublish>, FakeProviderStateError> {
        Ok(self.state()?.recorded.clone())
    }

    /// Returns how many publish actions crossed the fake adapter boundary.
    pub fn publish_count(&self) -> Result<usize, FakeProviderStateError> {
        Ok(self.state()?.recorded.len())
    }

    fn state(&self) -> Result<MutexGuard<'_, FakeState>, FakeProviderStateError> {
        self.state.lock().map_err(|_| FakeProviderStateError)
    }

    fn poisoned_delivery_error() -> DeliveryError {
        DeliveryError::new(
            DeliveryErrorClass::Terminal,
            "testkit.state_unavailable",
            "fake provider state is unavailable",
        )
    }
}

#[async_trait]
impl ProviderAdapter for FakeProvider {
    fn provider_id(&self) -> &ProviderId {
        &self.provider_id
    }

    async fn capabilities(
        &self,
        _target: &ProviderTargetContext,
    ) -> Result<ProviderCapabilities, DeliveryError> {
        let state = self.state().map_err(|_| Self::poisoned_delivery_error())?;
        if let Some(error) = &state.capability_error {
            return Err(error.clone());
        }
        Ok(self.capabilities.clone())
    }

    async fn validate_publish(
        &self,
        _request: &ProviderPublishRequest,
    ) -> Result<Vec<ValidationIssue>, DeliveryError> {
        let state = self.state().map_err(|_| Self::poisoned_delivery_error())?;
        if let Some(error) = &state.validation_error {
            return Err(error.clone());
        }
        Ok(state.validation_issues.clone())
    }

    async fn publish(
        &self,
        request: &ProviderPublishRequest,
    ) -> Result<ProviderReceipt, DeliveryError> {
        let mut state = self.state().map_err(|_| Self::poisoned_delivery_error())?;
        state.recorded.push(RecordedPublish {
            request_id: request.request_id.clone(),
            idempotency_key: request.idempotency.stable_string(),
            target_id: request.target.id.clone(),
            variant_id: request.variant.id.clone(),
        });

        state.publish_results.pop_front().unwrap_or_else(|| {
            Ok(ProviderReceipt {
                external_id: format!(
                    "fake:{}:{}",
                    self.provider_id,
                    request.idempotency.stable_string()
                ),
                external_url: None,
                details: Extensions::new(),
            })
        })
    }
}

/// Reusable provider contract violation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConformanceViolation {
    /// Stable violation code.
    pub code: &'static str,
    /// Safe explanation.
    pub message: &'static str,
}

/// Probes identity and capability consistency without publishing.
pub async fn probe_adapter(
    adapter: &dyn ProviderAdapter,
    context: &ProviderTargetContext,
) -> Vec<ConformanceViolation> {
    let mut violations = Vec::new();
    if adapter.provider_id() != &context.provider_id {
        violations.push(ConformanceViolation {
            code: "adapter.provider_id.context_mismatch",
            message: "adapter identity differs from the requested provider context",
        });
        return violations;
    }

    if let Ok(capabilities) = adapter.capabilities(context).await {
        if capabilities.provider_id != *adapter.provider_id() {
            violations.push(ConformanceViolation {
                code: "adapter.provider_id.capability_mismatch",
                message: "capability snapshot identity differs from the adapter identity",
            });
        }
    }
    violations
}
