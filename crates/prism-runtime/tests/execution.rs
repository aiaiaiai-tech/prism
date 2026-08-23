// © 2026 aiaiaiai · aiaiaiai.org
// SPDX-License-Identifier: Apache-2.0

use std::{collections::BTreeSet, sync::Arc};

use prism_core::{
    AltTextCapabilities, ChannelRef, ContentBody, ContentVariant, DispatchPolicy, Extensions,
    IdempotencyKey, LocaleTag, Media, MediaCapabilities, MediaKind, MediaRef, Provenance,
    ProvenanceKind, ProviderCapabilities, ProviderId, PublicationFormat, PublishRequest,
    PublishTarget, RequestId, TargetId, TargetStatus, TextCapabilities, VariantId,
    VariantSelection,
};
use prism_provider::ProviderRegistry;
use prism_runtime::Executor;
use prism_testkit::FakeProvider;

fn capabilities(max_text: u32, media_kinds: BTreeSet<MediaKind>) -> ProviderCapabilities {
    ProviderCapabilities {
        provider_id: ProviderId::new("test.fake").expect("valid provider ID"),
        revision: Some("test-v1".to_owned()),
        formats: BTreeSet::from([PublicationFormat::Post]),
        text: TextCapabilities {
            supported: true,
            max_characters: Some(max_text),
        },
        media: MediaCapabilities {
            supported_kinds: media_kinds,
            max_items: Some(4),
            mixed_kinds: false,
            alt_text: AltTextCapabilities {
                supported: true,
                max_characters: Some(100),
            },
        },
        extensions: Extensions::new(),
    }
}

fn variant(id: &str, text: &str, media_kind: Option<MediaKind>) -> ContentVariant {
    ContentVariant {
        id: VariantId::new(id).expect("valid variant ID"),
        locale: LocaleTag::new("uk-UA").expect("valid locale"),
        voice_profile: None,
        audience: None,
        provider_target: None,
        format: PublicationFormat::Post,
        body: ContentBody {
            text: Some(text.to_owned()),
            media: media_kind
                .map(|kind| Media {
                    reference: MediaRef::new(format!("media-{id}")).expect("valid media ref"),
                    kind,
                    mime_type: None,
                    alt_text: Some("опис".to_owned()),
                    extensions: Extensions::new(),
                })
                .into_iter()
                .collect(),
        },
        provenance: Provenance {
            kind: ProvenanceKind::Human,
            producer: None,
            source_refs: Vec::new(),
        },
        extensions: Extensions::new(),
    }
}

fn target(id: &str, variant_id: &str) -> PublishTarget {
    PublishTarget {
        id: TargetId::new(id).expect("valid target ID"),
        provider_id: ProviderId::new("test.fake").expect("valid provider ID"),
        channel: ChannelRef::new("channel-1").expect("valid channel"),
        credential: None,
        selection: VariantSelection::Exact {
            variant_id: VariantId::new(variant_id).expect("valid variant ID"),
        },
        options: Extensions::new(),
    }
}

fn executor(capabilities: ProviderCapabilities) -> (Executor, FakeProvider) {
    let provider_id = capabilities.provider_id.clone();
    let fake = FakeProvider::new(provider_id, capabilities);
    let mut registry = ProviderRegistry::new();
    registry
        .register(Arc::new(fake.clone()))
        .expect("provider registers once");
    (Executor::new(registry), fake)
}

fn request(
    policy: DispatchPolicy,
    variants: Vec<ContentVariant>,
    targets: Vec<PublishTarget>,
) -> PublishRequest {
    PublishRequest {
        idempotency_key: IdempotencyKey::new("publication-42").expect("valid key"),
        dispatch_policy: policy,
        variants,
        targets,
    }
}

#[tokio::test]
async fn unsupported_media_fails_before_external_action() {
    let (executor, fake) = executor(capabilities(100, BTreeSet::from([MediaKind::Image])));
    let report = executor
        .publish(
            RequestId::new("request-1").expect("valid request ID"),
            request(
                DispatchPolicy::Independent,
                vec![variant("video", "привіт", Some(MediaKind::Video))],
                vec![target("target-video", "video")],
            ),
        )
        .await
        .expect("request structure is valid");

    assert_eq!(report.outcomes[0].status, TargetStatus::Failed);
    assert_eq!(fake.publish_count().expect("fake state"), 0);
    assert!(
        report.outcomes[0]
            .validation_issues
            .iter()
            .any(|issue| issue.code == "capability.media.kind_unsupported")
    );
}

#[tokio::test]
async fn require_all_valid_prevents_every_external_action() {
    let (executor, fake) = executor(capabilities(5, BTreeSet::new()));
    let report = executor
        .publish(
            RequestId::new("request-2").expect("valid request ID"),
            request(
                DispatchPolicy::RequireAllValid,
                vec![
                    variant("valid", "так", None),
                    variant("invalid", "занадто довго", None),
                ],
                vec![
                    target("target-valid", "valid"),
                    target("target-invalid", "invalid"),
                ],
            ),
        )
        .await
        .expect("request structure is valid");

    assert_eq!(report.outcomes[0].status, TargetStatus::Skipped);
    assert_eq!(report.outcomes[1].status, TargetStatus::Failed);
    assert_eq!(fake.publish_count().expect("fake state"), 0);
}

#[tokio::test]
async fn independent_policy_isolates_targets_and_derives_idempotency() {
    let (executor, fake) = executor(capabilities(5, BTreeSet::new()));
    let report = executor
        .publish(
            RequestId::new("request-3").expect("valid request ID"),
            request(
                DispatchPolicy::Independent,
                vec![
                    variant("valid", "так", None),
                    variant("invalid", "занадто довго", None),
                ],
                vec![
                    target("target-valid", "valid"),
                    target("target-invalid", "invalid"),
                ],
            ),
        )
        .await
        .expect("request structure is valid");

    assert_eq!(report.outcomes[0].status, TargetStatus::Published);
    assert_eq!(report.outcomes[1].status, TargetStatus::Failed);
    let calls = fake.recorded_publishes().expect("fake state");
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].idempotency_key, "publication-42:target-valid");
}

#[tokio::test]
async fn validate_mode_never_publishes_and_events_are_sequence_numbered() {
    let (executor, fake) = executor(capabilities(100, BTreeSet::new()));
    let report = executor
        .validate(
            RequestId::new("request-4").expect("valid request ID"),
            request(
                DispatchPolicy::Independent,
                vec![variant("valid", "привіт", None)],
                vec![target("target-valid", "valid")],
            ),
        )
        .await
        .expect("request structure is valid");

    assert_eq!(report.outcomes[0].status, TargetStatus::Validated);
    assert_eq!(fake.publish_count().expect("fake state"), 0);
    assert!(
        report
            .events
            .iter()
            .enumerate()
            .all(|(index, event)| event.sequence as usize == index + 1)
    );
}
