//! Versioned language-neutral execution envelopes.

use prism_core::{
    ChannelRef, CredentialRef, ExecutionReport, Extensions, ProviderCapabilities, ProviderId,
    PublishRequest, RequestId, ValidationIssue,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Exact wire protocol identifier. It is independent from crate versions.
pub const PROTOCOL_VERSION: &str = "prism-execution.v1";

/// Capability discovery input for one provider/channel context.
#[derive(Clone, Debug, Eq, JsonSchema, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CapabilitiesRequest {
    /// Provider adapter identifier.
    pub provider_id: ProviderId,
    /// Opaque channel reference.
    pub channel: ChannelRef,
    /// Optional credential reference resolved by the adapter.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub credential: Option<CredentialRef>,
    /// Provider-scoped capability context.
    #[serde(default, skip_serializing_if = "Extensions::is_empty")]
    pub options: Extensions,
}

/// Operation embedded in a request envelope.
#[derive(Clone, Debug, Eq, JsonSchema, PartialEq, Serialize, Deserialize)]
#[serde(tag = "operation", content = "payload", rename_all = "snake_case")]
pub enum ExecutionCommand {
    /// Discover current provider capabilities.
    Capabilities(CapabilitiesRequest),
    /// Run all preflight stages with no publish action.
    Validate(PublishRequest),
    /// Run preflight and explicitly dispatch according to policy.
    Publish(PublishRequest),
}

impl ExecutionCommand {
    /// Stable operation label used in metadata-only diagnostics.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Capabilities(_) => "capabilities",
            Self::Validate(_) => "validate",
            Self::Publish(_) => "publish",
        }
    }
}

/// One `prism-execution.v1` request.
#[derive(Clone, Debug, Eq, JsonSchema, PartialEq, Serialize, Deserialize)]
pub struct RequestEnvelope {
    /// Must equal [`PROTOCOL_VERSION`].
    pub protocol_version: String,
    /// Caller correlation identifier.
    pub request_id: RequestId,
    /// Requested operation and payload.
    #[serde(flatten)]
    pub command: ExecutionCommand,
}

/// Successful operation result.
#[derive(Clone, Debug, Eq, JsonSchema, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum ExecutionResult {
    /// Capability snapshot.
    Capabilities(ProviderCapabilities),
    /// Validation or publication report.
    Execution(ExecutionReport),
}

/// Stable request-level protocol error code.
#[derive(Clone, Copy, Debug, Eq, JsonSchema, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProtocolErrorCode {
    /// Input is not a valid JSON protocol envelope.
    InvalidEnvelope,
    /// The requested protocol version is unsupported.
    UnsupportedProtocol,
    /// The envelope is valid but domain structure is not.
    InvalidRequest,
    /// No adapter is registered for the requested provider.
    ProviderNotFound,
    /// Adapter returned a request-level capability failure.
    CapabilityDiscoveryFailed,
    /// Unexpected internal failure that is safe to expose.
    Internal,
}

/// Safe request-level error. It never echoes the source payload.
#[derive(Clone, Debug, Eq, JsonSchema, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProtocolError {
    /// Machine-stable error code.
    pub code: ProtocolErrorCode,
    /// Redacted human-readable explanation.
    pub message: String,
    /// Structured domain issues when available.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub issues: Vec<ValidationIssue>,
}

impl ProtocolError {
    /// Creates a protocol error without validation issues.
    #[must_use]
    pub fn new(code: ProtocolErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            issues: Vec::new(),
        }
    }

    /// Creates an invalid-request error with structured issues.
    #[must_use]
    pub fn invalid_request(issues: Vec<ValidationIssue>) -> Self {
        Self {
            code: ProtocolErrorCode::InvalidRequest,
            message: "request failed deterministic structural validation".to_owned(),
            issues,
        }
    }
}

/// Mutually exclusive successful or failed response body.
#[derive(Clone, Debug, Eq, JsonSchema, PartialEq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum ResponseBody {
    /// Successful operation.
    Ok {
        /// Typed operation result.
        result: ExecutionResult,
    },
    /// Request-level failure.
    Error {
        /// Typed safe error.
        error: ProtocolError,
    },
}

/// One `prism-execution.v1` response.
#[derive(Clone, Debug, Eq, JsonSchema, PartialEq, Serialize, Deserialize)]
pub struct ResponseEnvelope {
    /// Protocol version used by the runtime.
    pub protocol_version: String,
    /// Correlation identifier copied from a parsed request, or a safe fallback.
    pub request_id: RequestId,
    /// Exactly one success result or error.
    #[serde(flatten)]
    pub body: ResponseBody,
}

impl ResponseEnvelope {
    /// Creates a successful response.
    #[must_use]
    pub fn ok(request_id: RequestId, result: ExecutionResult) -> Self {
        Self {
            protocol_version: PROTOCOL_VERSION.to_owned(),
            request_id,
            body: ResponseBody::Ok { result },
        }
    }

    /// Creates a failed response.
    #[must_use]
    pub fn error(request_id: RequestId, error: ProtocolError) -> Self {
        Self {
            protocol_version: PROTOCOL_VERSION.to_owned(),
            request_id,
            body: ResponseBody::Error { error },
        }
    }
}
