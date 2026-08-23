//! Provider-neutral types and deterministic rules for Prism.
//!
//! This crate deliberately has no network, storage, process, client, AI, or
//! infrastructure dependency.

mod capabilities;
mod content;
mod ids;
mod request;
mod result;
mod validation;

pub use capabilities::{
    AltTextCapabilities, MediaCapabilities, ProviderCapabilities, TextCapabilities,
};
pub use content::{
    Audience, ContentBody, ContentVariant, Extensions, Media, MediaKind, Provenance,
    ProvenanceKind, PublicationFormat,
};
pub use ids::{
    ChannelRef, CredentialRef, IdempotencyKey, LocaleTag, MediaRef, NamespacedKey, ProviderId,
    ReferenceError, RequestId, TargetId, VariantId, VoiceProfileRef,
};
pub use request::{
    DispatchPolicy, IdempotencyScope, PublishRequest, PublishTarget, VariantSelection,
};
pub use result::{
    DeliveryError, DeliveryErrorClass, EventKind, ExecutionEvent, ExecutionMode, ExecutionReport,
    ProviderReceipt, TargetOutcome, TargetStatus,
};
pub use validation::{ValidationIssue, ValidationSeverity};
