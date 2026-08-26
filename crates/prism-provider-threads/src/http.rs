// © 2026 aiaiaiai · aiaiaiai.org
// SPDX-License-Identifier: Apache-2.0

use async_trait::async_trait;
use reqwest::{Client, StatusCode, Url, header::RETRY_AFTER};
use serde::Deserialize;
use thiserror::Error;

use crate::{
    ThreadsBinding, ThreadsBindingError, ThreadsObjectId, ThreadsTransport, ThreadsTransportError,
    ThreadsTransportErrorKind,
};

const DEFAULT_API_BASE: &str = "https://graph.threads.net/v1.0/";

/// Invalid Threads HTTP transport configuration.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum ThreadsTransportConfigError {
    /// Base URL is malformed or cannot safely accept path segments.
    #[error("invalid Threads API base URL")]
    InvalidBaseUrl,
    /// Provider credentials must never be sent over plaintext HTTP.
    #[error("Threads API base URL must use HTTPS")]
    InsecureBaseUrl,
}

/// Reqwest implementation of the official Threads container/publish flow.
#[derive(Clone, Debug)]
pub struct ReqwestThreadsTransport {
    client: Client,
    api_base: Url,
}

impl ReqwestThreadsTransport {
    /// Creates a transport for the official production API.
    pub fn new(client: Client) -> Result<Self, ThreadsTransportConfigError> {
        Self::with_api_base(client, DEFAULT_API_BASE)
    }

    /// Creates a transport with an explicit HTTPS API base.
    pub fn with_api_base(
        client: Client,
        api_base: &str,
    ) -> Result<Self, ThreadsTransportConfigError> {
        let api_base =
            Url::parse(api_base).map_err(|_| ThreadsTransportConfigError::InvalidBaseUrl)?;
        if api_base.scheme() != "https" {
            return Err(ThreadsTransportConfigError::InsecureBaseUrl);
        }
        if api_base.cannot_be_a_base()
            || !api_base.username().is_empty()
            || api_base.password().is_some()
            || api_base.query().is_some()
            || api_base.fragment().is_some()
        {
            return Err(ThreadsTransportConfigError::InvalidBaseUrl);
        }
        Ok(Self { client, api_base })
    }

    fn endpoint(
        &self,
        user_id: &ThreadsObjectId,
        operation: &str,
    ) -> Result<Url, ThreadsTransportError> {
        let mut url = self.api_base.clone();
        let mut segments = url.path_segments_mut().map_err(|_| {
            ThreadsTransportError::new(
                ThreadsTransportErrorKind::InvalidResponse,
                "threads.transport.invalid_base_url",
            )
        })?;
        segments.pop_if_empty();
        segments.push(user_id.as_str());
        segments.push(operation);
        drop(segments);
        Ok(url)
    }

    async fn post_form(
        &self,
        binding: &ThreadsBinding,
        operation: &str,
        form: &[(&str, &str)],
    ) -> Result<ThreadsObjectId, ThreadsTransportError> {
        let url = self.endpoint(binding.user_id(), operation)?;
        let response = self
            .client
            .post(url)
            .bearer_auth(binding.access_token.expose_secret())
            .form(form)
            .send()
            .await
            .map_err(|_| {
                ThreadsTransportError::new(
                    ThreadsTransportErrorKind::Transient,
                    "threads.transport",
                )
            })?;

        let status = response.status();
        let retry_after_seconds = response
            .headers()
            .get(RETRY_AFTER)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.parse().ok());
        let body = response.bytes().await.map_err(|_| {
            ThreadsTransportError::new(
                ThreadsTransportErrorKind::InvalidResponse,
                "threads.response.unreadable",
            )
        })?;

        if status.is_success() {
            let response: IdResponse = serde_json::from_slice(&body).map_err(|_| {
                ThreadsTransportError::new(
                    ThreadsTransportErrorKind::InvalidResponse,
                    "threads.response.invalid_json",
                )
            })?;
            return ThreadsObjectId::new(response.id).map_err(invalid_object_id);
        }

