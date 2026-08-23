use std::{fmt, str::FromStr};

use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize};
use thiserror::Error;

const MAX_REFERENCE_LENGTH: usize = 256;

/// Error returned when an opaque Prism reference is malformed.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
#[error("invalid {kind}: {reason}")]
pub struct ReferenceError {
    kind: &'static str,
    reason: &'static str,
}

impl ReferenceError {
    fn new(kind: &'static str, reason: &'static str) -> Self {
        Self { kind, reason }
    }
}

fn validate_reference(value: &str, kind: &'static str) -> Result<(), ReferenceError> {
    if value.is_empty() {
        return Err(ReferenceError::new(kind, "must not be empty"));
    }
    if value.len() > MAX_REFERENCE_LENGTH {
        return Err(ReferenceError::new(kind, "is longer than 256 bytes"));
    }
    if value.trim() != value {
        return Err(ReferenceError::new(
            kind,
            "must not have leading or trailing whitespace",
        ));
    }
    if value.chars().any(char::is_control) {
        return Err(ReferenceError::new(
            kind,
            "must not contain control characters",
        ));
    }
    Ok(())
}

macro_rules! opaque_reference {
    ($name:ident, $kind:literal, $docs:literal) => {
        #[doc = $docs]
        #[derive(Clone, Debug, Eq, Hash, JsonSchema, Ord, PartialEq, PartialOrd, Serialize)]
        #[serde(transparent)]
        #[schemars(transparent)]
        pub struct $name(String);

        impl $name {
            /// Creates a validated opaque reference.
            pub fn new(value: impl Into<String>) -> Result<Self, ReferenceError> {
                let value = value.into();
                validate_reference(&value, $kind)?;
                Ok(Self(value))
            }

            /// Returns the reference as a string slice.
            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str(&self.0)
            }
        }

        impl FromStr for $name {
            type Err = ReferenceError;

            fn from_str(value: &str) -> Result<Self, Self::Err> {
                Self::new(value)
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                let value = String::deserialize(deserializer)?;
                Self::new(value).map_err(serde::de::Error::custom)
            }
        }
    };
}

opaque_reference!(
    ChannelRef,
    "channel reference",
    "Opaque reference to a configured provider channel."
);
opaque_reference!(
    CredentialRef,
    "credential reference",
    "Opaque reference resolved by a provider adapter; never a raw secret."
);
opaque_reference!(
    IdempotencyKey,
    "idempotency key",
    "Caller-owned idempotency key for one logical publication."
);
opaque_reference!(
    MediaRef,
    "media reference",
    "Opaque reference to media resolved outside the core domain."
);
opaque_reference!(
    RequestId,
    "request ID",
    "Correlation identifier supplied by the caller."
);
opaque_reference!(
    TargetId,
    "target ID",
    "Identifier unique within one publish request."
);
opaque_reference!(
    VariantId,
    "variant ID",
    "Identifier unique within one publish request."
);
opaque_reference!(
    VoiceProfileRef,
    "voice profile reference",
    "Reference to a voice or style profile, independent from locale."
);

/// Stable provider identity such as `instagram` or `meta.threads`.
#[derive(Clone, Debug, Eq, Hash, JsonSchema, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
#[schemars(transparent)]
pub struct ProviderId(String);

impl ProviderId {
    /// Creates a validated provider identifier.
    pub fn new(value: impl Into<String>) -> Result<Self, ReferenceError> {
        let value = value.into();
        validate_dotted_identifier(&value, "provider ID", false)?;
        Ok(Self(value))
    }

    /// Returns the identifier as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ProviderId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl FromStr for ProviderId {
    type Err = ReferenceError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::new(value)
    }
}

impl<'de> Deserialize<'de> for ProviderId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::new(value).map_err(serde::de::Error::custom)
    }
}

/// A BCP-47 locale tag. Voice/style profiles are intentionally separate.
#[derive(Clone, Debug, Eq, Hash, JsonSchema, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
#[schemars(transparent)]
pub struct LocaleTag(String);

