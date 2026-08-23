// © 2026 aiaiaiai · aiaiaiai.org
// SPDX-License-Identifier: Apache-2.0

use std::collections::BTreeSet;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    ChannelRef, ContentVariant, CredentialRef, Extensions, IdempotencyKey, ProviderId, TargetId,
    ValidationIssue, VariantId,
};

/// Behavior when one or more targets fail preflight validation.
#[derive(Clone, Copy, Debug, Default, Eq, JsonSchema, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DispatchPolicy {
    /// Perform no external action unless every target passes preflight.
    #[default]
    RequireAllValid,
    /// Dispatch each valid target independently.
    Independent,
}

/// Explicit deterministic selection of a variant for one target.
#[derive(Clone, Debug, Eq, JsonSchema, PartialEq, Serialize, Deserialize)]
#[serde(tag = "mode", rename_all = "snake_case", deny_unknown_fields)]
pub enum VariantSelection {
    /// Select exactly one variant.
    Exact {
        /// Required variant identifier.
        variant_id: VariantId,
    },
    /// Select the first existing, provider-compatible variant in order.
    Ordered {
        /// Candidate identifiers from highest to lowest priority.
        variant_ids: Vec<VariantId>,
    },
}

impl VariantSelection {
    /// Returns candidates in deterministic priority order.
    pub fn candidates(&self) -> Box<dyn Iterator<Item = &VariantId> + '_> {
        match self {
            Self::Exact { variant_id } => Box::new(std::iter::once(variant_id)),
            Self::Ordered { variant_ids } => Box::new(variant_ids.iter()),
        }
    }
}

/// One configured provider destination and its explicit variant selection.
#[derive(Clone, Debug, Eq, JsonSchema, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublishTarget {
    /// Request-local target identifier.
    pub id: TargetId,
    /// Provider adapter identifier.
    pub provider_id: ProviderId,
    /// Opaque configured channel reference.
    pub channel: ChannelRef,
    /// Optional credential reference resolved inside the adapter boundary.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub credential: Option<CredentialRef>,
    /// Deterministic content variant selection.
    pub selection: VariantSelection,
    /// Provider-scoped options; raw credentials are prohibited.
    #[serde(default, skip_serializing_if = "Extensions::is_empty")]
    pub options: Extensions,
}

/// Immutable input for validation or publication.
#[derive(Clone, Debug, Eq, JsonSchema, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublishRequest {
    /// Caller-owned key for one logical multi-target publication.
    pub idempotency_key: IdempotencyKey,
    /// Explicit partial-publication behavior.
    #[serde(default = "default_dispatch_policy")]
    pub dispatch_policy: DispatchPolicy,
    /// Explicit localized variants.
    pub variants: Vec<ContentVariant>,
    /// Ordered provider targets. Outcome order follows this order.
    pub targets: Vec<PublishTarget>,
}

const fn default_dispatch_policy() -> DispatchPolicy {
    DispatchPolicy::RequireAllValid
}

/// Stable target-scoped idempotency material passed to an adapter.
#[derive(Clone, Debug, Eq, JsonSchema, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IdempotencyScope {
    /// Logical publication key.
    pub root: IdempotencyKey,
    /// Target within the publication.
    pub target_id: TargetId,
}

impl IdempotencyScope {
    /// Returns a stable string. Adapters may hash or map it to provider limits.
    #[must_use]
    pub fn stable_string(&self) -> String {
        format!("{}:{}", self.root, self.target_id)
    }
}

impl PublishRequest {
    /// Finds the first eligible variant according to the target selection.
    pub fn resolve_variant(
        &self,
        target: &PublishTarget,
    ) -> Result<&ContentVariant, ValidationIssue> {
        for candidate in target.selection.candidates() {
            if let Some(variant) = self
                .variants
                .iter()
                .find(|variant| &variant.id == candidate)
            {
                if variant.supports_provider(&target.provider_id) {
                    return Ok(variant);
                }
            }
        }

        Err(ValidationIssue::error(
            "selection.no_eligible_variant",
            format!("/targets/{}/selection", target.id),
            "selection contains no existing variant eligible for this provider",
        ))
    }

