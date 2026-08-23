use std::collections::BTreeSet;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    ContentVariant, Extensions, MediaKind, ProviderId, PublicationFormat, ValidationIssue,
};

/// Text support and limits reported by a provider at runtime.
#[derive(Clone, Debug, Eq, JsonSchema, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TextCapabilities {
    /// Whether text can be included at all.
    pub supported: bool,
    /// Maximum Unicode scalar values; `None` means Prism has no known limit.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_characters: Option<u32>,
}

/// Alternative-text support and limits.
#[derive(Clone, Debug, Eq, JsonSchema, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AltTextCapabilities {
    /// Whether the provider can preserve alt text.
    pub supported: bool,
    /// Maximum Unicode scalar values; `None` means Prism has no known limit.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_characters: Option<u32>,
}

/// Media support and limits reported by a provider at runtime.
#[derive(Clone, Debug, Eq, JsonSchema, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MediaCapabilities {
    /// Supported media kinds.
    #[serde(default)]
    pub supported_kinds: BTreeSet<MediaKind>,
    /// Maximum media items in one publication; `None` means no known limit.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_items: Option<u16>,
    /// Whether one publication can mix media kinds.
    pub mixed_kinds: bool,
    /// Alternative-text behavior.
    pub alt_text: AltTextCapabilities,
}

/// Runtime capability snapshot for one provider/channel context.
#[derive(Clone, Debug, Eq, JsonSchema, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProviderCapabilities {
    /// Provider that produced the snapshot.
    pub provider_id: ProviderId,
    /// Optional provider-defined snapshot revision.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub revision: Option<String>,
    /// Supported publication formats.
    #[serde(default)]
    pub formats: BTreeSet<PublicationFormat>,
    /// Text support.
    pub text: TextCapabilities,
    /// Media support.
    pub media: MediaCapabilities,
    /// Namespaced capabilities that are not yet canonical.
    #[serde(default, skip_serializing_if = "Extensions::is_empty")]
    pub extensions: Extensions,
}

impl ProviderCapabilities {
    /// Applies provider-neutral capability checks to one resolved variant.
    #[must_use]
    pub fn validate_variant(&self, variant: &ContentVariant) -> Vec<ValidationIssue> {
        let base = format!("/variants/{}", variant.id);
        let mut issues = Vec::new();

        if !self.formats.contains(&variant.format) {
            issues.push(ValidationIssue::error(
                "capability.format.unsupported",
                format!("{base}/format"),
                "provider does not support the requested publication format",
            ));
        }

        if variant.body.text.is_some() && !self.text.supported {
            issues.push(ValidationIssue::error(
                "capability.text.unsupported",
                format!("{base}/body/text"),
                "provider does not support text for this target",
            ));
        }
        if let Some(maximum) = self.text.max_characters {
            if variant.body.text_character_count() > maximum as usize {
                issues.push(ValidationIssue::error(
                    "capability.text.too_long",
                    format!("{base}/body/text"),
                    "text exceeds the provider capability snapshot",
                ));
            }
        }

        if let Some(maximum) = self.media.max_items {
            if variant.body.media.len() > maximum as usize {
                issues.push(ValidationIssue::error(
                    "capability.media.too_many_items",
                    format!("{base}/body/media"),
                    "media count exceeds the provider capability snapshot",
                ));
            }
        }

        let distinct_kinds: BTreeSet<_> =
            variant.body.media.iter().map(|media| media.kind).collect();
        if distinct_kinds.len() > 1 && !self.media.mixed_kinds {
            issues.push(ValidationIssue::error(
                "capability.media.mixed_kinds_unsupported",
                format!("{base}/body/media"),
                "provider does not support mixed media kinds",
            ));
        }

        for (index, media) in variant.body.media.iter().enumerate() {
            let media_path = format!("{base}/body/media/{index}");
            if !self.media.supported_kinds.contains(&media.kind) {
                issues.push(ValidationIssue::error(
                    "capability.media.kind_unsupported",
                    format!("{media_path}/kind"),
                    "provider does not support this media kind",
                ));
            }
            if let Some(alt_text) = &media.alt_text {
                if !self.media.alt_text.supported {
                    issues.push(ValidationIssue::error(
                        "capability.media.alt_text_unsupported",
                        format!("{media_path}/alt_text"),
                        "provider cannot preserve alternative text",
                    ));
                }
                if let Some(maximum) = self.media.alt_text.max_characters {
                    if alt_text.chars().count() > maximum as usize {
                        issues.push(ValidationIssue::error(
                            "capability.media.alt_text_too_long",
                            format!("{media_path}/alt_text"),
                            "alternative text exceeds the provider capability snapshot",
                        ));
                    }
                }
            }
        }

        issues
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use crate::{ContentBody, Extensions, LocaleTag, Provenance, ProvenanceKind, VariantId};

    use super::*;

    fn text_variant(text: &str) -> ContentVariant {
        ContentVariant {
            id: VariantId::new("uk").expect("valid variant ID"),
            locale: LocaleTag::new("uk-UA").expect("valid locale"),
            voice_profile: None,
            audience: None,
            provider_target: None,
            format: PublicationFormat::Post,
            body: ContentBody {
                text: Some(text.to_owned()),
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
    fn text_limit_uses_characters_not_utf8_bytes() {
        let capabilities = ProviderCapabilities {
            provider_id: ProviderId::new("test.fake").expect("valid provider ID"),
            revision: None,
            formats: BTreeSet::from([PublicationFormat::Post]),
            text: TextCapabilities {
                supported: true,
                max_characters: Some(3),
            },
            media: MediaCapabilities {
                supported_kinds: BTreeSet::new(),
                max_items: Some(0),
                mixed_kinds: false,
                alt_text: AltTextCapabilities {
                    supported: false,
                    max_characters: None,
                },
            },
            extensions: Extensions::new(),
        };

        assert!(
            capabilities
                .validate_variant(&text_variant("три"))
                .is_empty()
        );
        assert_eq!(
            capabilities.validate_variant(&text_variant("чотири")).len(),
            1
        );
    }
}
