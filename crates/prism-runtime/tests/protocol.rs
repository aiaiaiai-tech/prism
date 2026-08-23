use std::{collections::BTreeSet, fs, path::PathBuf, sync::Arc};

use prism_core::{
    AltTextCapabilities, Extensions, MediaCapabilities, MediaKind, ProviderCapabilities,
    ProviderId, PublicationFormat, TextCapabilities,
};
use prism_protocol::{RequestEnvelope, ResponseEnvelope};
use prism_provider::ProviderRegistry;
use prism_runtime::{ExecutionService, Executor};
use prism_testkit::FakeProvider;

fn fixture(path: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../contracts/examples")
        .join(path)
}

#[tokio::test]
async fn publish_example_is_a_golden_protocol_contract() {
    let provider_id = ProviderId::new("test.fake").expect("valid provider ID");
    let capabilities = ProviderCapabilities {
        provider_id: provider_id.clone(),
        revision: Some("fixture-v1".to_owned()),
        formats: BTreeSet::from([
            PublicationFormat::Post,
            PublicationFormat::Story,
            PublicationFormat::ShortVideo,
        ]),
        text: TextCapabilities {
            supported: true,
            max_characters: Some(10_000),
        },
        media: MediaCapabilities {
            supported_kinds: BTreeSet::from([MediaKind::Image, MediaKind::Video]),
            max_items: Some(10),
            mixed_kinds: true,
            alt_text: AltTextCapabilities {
                supported: true,
                max_characters: Some(1_000),
            },
        },
        extensions: Extensions::new(),
    };
    let fake = FakeProvider::new(provider_id, capabilities);
    let mut registry = ProviderRegistry::new();
    registry
        .register(Arc::new(fake))
        .expect("provider registers once");
    let service = ExecutionService::new(Executor::new(registry));

    let request_source = fs::read_to_string(fixture("publish.request.json"))
        .expect("request fixture can be read");
    let expected_source = fs::read_to_string(fixture("publish.response.json"))
        .expect("response fixture can be read");
    let request: RequestEnvelope =
        serde_json::from_str(&request_source).expect("request fixture is valid");
    let expected: ResponseEnvelope =
        serde_json::from_str(&expected_source).expect("response fixture is valid");

    assert_eq!(service.handle(request).await, expected);
}
