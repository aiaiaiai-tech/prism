//! Stateless two-phase Prism executor and protocol service.

use std::sync::Arc;

use prism_core::{
    DeliveryError, DeliveryErrorClass, DispatchPolicy, EventKind, ExecutionEvent, ExecutionMode,
    ExecutionReport, ProviderCapabilities, ProviderId, RequestId, TargetOutcome, TargetStatus,
    ValidationIssue,
};
use prism_protocol::{
    ExecutionCommand, ExecutionResult, PROTOCOL_VERSION, ProtocolError, ProtocolErrorCode,
    RequestEnvelope, ResponseEnvelope,
};
use prism_provider::{
    ProviderAdapter, ProviderPublishRequest, ProviderRegistry, ProviderTargetContext,
};
use thiserror::Error;

/// Request-level execution failure. Target-local failures remain in reports.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum ExecutionFailure {
    /// Deterministic request structure is invalid.
    #[error("request failed structural validation")]
    InvalidRequest {
        /// Structured issues.
        issues: Vec<ValidationIssue>,
    },
    /// The requested provider adapter is absent.
    #[error("provider `{0}` is not registered")]
    ProviderNotFound(ProviderId),
    /// Capability discovery failed for a direct capabilities operation.
    #[error("capability discovery failed")]
    CapabilityDiscovery(DeliveryError),
    /// Adapter violated a Prism identity contract.
    #[error("provider `{0}` returned an inconsistent capability identity")]
    ProviderContract(ProviderId),
}

struct TargetPlan {
    adapter: Option<Arc<dyn ProviderAdapter>>,
    request: Option<ProviderPublishRequest>,
    outcome: TargetOutcome,
    ready: bool,
}

/// Stateless executor. Long-term idempotency and persistence belong to callers
/// or provider adapters, not this type.
#[derive(Clone)]
pub struct Executor {
    registry: ProviderRegistry,
}

impl Executor {
    /// Creates an executor from an immutable provider registry snapshot.
    #[must_use]
    pub const fn new(registry: ProviderRegistry) -> Self {
        Self { registry }
    }

    /// Discovers capabilities for one provider/channel context.
    pub async fn capabilities(
        &self,
        context: ProviderTargetContext,
    ) -> Result<ProviderCapabilities, ExecutionFailure> {
        let issues = validate_capability_context(&context);
        if !issues.is_empty() {
            return Err(ExecutionFailure::InvalidRequest { issues });
        }
        let adapter = self
            .registry
            .get(&context.provider_id)
            .ok_or_else(|| ExecutionFailure::ProviderNotFound(context.provider_id.clone()))?;
        let capabilities = adapter
            .capabilities(&context)
            .await
            .map_err(ExecutionFailure::CapabilityDiscovery)?;
        if capabilities.provider_id != context.provider_id {
            return Err(ExecutionFailure::ProviderContract(context.provider_id));
        }
        Ok(capabilities)
    }

    /// Runs all preflight stages without crossing a publish boundary.
    pub async fn validate(
        &self,
        request_id: RequestId,
        request: prism_core::PublishRequest,
    ) -> Result<ExecutionReport, ExecutionFailure> {
        self.execute(request_id, request, ExecutionMode::Validate).await
    }

    /// Runs preflight for every target, then dispatches according to policy.
    pub async fn publish(
        &self,
        request_id: RequestId,
        request: prism_core::PublishRequest,
    ) -> Result<ExecutionReport, ExecutionFailure> {
        self.execute(request_id, request, ExecutionMode::Publish).await
    }

    async fn execute(
        &self,
        request_id: RequestId,
        request: prism_core::PublishRequest,
        mode: ExecutionMode,
    ) -> Result<ExecutionReport, ExecutionFailure> {
        let structural_issues = request.structural_issues();
        if structural_issues.iter().any(ValidationIssue::is_error) {
            return Err(ExecutionFailure::InvalidRequest {
                issues: structural_issues,
            });
        }

        let mut events = Vec::new();
        let mut plans = Vec::with_capacity(request.targets.len());
        for target in &request.targets {
            plans.push(
                self.preflight_target(&request_id, &request, target, &mut events)
                    .await,
            );
        }

        if mode == ExecutionMode::Publish {
            let any_rejected = plans.iter().any(|plan| !plan.ready);
            let block_all = request.dispatch_policy == DispatchPolicy::RequireAllValid
                && any_rejected;

            for plan in &mut plans {
                if !plan.ready {
                    continue;
                }
                if block_all {
                    plan.outcome.status = TargetStatus::Skipped;
                    plan.outcome.error = Some(DeliveryError::new(
                        DeliveryErrorClass::InvalidRequest,
                        "dispatch.require_all_valid_preflight_failed",
                        "dispatch policy prevented external actions because another target failed preflight",
                    ));
                    push_event(
                        &mut events,
                        EventKind::DispatchSkipped,
                        &plan.outcome,
                        Some("dispatch.require_all_valid_preflight_failed"),
                    );
                    continue;
                }

                push_event(
                    &mut events,
                    EventKind::DispatchStarted,
                    &plan.outcome,
                    None,
                );
                let Some(adapter) = &plan.adapter else {
                    mark_internal_plan_failure(plan, &mut events);
                    continue;
                };
                let Some(provider_request) = &plan.request else {
                    mark_internal_plan_failure(plan, &mut events);
                    continue;
                };
                match adapter.publish(provider_request).await {
                    Ok(receipt) => {
                        plan.outcome.status = TargetStatus::Published;
                        plan.outcome.receipt = Some(receipt);
                        push_event(&mut events, EventKind::Published, &plan.outcome, None);
                    }
                    Err(error) => {
                        let code = error.code.clone();
                        plan.outcome.status = TargetStatus::Failed;
                        plan.outcome.error = Some(error);
                        push_event(
                            &mut events,
                            EventKind::DeliveryFailed,
                            &plan.outcome,
                            Some(&code),
                        );
                    }
                }
            }
        }

        Ok(ExecutionReport {
            request_id,
            mode,
            dispatch_policy: request.dispatch_policy,
            outcomes: plans.into_iter().map(|plan| plan.outcome).collect(),
            events,
        })
    }