    /// Derives stable adapter idempotency material for a target.
    #[must_use]
    pub fn idempotency_scope(&self, target: &PublishTarget) -> IdempotencyScope {
        IdempotencyScope {
            root: self.idempotency_key.clone(),
            target_id: target.id.clone(),
        }
    }

    /// Validates request-local invariants without consulting a provider.
    #[must_use]
    pub fn structural_issues(&self) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();
        if self.variants.is_empty() {
            issues.push(ValidationIssue::error(
                "request.variants.empty",
                "/variants",
                "at least one content variant is required",
            ));
        }
        if self.targets.is_empty() {
            issues.push(ValidationIssue::error(
                "request.targets.empty",
                "/targets",
                "at least one publish target is required",
            ));
        }

        let mut variant_ids = BTreeSet::new();
        for (index, variant) in self.variants.iter().enumerate() {
            let path = format!("/variants/{index}");
            if !variant_ids.insert(&variant.id) {
                issues.push(ValidationIssue::error(
                    "request.variant_id.duplicate",
                    format!("{path}/id"),
                    "variant IDs must be unique within a request",
                ));
            }
            if variant.body.is_empty() {
                issues.push(ValidationIssue::error(
                    "content.empty",
                    format!("{path}/body"),
                    "a variant must contain meaningful text or media",
                ));
            }
            if variant
                .body
                .text
                .as_ref()
                .is_some_and(|text| text.trim().is_empty())
            {
                issues.push(ValidationIssue::error(
                    "content.text.empty",
                    format!("{path}/body/text"),
                    "explicit text must not be empty or whitespace-only",
                ));
            }

            let mut media_refs = BTreeSet::new();
            for (media_index, media) in variant.body.media.iter().enumerate() {
                if !media_refs.insert(&media.reference) {
                    issues.push(ValidationIssue::error(
                        "content.media_ref.duplicate",
                        format!("{path}/body/media/{media_index}/ref"),
                        "media references must be unique within a variant",
                    ));
                }
                validate_extension_secrets(
                    &media.extensions,
                    &format!("{path}/body/media/{media_index}/extensions"),
                    &mut issues,
                );
            }
            validate_extension_secrets(
                &variant.extensions,
                &format!("{path}/extensions"),
                &mut issues,
            );
            if let Some(audience) = &variant.audience {
                if let Some(country) = &audience.country {
                    if country.len() != 2
                        || !country
                            .chars()
                            .all(|character| character.is_ascii_uppercase())
                    {
                        issues.push(ValidationIssue::error(
                            "audience.country.invalid",
                            format!("{path}/audience/country"),
                            "country must be an ISO 3166-1 alpha-2 uppercase code",
                        ));
                    }
                }
                for (field, value) in [
                    ("region", &audience.region),
                    ("city", &audience.city),
                    ("segment", &audience.segment),
                ] {
                    if value.as_ref().is_some_and(|value| value.trim().is_empty()) {
                        issues.push(ValidationIssue::error(
                            "audience.value.empty",
                            format!("{path}/audience/{field}"),
                            "audience values must not be empty or whitespace-only",
                        ));
                    }
                }
            }
        }

        let mut target_ids = BTreeSet::new();
        for (index, target) in self.targets.iter().enumerate() {
            let path = format!("/targets/{index}");
            if !target_ids.insert(&target.id) {
                issues.push(ValidationIssue::error(
                    "request.target_id.duplicate",
                    format!("{path}/id"),
                    "target IDs must be unique within a request",
                ));
            }
            if let VariantSelection::Ordered { variant_ids } = &target.selection {
                if variant_ids.is_empty() {
                    issues.push(ValidationIssue::error(
                        "selection.candidates.empty",
                        format!("{path}/selection/variant_ids"),
                        "ordered selection requires at least one candidate",
                    ));
                }
                let unique: BTreeSet<_> = variant_ids.iter().collect();
                if unique.len() != variant_ids.len() {
                    issues.push(ValidationIssue::error(
                        "selection.candidates.duplicate",
                        format!("{path}/selection/variant_ids"),
                        "ordered selection candidates must be unique",
                    ));
                }
            }

            for (key, _) in target.options.iter() {
                if !key.is_scoped_to(&target.provider_id) {
                    issues.push(ValidationIssue::error(
                        "target.option.namespace_mismatch",
                        format!("{path}/options/{key}"),
                        "provider options must use the target provider namespace",
                    ));
                }
            }
            validate_extension_secrets(&target.options, &format!("{path}/options"), &mut issues);

            if let Err(issue) = self.resolve_variant(target) {
                issues.push(issue);
            }
        }

