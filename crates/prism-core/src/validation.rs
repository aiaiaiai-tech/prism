// © 2026 aiaiaiai · aiaiaiai.org
// SPDX-License-Identifier: Apache-2.0

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Severity of a deterministic validation issue.
#[derive(Clone, Copy, Debug, Eq, JsonSchema, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValidationSeverity {
    /// Prevents the target or request from being dispatched.
    Error,
    /// Preserves a non-blocking provider or content warning.
    Warning,
}

/// Stable validation code plus a safe human-readable explanation.
#[derive(Clone, Debug, Eq, JsonSchema, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ValidationIssue {
    /// Machine-stable code.
    pub code: String,
    /// JSON-pointer-like location in the request.
    pub path: String,
    /// Safe explanation that does not reproduce content or secrets.
    pub message: String,
    /// Whether the issue blocks dispatch.
    pub severity: ValidationSeverity,
}

impl ValidationIssue {
    /// Creates a blocking issue.
    #[must_use]
    pub fn error(
        code: impl Into<String>,
        path: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            code: code.into(),
            path: path.into(),
            message: message.into(),
            severity: ValidationSeverity::Error,
        }
    }

    /// Creates a non-blocking issue.
    #[must_use]
    pub fn warning(
        code: impl Into<String>,
        path: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            code: code.into(),
            path: path.into(),
            message: message.into(),
            severity: ValidationSeverity::Warning,
        }
    }

    /// Returns whether this issue prevents dispatch.
    #[must_use]
    pub fn is_error(&self) -> bool {
        self.severity == ValidationSeverity::Error
    }
}