    async fn preflight_target(
        &self,
        request_id: &RequestId,
        request: &prism_core::PublishRequest,
        target: &prism_core::PublishTarget,
        events: &mut Vec<ExecutionEvent>,
    ) -> TargetPlan {
        let variant = match request.resolve_variant(target) {
            Ok(variant) => variant.clone(),
            Err(issue) => {
                let outcome = failed_outcome(
                    target,
                    None,
                    DeliveryError::new(
                        DeliveryErrorClass::InvalidRequest,
                        "selection.no_eligible_variant",
                        "target has no eligible content variant",
                    ),
                    vec![issue],
                );
                push_event(
                    events,
                    EventKind::TargetRejected,
                    &outcome,
                    Some("selection.no_eligible_variant"),
                );
                return TargetPlan {
                    adapter: None,
                    request: None,
                    outcome,
                    ready: false,
                };
            }
        };

        let mut outcome = TargetOutcome {
            target_id: target.id.clone(),
            provider_id: target.provider_id.clone(),
            selected_variant_id: Some(variant.id.clone()),
            status: TargetStatus::Validated,
            receipt: None,
            error: None,
            validation_issues: Vec::new(),
        };

        let Some(adapter) = self.registry.get(&target.provider_id) else {
            outcome.status = TargetStatus::Failed;
            outcome.error = Some(DeliveryError::new(
                DeliveryErrorClass::Terminal,
                "provider.not_registered",
                "no adapter is registered for this provider",
            ));
            push_event(
                events,
                EventKind::TargetRejected,
                &outcome,
                Some("provider.not_registered"),
            );
            return TargetPlan {
                adapter: None,
                request: None,
                outcome,
                ready: false,
            };
        };

        push_event(events, EventKind::TargetResolved, &outcome, None);
        let context = ProviderTargetContext::from(target);
        let capabilities = match adapter.capabilities(&context).await {
            Ok(capabilities) if capabilities.provider_id == target.provider_id => capabilities,
            Ok(_) => {
                outcome.status = TargetStatus::Failed;
                outcome.error = Some(DeliveryError::new(
                    DeliveryErrorClass::Terminal,
                    "provider.capability_identity_mismatch",
                    "adapter returned capabilities for a different provider",
                ));
                push_event(
                    events,
                    EventKind::TargetRejected,
                    &outcome,
                    Some("provider.capability_identity_mismatch"),
                );
                return TargetPlan {
                    adapter: Some(adapter),
                    request: None,
                    outcome,
                    ready: false,
                };
            }
            Err(error) => {
                let code = error.code.clone();
                outcome.status = TargetStatus::Failed;
                outcome.error = Some(error);
                push_event(
                    events,
                    EventKind::TargetRejected,
                    &outcome,
                    Some(&code),
                );
                return TargetPlan {
                    adapter: Some(adapter),
                    request: None,
                    outcome,
                    ready: false,
                };
            }
        };
        push_event(events, EventKind::CapabilitiesLoaded, &outcome, None);

        let provider_request = ProviderPublishRequest {
            request_id: request_id.clone(),
            idempotency: request.idempotency_scope(target),
            target: target.clone(),
            variant,
        };

        let mut issues = capabilities.validate_variant(&provider_request.variant);
        if !issues.iter().any(ValidationIssue::is_error) {
            match adapter.validate_publish(&provider_request).await {
                Ok(provider_issues) => issues.extend(provider_issues),
                Err(error) => {
                    let code = error.code.clone();
                    outcome.status = TargetStatus::Failed;
                    outcome.error = Some(error);
                    push_event(
                        events,
                        EventKind::TargetRejected,
                        &outcome,
                        Some(&code),
                    );
                    return TargetPlan {
                        adapter: Some(adapter),
                        request: Some(provider_request),
                        outcome,
                        ready: false,
                    };
                }
            }
        }

        let rejected = issues.iter().any(ValidationIssue::is_error);
        outcome.validation_issues = issues;
        if rejected {
            outcome.status = TargetStatus::Failed;
            outcome.error = Some(DeliveryError::new(
                DeliveryErrorClass::InvalidRequest,
                "target.preflight_failed",
                "target failed capability or provider validation",
            ));
            push_event(
                events,
                EventKind::TargetRejected,
                &outcome,
                Some("target.preflight_failed"),
            );
            TargetPlan {
                adapter: Some(adapter),
                request: Some(provider_request),
                outcome,
                ready: false,
            }
        } else {
            push_event(events, EventKind::TargetValidated, &outcome, None);
            TargetPlan {
                adapter: Some(adapter),
                request: Some(provider_request),
                outcome,
                ready: true,
            }
        }
    }
}