        issues
    }
}

fn validate_extension_secrets(
    extensions: &Extensions,
    path: &str,
    issues: &mut Vec<ValidationIssue>,
) {
    for (key, _) in extensions.iter() {
        if key.looks_secret_bearing() {
            issues.push(ValidationIssue::error(
                "extension.secret_prohibited",
                format!("{path}/{key}"),
                "raw credentials and secret-bearing fields are prohibited in extensions",
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ContentBody, LocaleTag, Provenance, ProvenanceKind, PublicationFormat, VoiceProfileRef,
    };

    use super::*;

    fn variant(id: &str, provider_target: Option<&str>) -> ContentVariant {
        ContentVariant {
            id: VariantId::new(id).expect("valid variant ID"),
            locale: LocaleTag::new("uk-UA").expect("valid locale"),
            voice_profile: Some(
                VoiceProfileRef::new("0x0sky.instagram.uk_SP").expect("valid voice profile"),
            ),
            audience: None,
            provider_target: provider_target
                .map(|provider| ProviderId::new(provider).expect("valid provider ID")),
            format: PublicationFormat::Post,
            body: ContentBody {
                text: Some("привіт".to_owned()),
                media: Vec::new(),
            },
            provenance: Provenance {
                kind: ProvenanceKind::Human,
                producer: None,
                source_refs: Vec::new(),
            },
            extensions: Extensions::new(),
        }
    }

    #[test]
    fn ordered_selection_skips_provider_incompatible_variant() {
        let target = PublishTarget {
            id: TargetId::new("threads-uk").expect("valid target ID"),
            provider_id: ProviderId::new("meta.threads").expect("valid provider ID"),
            channel: ChannelRef::new("personal").expect("valid channel"),
            credential: None,
            selection: VariantSelection::Ordered {
                variant_ids: vec![
                    VariantId::new("instagram").expect("valid variant ID"),
                    VariantId::new("generic").expect("valid variant ID"),
                ],
            },
            options: Extensions::new(),
        };
        let request = PublishRequest {
            idempotency_key: IdempotencyKey::new("publication-1").expect("valid key"),
            dispatch_policy: DispatchPolicy::RequireAllValid,
            variants: vec![
                variant("instagram", Some("meta.instagram")),
                variant("generic", None),
            ],
            targets: vec![target.clone()],
        };

        assert_eq!(
            request
                .resolve_variant(&target)
                .expect("eligible variant")
                .id
                .as_str(),
            "generic"
        );
        assert!(request.structural_issues().is_empty());
    }

    #[test]
    fn provider_options_cannot_smuggle_tokens() {
        let mut options = Extensions::new();
        options.insert(
            "meta.threads.access_token".parse().expect("valid key"),
            serde_json::Value::String("redacted-test-value".to_owned()),
        );
        let request = PublishRequest {
            idempotency_key: IdempotencyKey::new("publication-1").expect("valid key"),
            dispatch_policy: DispatchPolicy::Independent,
            variants: vec![variant("generic", None)],
            targets: vec![PublishTarget {
                id: TargetId::new("threads-uk").expect("valid target ID"),
                provider_id: ProviderId::new("meta.threads").expect("valid provider ID"),
                channel: ChannelRef::new("personal").expect("valid channel"),
                credential: None,
                selection: VariantSelection::Exact {
                    variant_id: VariantId::new("generic").expect("valid variant ID"),
                },
                options,
            }],
        };

        assert!(
            request
                .structural_issues()
                .iter()
                .any(|issue| issue.code == "extension.secret_prohibited")
        );
    }
}