impl LocaleTag {
    /// Parses and canonicalizes a well-formed BCP-47 tag.
    pub fn new(value: impl AsRef<str>) -> Result<Self, ReferenceError> {
        let tag = value
            .as_ref()
            .parse::<language_tags::LanguageTag>()
            .map_err(|_| ReferenceError::new("locale tag", "must be a well-formed BCP-47 tag"))?;
        Ok(Self(tag.to_string()))
    }

    /// Returns the canonical tag.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for LocaleTag {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl FromStr for LocaleTag {
    type Err = ReferenceError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::new(value)
    }
}

impl<'de> Deserialize<'de> for LocaleTag {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::new(value).map_err(serde::de::Error::custom)
    }
}

/// Key for a provider or Prism extension, for example `meta.instagram.location_id`.
#[derive(Clone, Debug, Eq, Hash, JsonSchema, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
#[schemars(transparent)]
pub struct NamespacedKey(String);

impl NamespacedKey {
    /// Creates a dotted extension key. At least two segments are required.
    pub fn new(value: impl Into<String>) -> Result<Self, ReferenceError> {
        let value = value.into();
        validate_dotted_identifier(&value, "namespaced key", true)?;
        Ok(Self(value))
    }

    /// Returns the key as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Returns whether this key belongs to the provider namespace.
    #[must_use]
    pub fn is_scoped_to(&self, provider: &ProviderId) -> bool {
        self.0
            .strip_prefix(provider.as_str())
            .is_some_and(|suffix| suffix.starts_with('.'))
    }

    /// Flags fields that must use credential resolution rather than options.
    #[must_use]
    pub fn looks_secret_bearing(&self) -> bool {
        self.0.split('.').any(|segment| {
            matches!(
                segment,
                "authorization" | "credential" | "password" | "secret" | "token"
            )
        })
    }
}

impl fmt::Display for NamespacedKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl FromStr for NamespacedKey {
    type Err = ReferenceError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::new(value)
    }
}

impl<'de> Deserialize<'de> for NamespacedKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::new(value).map_err(serde::de::Error::custom)
    }
}

fn validate_dotted_identifier(
    value: &str,
    kind: &'static str,
    namespace_required: bool,
) -> Result<(), ReferenceError> {
    validate_reference(value, kind)?;
    if value.len() > 128 {
        return Err(ReferenceError::new(kind, "is longer than 128 bytes"));
    }

    let segments: Vec<_> = value.split('.').collect();
    if namespace_required && segments.len() < 2 {
        return Err(ReferenceError::new(kind, "must contain a namespace"));
    }
    for segment in segments {
        let mut chars = segment.chars();
        let Some(first) = chars.next() else {
            return Err(ReferenceError::new(kind, "must not contain empty segments"));
        };
        if !first.is_ascii_lowercase() && !first.is_ascii_digit() {
            return Err(ReferenceError::new(
                kind,
                "segments must start with a lowercase ASCII letter or digit",
            ));
        }
        if !chars.all(|character| {
            character.is_ascii_lowercase()
                || character.is_ascii_digit()
                || character == '-'
                || character == '_'
        }) {
            return Err(ReferenceError::new(
                kind,
                "contains an unsupported character",
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn locale_accepts_bcp47_and_rejects_style_profiles() {
        assert!(LocaleTag::new("uk-UA").is_ok());
        assert!(LocaleTag::new("en-GB").is_ok());
        assert!(LocaleTag::new("uk_SP").is_err());
    }

    #[test]
    fn provider_and_extension_namespaces_are_strict() {
        let provider = ProviderId::new("meta.instagram").expect("valid provider ID");
        let key = NamespacedKey::new("meta.instagram.location_id").expect("valid key");
        assert!(key.is_scoped_to(&provider));
        assert!(ProviderId::new("Meta.Instagram").is_err());
        assert!(NamespacedKey::new("location_id").is_err());
    }

    #[test]
    fn secret_bearing_options_are_detected() {
        let key = NamespacedKey::new("meta.instagram.access_token").expect("valid key");
        assert!(key.looks_secret_bearing());
    }
}