fn failed_outcome(
    target: &prism_core::PublishTarget,
    selected_variant_id: Option<prism_core::VariantId>,
    error: DeliveryError,
    validation_issues: Vec<ValidationIssue>,
) -> TargetOutcome {
    TargetOutcome {
        target_id: target.id.clone(),
        provider_id: target.provider_id.clone(),
        selected_variant_id,
        status: TargetStatus::Failed,
        receipt: None,
        error: Some(error),
        validation_issues,
    }
}

fn push_event(
    events: &mut Vec<ExecutionEvent>,
    kind: EventKind,
    outcome: &TargetOutcome,
    code: Option<&str>,
) {
    let sequence = u32::try_from(events.len().saturating_add(1)).unwrap_or(u32::MAX);
    events.push(ExecutionEvent {
        sequence,
        kind,
        target_id: outcome.target_id.clone(),
        provider_id: outcome.provider_id.clone(),
        code: code.map(str::to_owned),
    });
}

fn mark_internal_plan_failure(plan: &mut TargetPlan, events: &mut Vec<ExecutionEvent>) {
    plan.ready = false;
    plan.outcome.status = TargetStatus::Failed;
    plan.outcome.error = Some(DeliveryError::new(
        DeliveryErrorClass::Terminal,
        "runtime.invalid_execution_plan",
        "runtime could not dispatch a validated target",
    ));
    push_event(
        events,
        EventKind::DeliveryFailed,
        &plan.outcome,
        Some("runtime.invalid_execution_plan"),
    );
}

fn validate_capability_context(context: &ProviderTargetContext) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();
    for (key, _) in context.options.iter() {
        if !key.is_scoped_to(&context.provider_id) {
            issues.push(ValidationIssue::error(
                "target.option.namespace_mismatch",
                format!("/payload/options/{key}"),
                "provider options must use the requested provider namespace",
            ));
        }
        if key.looks_secret_bearing() {
            issues.push(ValidationIssue::error(
                "extension.secret_prohibited",
                format!("/payload/options/{key}"),
                "raw credentials and secret-bearing fields are prohibited in options",
            ));
        }
    }
    issues
}

/// Protocol-facing service around an executor.
#[derive(Clone)]
pub struct ExecutionService {
    executor: Executor,
}

impl ExecutionService {
    /// Creates a service.
    #[must_use]
    pub const fn new(executor: Executor) -> Self {
        Self { executor }
    }

    /// Handles one already-parsed request envelope.
    pub async fn handle(&self, request: RequestEnvelope) -> ResponseEnvelope {
        let request_id = request.request_id.clone();
        if request.protocol_version != PROTOCOL_VERSION {
            return ResponseEnvelope::error(
                request_id,
                ProtocolError::new(
                    ProtocolErrorCode::UnsupportedProtocol,
                    "runtime supports only prism-execution.v1",
                ),
            );
        }

        let result = match request.command {
            ExecutionCommand::Capabilities(payload) => {
                let context = ProviderTargetContext {
                    provider_id: payload.provider_id,
                    channel: payload.channel,
                    credential: payload.credential,
                    options: payload.options,
                };
                self.executor
                    .capabilities(context)
                    .await
                    .map(ExecutionResult::Capabilities)
            }
            ExecutionCommand::Validate(payload) => self
                .executor
                .validate(request_id.clone(), payload)
                .await
                .map(ExecutionResult::Execution),
            ExecutionCommand::Publish(payload) => self
                .executor
                .publish(request_id.clone(), payload)
                .await
                .map(ExecutionResult::Execution),
        };

        match result {
            Ok(result) => ResponseEnvelope::ok(request_id, result),
            Err(failure) => ResponseEnvelope::error(request_id, protocol_error(failure)),
        }
    }
}

fn protocol_error(failure: ExecutionFailure) -> ProtocolError {
    match failure {
        ExecutionFailure::InvalidRequest { issues } => ProtocolError::invalid_request(issues),
        ExecutionFailure::ProviderNotFound(_) => ProtocolError::new(
            ProtocolErrorCode::ProviderNotFound,
            "requested provider adapter is not registered",
        ),
        ExecutionFailure::CapabilityDiscovery(error) => ProtocolError::new(
            ProtocolErrorCode::CapabilityDiscoveryFailed,
            format!("capability discovery failed: {}", error.code),
        ),
        ExecutionFailure::ProviderContract(_) => ProtocolError::new(
            ProtocolErrorCode::Internal,
            "provider adapter violated the capability identity contract",
        ),
    }
}
