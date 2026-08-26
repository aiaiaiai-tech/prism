// © 2026 aiaiaiai · aiaiaiai.org
// SPDX-License-Identifier: Apache-2.0

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    DispatchPolicy, Extensions, ProviderId, RequestId, TargetId, ValidationIssue, VariantId,
};

/// Stable provider failure class used across all adapters.
#[derive(Clone, Copy, Debug, Eq, JsonSchema, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeliveryErrorClass {
    /// Deterministic request or capability failure.
    InvalidRequest,
    /// Transient provider or transport failure.
    Retryable,
    /// Provider rate limit with optional retry guidance.
    RateLimited,
    /// The external action may have succeeded; retry requires reconciliation.
    OutcomeUnknown,
    /// Missing, expired, or insufficient authorization.
    AuthRequired,
    /// Provider understood but rejected the request.
    ProviderRejected,
    /// Non-retryable provider or adapter failure.
    Terminal,
}

/// Typed, redacted error for one target.
#[derive(Clone, Debug, Eq, JsonSchema, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeliveryError {
    /// Stable cross-provider class.
    pub class: DeliveryErrorClass,
    /// Stable adapter or Prism code.
    pub code: String,
    /// Safe message that contains no credentials or raw upstream payload.
    pub message: String,
    /// Suggested delay for a rate limit or transient failure.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retry_after_seconds: Option<u64>,
    /// Namespaced, explicitly safe recovery metadata.
    #[serde(default, skip_serializing_if = "Extensions::is_empty")]
    pub details: Extensions,
}

impl DeliveryError {
    /// Creates a typed redacted delivery error.
    #[must_use]
    pub fn new(
        class: DeliveryErrorClass,
        code: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            class,
            code: code.into(),
            message: message.into(),
            retry_after_seconds: None,
            details: Extensions::new(),
        }
    }

    /// Adds retry guidance.
    #[must_use]
    pub const fn with_retry_after(mut self, seconds: u64) -> Self {
        self.retry_after_seconds = Some(seconds);
        self
    }

    /// Adds explicitly safe, namespaced recovery metadata.
    #[must_use]
    pub fn with_detail(mut self, key: crate::NamespacedKey, value: serde_json::Value) -> Self {
        self.details.insert(key, value);
        self
    }
}

/// Safe provider receipt. Raw upstream responses do not cross this boundary.
#[derive(Clone, Debug, Eq, JsonSchema, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProviderReceipt {
    /// Provider-native content identifier.
    pub external_id: String,
    /// Optional canonical public URL.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub external_url: Option<String>,
    /// Namespaced, explicitly safe receipt metadata.
    #[serde(default, skip_serializing_if = "Extensions::is_empty")]
    pub details: Extensions,
}

/// Requested execution operation.
#[derive(Clone, Copy, Debug, Eq, JsonSchema, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionMode {
    /// Preflight only; never performs an external action.
    Validate,
    /// Preflight followed by explicit provider dispatch.
    Publish,
}

/// Outcome state for one target.
#[derive(Clone, Copy, Debug, Eq, JsonSchema, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TargetStatus {
    /// Target passed core and adapter preflight.
    Validated,
    /// Provider returned a safe receipt after dispatch.
    Published,
    /// Target was valid but policy prevented dispatch.
    Skipped,
    /// Target failed preflight or dispatch.
    Failed,
}

/// Independent result for one target, kept in request order.
#[derive(Clone, Debug, Eq, JsonSchema, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TargetOutcome {
    /// Request-local target identifier.
    pub target_id: TargetId,
    /// Provider identity.
    pub provider_id: ProviderId,
    /// Variant selected during deterministic resolution.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selected_variant_id: Option<VariantId>,
    /// Current terminal outcome for this execution.
    pub status: TargetStatus,
    /// Safe provider receipt for a successful publish.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub receipt: Option<ProviderReceipt>,
    /// Typed failure for a failed or skipped target.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<DeliveryError>,
    /// Structured blocking issues and non-blocking warnings from preflight.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub validation_issues: Vec<ValidationIssue>,
}

/// Deterministic execution event kind.
#[derive(Clone, Copy, Debug, Eq, JsonSchema, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventKind {
    /// Variant and adapter were resolved.
    TargetResolved,
    /// Provider capabilities were loaded.
    CapabilitiesLoaded,
    /// Target passed preflight.
    TargetValidated,
    /// Target failed preflight.
    TargetRejected,
    /// Policy prevented an otherwise valid dispatch.
    DispatchSkipped,
    /// Adapter dispatch is about to cross the external-action boundary.
    DispatchStarted,
    /// Provider returned a safe receipt.
    Published,
    /// Provider dispatch failed.
    DeliveryFailed,
}

/// Sequence-numbered domain event with no wall-clock dependency.
#[derive(Clone, Debug, Eq, JsonSchema, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionEvent {
    /// One-based stable sequence within the report.
    pub sequence: u32,
    /// Event kind.
    pub kind: EventKind,
    /// Target associated with the event.
    pub target_id: TargetId,
    /// Provider associated with the event.
    pub provider_id: ProviderId,
    /// Optional stable failure or decision code.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
}

/// Complete deterministic result for a validation or publish operation.
#[derive(Clone, Debug, Eq, JsonSchema, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionReport {
    /// Caller correlation identifier.
    pub request_id: RequestId,
    /// Operation that produced the report.
    pub mode: ExecutionMode,
    /// Partial-publication policy applied by the executor.
    pub dispatch_policy: DispatchPolicy,
    /// Independent results in request target order.
    pub outcomes: Vec<TargetOutcome>,
    /// Deterministic event stream.
    pub events: Vec<ExecutionEvent>,
}