        Err(classify_api_error(status, &body, retry_after_seconds))
    }
}

#[async_trait]
impl ThreadsTransport for ReqwestThreadsTransport {
    async fn create_text_container(
        &self,
        binding: &ThreadsBinding,
        text: &str,
    ) -> Result<ThreadsObjectId, ThreadsTransportError> {
        self.post_form(
            binding,
            "threads",
            &[("media_type", "TEXT"), ("text", text)],
        )
        .await
    }

    async fn publish_container(
        &self,
        binding: &ThreadsBinding,
        container_id: &ThreadsObjectId,
    ) -> Result<ThreadsObjectId, ThreadsTransportError> {
        self.post_form(
            binding,
            "threads_publish",
            &[("creation_id", container_id.as_str())],
        )
        .await
    }
}

#[derive(Deserialize)]
struct IdResponse {
    id: String,
}

#[derive(Default, Deserialize)]
struct ErrorEnvelope {
    #[serde(default)]
    error: Option<MetaError>,
}

#[derive(Default, Deserialize)]
struct MetaError {
    #[serde(default)]
    code: Option<i64>,
    #[serde(default)]
    error_subcode: Option<i64>,
    #[serde(default)]
    is_transient: bool,
}

fn classify_api_error(
    status: StatusCode,
    body: &[u8],
    retry_after_seconds: Option<u64>,
) -> ThreadsTransportError {
    let meta = serde_json::from_slice::<ErrorEnvelope>(body)
        .ok()
        .and_then(|envelope| envelope.error)
        .unwrap_or_default();
    let kind = if status == StatusCode::UNAUTHORIZED
        || status == StatusCode::FORBIDDEN
        || meta.code == Some(190)
    {
        ThreadsTransportErrorKind::Authentication
    } else if status == StatusCode::TOO_MANY_REQUESTS
        || matches!(meta.code, Some(4 | 17 | 32 | 613))
    {
        ThreadsTransportErrorKind::RateLimited
    } else if status.is_server_error() || meta.is_transient {
        ThreadsTransportErrorKind::Transient
    } else {
        ThreadsTransportErrorKind::Rejected
    };

    let code = match (meta.code, meta.error_subcode) {
        (Some(code), Some(subcode)) => format!("threads.api.{code}.{subcode}"),
        (Some(code), None) => format!("threads.api.{code}"),
        (None, _) => format!("threads.http.{}", status.as_u16()),
    };
    let mut error = ThreadsTransportError::new(kind, code);
    if let Some(seconds) = retry_after_seconds {
        error = error.with_retry_after(seconds);
    }
    error
}

fn invalid_object_id(_error: ThreadsBindingError) -> ThreadsTransportError {
    ThreadsTransportError::new(
        ThreadsTransportErrorKind::InvalidResponse,
        "threads.response.invalid_object_id",
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn meta_auth_and_rate_limit_errors_are_classified_without_messages() {
        let auth = classify_api_error(
            StatusCode::BAD_REQUEST,
            br#"{"error":{"message":"secret upstream detail","code":190}}"#,
            None,
        );
        let rate = classify_api_error(
            StatusCode::TOO_MANY_REQUESTS,
            br#"{"error":{"message":"quota","code":4}}"#,
            Some(60),
        );

        assert_eq!(auth.kind, ThreadsTransportErrorKind::Authentication);
        assert_eq!(auth.code, "threads.api.190");
        assert!(!format!("{auth:?}").contains("secret upstream detail"));
        assert_eq!(rate.kind, ThreadsTransportErrorKind::RateLimited);
        assert_eq!(rate.retry_after_seconds, Some(60));
    }

    #[test]
    fn transport_rejects_plaintext_api_bases() {
        let error =
            ReqwestThreadsTransport::with_api_base(Client::new(), "http://graph.threads.net/v1.0/")
                .expect_err("HTTP must be rejected");

        assert_eq!(error, ThreadsTransportConfigError::InsecureBaseUrl);
    }
}
