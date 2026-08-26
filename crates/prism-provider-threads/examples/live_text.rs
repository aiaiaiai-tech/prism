// © 2026 aiaiaiai · aiaiaiai.org
// SPDX-License-Identifier: Apache-2.0

//! Explicit, opt-in end-to-end validation against a controlled Threads account.
//!
//! This example performs an irreversible external publish. It is intentionally
//! excluded from required CI and refuses to run without an exact confirmation.

use std::{env, error::Error, sync::Arc};

use async_trait::async_trait;
use prism_core::{
    ChannelRef, ContentBody, ContentVariant, CredentialRef, DeliveryError, DeliveryErrorClass,
    DispatchPolicy, Extensions, IdempotencyKey, LocaleTag, Provenance, ProvenanceKind, ProviderId,
    PublicationFormat, PublishRequest, PublishTarget, RequestId, TargetId, TargetStatus, VariantId,
    VariantSelection,
};
use prism_provider::ProviderRegistry;
use prism_provider_threads::{
    ReqwestThreadsTransport, THREADS_PROVIDER_ID, ThreadsAccessToken, ThreadsAdapter,
    ThreadsBinding, ThreadsBindingResolver, ThreadsObjectId,
};
use prism_runtime::Executor;
use thiserror::Error;

const REQUIRED_CONFIRMATION: &str = "PUBLISH_THREADS_TEST";

#[derive(Debug, Error)]
enum LiveValidationError {
    #[error("live publishing requires the exact confirmation PUBLISH_THREADS_TEST")]
    ConfirmationRequired,
    #[error("required live-validation environment is missing or invalid")]
    InvalidEnvironment,
    #[error("failed to initialize the Threads transport")]
    TransportInitialization,
    #[error("failed to register the Threads adapter")]
    AdapterRegistration,
    #[error("Prism execution failed before producing a report")]
    Execution,
    #[error("Threads live validation did not produce one published outcome")]
    PublishFailed,
    #[error("failed to serialize the redacted execution report")]
    Serialization,
}

struct EnvironmentBindingResolver {
    user_id: String,
    access_token: String,
}

#[async_trait]
impl ThreadsBindingResolver for EnvironmentBindingResolver {
    async fn resolve(
        &self,
        _channel: &ChannelRef,
        _credential: &CredentialRef,
    ) -> Result<ThreadsBinding, DeliveryError> {
        let user_id = ThreadsObjectId::new(self.user_id.clone()).map_err(|_| {
            DeliveryError::new(
                DeliveryErrorClass::AuthRequired,
                "threads.live.binding_invalid",
                "Threads live-validation binding is invalid",
            )
        })?;
        let access_token = ThreadsAccessToken::new(self.access_token.clone()).map_err(|_| {
            DeliveryError::new(
                DeliveryErrorClass::AuthRequired,
                "threads.live.binding_invalid",
                "Threads live-validation binding is invalid",
            )
        })?;
        Ok(ThreadsBinding::new(user_id, access_token))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    if env::var("PRISM_LIVE_CONFIRMATION").ok().as_deref() != Some(REQUIRED_CONFIRMATION) {
        return Err(LiveValidationError::ConfirmationRequired.into());
    }

    let text = read_required("PRISM_THREADS_TEXT")?;
    let resolver = EnvironmentBindingResolver {
        user_id: read_required("PRISM_THREADS_USER_ID")?,
        access_token: read_required("PRISM_THREADS_ACCESS_TOKEN")?,
    };
    let transport = ReqwestThreadsTransport::new(reqwest::Client::new())
        .map_err(|_| LiveValidationError::TransportInitialization)?;
    let adapter = Arc::new(ThreadsAdapter::new(Arc::new(resolver), Arc::new(transport)));
    let mut registry = ProviderRegistry::new();
    registry
        .register(adapter)
        .map_err(|_| LiveValidationError::AdapterRegistration)?;

    let report = Executor::new(registry)
        .publish(
            RequestId::new("threads-live-validation")?,
            live_request(text)?,
        )
        .await
        .map_err(|_| LiveValidationError::Execution)?;

    if report.outcomes.len() != 1 || report.outcomes[0].status != TargetStatus::Published {
        let safe_report = serde_json::to_string_pretty(&report)
            .map_err(|_| LiveValidationError::Serialization)?;
        eprintln!("{safe_report}");
        return Err(LiveValidationError::PublishFailed.into());
    }

    println!(
        "{}",
        serde_json::to_string_pretty(&report).map_err(|_| LiveValidationError::Serialization)?
    );
    Ok(())
}

fn read_required(name: &str) -> Result<String, LiveValidationError> {
    env::var(name)
        .ok()
        .filter(|value| !value.is_empty())
        .ok_or(LiveValidationError::InvalidEnvironment)
}

fn live_request(text: String) -> Result<PublishRequest, prism_core::ReferenceError> {
    let provider_id = ProviderId::new(THREADS_PROVIDER_ID)?;
    let variant_id = VariantId::new("threads-live-text")?;
    Ok(PublishRequest {
        idempotency_key: IdempotencyKey::new("threads-live-validation")?,
        dispatch_policy: DispatchPolicy::RequireAllValid,
        variants: vec![ContentVariant {
            id: variant_id.clone(),
            locale: LocaleTag::new("uk-UA")?,
            voice_profile: None,
            audience: None,
            provider_target: Some(provider_id.clone()),
            format: PublicationFormat::Post,
            body: ContentBody {
                text: Some(text),
                media: Vec::new(),
            },
            provenance: Provenance {
                kind: ProvenanceKind::Human,
                producer: Some("github-actions.manual".to_owned()),
                source_refs: Vec::new(),
            },
            extensions: Extensions::new(),
        }],
        targets: vec![PublishTarget {
            id: TargetId::new("threads-live")?,
            provider_id,
            channel: ChannelRef::new("threads-live-controlled")?,
            credential: Some(CredentialRef::new("threads-live-environment")?),
            selection: VariantSelection::Exact { variant_id },
            options: Extensions::new(),
        }],
    })
}
