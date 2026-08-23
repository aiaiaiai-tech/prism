use std::error::Error;

#[cfg(feature = "test-provider")]
use std::{collections::BTreeSet, sync::Arc};

use prism_core::RequestId;
#[cfg(feature = "test-provider")]
use prism_core::{
    AltTextCapabilities, Extensions, MediaCapabilities, MediaKind, ProviderCapabilities,
    ProviderId, PublicationFormat, TextCapabilities,
};
use prism_protocol::{
    ProtocolError, ProtocolErrorCode, RequestEnvelope, ResponseEnvelope,
};
use prism_provider::ProviderRegistry;
use prism_runtime::{ExecutionService, Executor};
use tokio::io::{self, AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tracing::{info, warn};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    init_tracing();
    let json_mode = parse_args()?;
    let service = build_service()?;
    if json_mode {
        run_json(&service).await
    } else {
        run_ndjson(&service).await
    }
}

fn parse_args() -> Result<bool, Box<dyn Error>> {
    let mut json_mode = false;
    for argument in std::env::args().skip(1) {
        match argument.as_str() {
            "--json" => json_mode = true,
            "--help" | "-h" => {
                eprintln!(
                    "prism-runtime [--json]\n\nDefault: NDJSON stdin/stdout. --json: one JSON envelope to EOF."
                );
                std::process::exit(0);
            }
            _ => return Err(format!("unknown argument: {argument}").into()),
        }
    }
    Ok(json_mode)
}

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .json()
        .with_writer(std::io::stderr)
        .init();
}

fn build_service() -> Result<ExecutionService, Box<dyn Error>> {
    #[allow(unused_mut)]
    let mut registry = ProviderRegistry::new();
    #[cfg(feature = "test-provider")]
    {
        let provider_id = ProviderId::new("test.fake")?;
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
        registry.register(Arc::new(prism_testkit::FakeProvider::new(
            provider_id,
            capabilities,
        )))?;
    }

    Ok(ExecutionService::new(Executor::new(registry)))
}

async fn run_json(service: &ExecutionService) -> Result<(), Box<dyn Error>> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).await?;
    let response = parse_and_handle(service, &input, "unparsed").await;
    write_response(&response).await
}

async fn run_ndjson(service: &ExecutionService) -> Result<(), Box<dyn Error>> {
    let mut lines = BufReader::new(io::stdin()).lines();
    let mut line_number = 0_u64;
    while let Some(line) = lines.next_line().await? {
        line_number = line_number.saturating_add(1);
        if line.trim().is_empty() {
            continue;
        }
        let fallback = format!("unparsed-{line_number}");
        let response = parse_and_handle(service, &line, &fallback).await;
        write_response(&response).await?;
    }
    Ok(())
}

async fn parse_and_handle(
    service: &ExecutionService,
    source: &str,
    fallback_request_id: &str,
) -> ResponseEnvelope {
    match serde_json::from_str::<RequestEnvelope>(source) {
        Ok(request) => {
            info!(
                request_id = %request.request_id,
                operation = request.command.name(),
                protocol_version = request.protocol_version,
                "handling Prism execution request"
            );
            service.handle(request).await
        }
        Err(error) => {
            warn!(
                line = error.line(),
                column = error.column(),
                "rejected malformed Prism envelope"
            );
            let request_id = RequestId::new(fallback_request_id)
                .unwrap_or_else(|_| RequestId::new("unparsed").expect("static fallback is valid"));
            ResponseEnvelope::error(
                request_id,
                ProtocolError::new(
                    ProtocolErrorCode::InvalidEnvelope,
                    format!(
                        "invalid JSON protocol envelope at line {} column {}",
                        error.line(),
                        error.column()
                    ),
                ),
            )
        }
    }
}

async fn write_response(response: &ResponseEnvelope) -> Result<(), Box<dyn Error>> {
    let mut output = serde_json::to_vec(response)?;
    output.push(b'\n');
    let mut stdout = io::stdout();
    stdout.write_all(&output).await?;
    stdout.flush().await?;
    Ok(())
}
