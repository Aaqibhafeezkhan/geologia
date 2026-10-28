// Copyright 2026 Andre Cipriani Bandarra
// SPDX-License-Identifier: Apache-2.0

use super::*;
use crate::network::event_source::ServerSentEvent;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>;

#[test]
fn stream_parses_blocked_prompt_response() -> Result<()> {
    let event = Ok(ServerSentEvent {
        data: Some(
            r#"{"promptFeedback":{"blockReason":"SAFETY"},"usageMetadata":{"promptTokenCount":5}}"#
                .to_string(),
        ),
        ..Default::default()
    });

    let result = GeminiClient::parse_event(event).ok_or("SSE event should produce a result")?;
    let response = result?;
    assert!(response.candidates.is_empty());
    assert_eq!(
        response
            .prompt_feedback
            .ok_or("missing prompt_feedback")?
            .block_reason,
        Some(crate::prelude::BlockReason::Safety)
    );
    Ok(())
}

#[test]
fn resource_url_handles_prefixed_and_unprefixed_names() {
    let client = GeminiClient::new("test-api-key".into());
    assert_eq!(
        client.cached_content_resource_url("cachedContents/abc123xyz"),
        "https://generativelanguage.googleapis.com/v1beta/cachedContents/abc123xyz"
    );
    assert_eq!(
        client.cached_content_resource_url("abc123xyz"),
        "https://generativelanguage.googleapis.com/v1beta/cachedContents/abc123xyz"
    );
}

#[test]
fn all_endpoint_urls_build_from_configured_base_url() {
    let client = GeminiClient::builder("test-api-key")
        .base_url("https://custom.endpoint.example/v1")
        .build();

    assert_eq!(
        client.endpoint_url("models/gemini-2.0-flash:streamGenerateContent?alt=sse"),
        "https://custom.endpoint.example/v1/models/gemini-2.0-flash:streamGenerateContent?alt=sse"
    );
    assert_eq!(
        client.endpoint_url("models/gemini-2.0-flash:generateContent"),
        "https://custom.endpoint.example/v1/models/gemini-2.0-flash:generateContent"
    );
    assert_eq!(
        client.endpoint_url("models/text-embedding-004:predict"),
        "https://custom.endpoint.example/v1/models/text-embedding-004:predict"
    );
    assert_eq!(
        client.endpoint_url("models/gemini-2.0-flash:countTokens"),
        "https://custom.endpoint.example/v1/models/gemini-2.0-flash:countTokens"
    );
    assert_eq!(
        client.endpoint_url("models/imagen-3.0-generate-002:predict"),
        "https://custom.endpoint.example/v1/models/imagen-3.0-generate-002:predict"
    );
    assert_eq!(
        client.endpoint_url("cachedContents"),
        "https://custom.endpoint.example/v1/cachedContents"
    );
    assert_eq!(
        client.cached_content_resource_url("cachedContents/abc123xyz"),
        "https://custom.endpoint.example/v1/cachedContents/abc123xyz"
    );
    assert_eq!(
        client.cached_content_resource_url("abc123xyz"),
        "https://custom.endpoint.example/v1/cachedContents/abc123xyz"
    );
}

#[test]
fn endpoint_url_normalizes_slashes() {
    let client = GeminiClient::builder("test-api-key")
        .base_url("https://custom.endpoint.example/v1/")
        .build();

    assert_eq!(
        client.endpoint_url("/models/gemini-2.0-flash:generateContent"),
        "https://custom.endpoint.example/v1/models/gemini-2.0-flash:generateContent"
    );
    assert_eq!(
        client.endpoint_url("models/gemini-2.0-flash:generateContent"),
        "https://custom.endpoint.example/v1/models/gemini-2.0-flash:generateContent"
    );
}

#[test]
fn default_client_uses_default_base_url() {
    let client = GeminiClient::new("test-api-key".into());
    assert_eq!(client.base_url(), DEFAULT_BASE_URL);
    assert_eq!(client.api_key(), "test-api-key");
    assert_eq!(
        client.endpoint_url("models/gemini-2.0-flash:generateContent"),
        "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.0-flash:generateContent"
    );
}

#[tokio::test]
async fn generate_content_against_mock_server_checks_path_and_api_key() -> Result<()> {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/models/gemini-2.0-flash:generateContent"))
        .and(header("x-goog-api-key", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "candidates": [{
                "index": 0,
                "content": {
                    "parts": [{"text": "Hello from mock server!"}],
                    "role": "model"
                }
            }]
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = GeminiClient::builder("test-api-key")
        .base_url(mock_server.uri())
        .build();

    let request = GenerateContentRequest::builder()
        .contents(vec![Content::builder().add_text_part("Hello").build()])
        .build();

    let response = client.generate_content(&request, "gemini-2.0-flash").await?;
    let candidate = response.candidates.first().ok_or("expected candidate")?;
    assert_eq!(candidate.get_text().as_deref(), Some("Hello from mock server!"));
    Ok(())
}

#[tokio::test]
async fn http_client_is_used_when_provided() -> Result<()> {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/models/gemini-2.0-flash:generateContent"))
        .and(header("x-goog-api-key", "test-api-key"))
        .and(header("x-custom-test-header", "custom-client-val"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "candidates": [{
                "index": 0,
                "content": {
                    "parts": [{"text": "Custom HTTP client used!"}],
                    "role": "model"
                }
            }]
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        "x-custom-test-header",
        reqwest::header::HeaderValue::from_static("custom-client-val"),
    );
    let custom_http_client = reqwest::Client::builder()
        .default_headers(headers)
        .build()?;

    let client = GeminiClient::builder("test-api-key")
        .base_url(mock_server.uri())
        .http_client(custom_http_client)
        .build();

    let request = GenerateContentRequest::builder()
        .contents(vec![Content::builder().add_text_part("Test").build()])
        .build();

    let response = client.generate_content(&request, "gemini-2.0-flash").await?;
    let candidate = response.candidates.first().ok_or("expected candidate")?;
    assert_eq!(candidate.get_text().as_deref(), Some("Custom HTTP client used!"));
    Ok(())
}

#[tokio::test]
async fn count_tokens_against_mock_server_checks_path_and_api_key() -> Result<()> {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/models/gemini-2.0-flash:countTokens"))
        .and(header("x-goog-api-key", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "totalTokens": 25
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = GeminiClient::builder("test-api-key")
        .base_url(mock_server.uri())
        .build();

    let request = CountTokensRequest::builder()
        .contents(vec![Content::builder().add_text_part("Count me").build()])
        .build();

    let response = client.count_tokens(&request, "gemini-2.0-flash").await?;
    assert_eq!(response.total_tokens, 25);
    Ok(())
}

#[tokio::test]
async fn error_for_status_maps_structured_gemini_error() -> Result<()> {
    let raw_response = http::Response::builder()
        .status(400)
        .body(reqwest::Body::from(
            r#"{"error":{"code":400,"message":"Invalid TTL duration","status":"INVALID_ARGUMENT"}}"#,
        ))?;
    let response: reqwest::Response = raw_response.into();

    let err = GeminiClient::error_for_status(response)
        .await
        .err()
        .ok_or("expected error")?;

    match err {
        GeminiError::VertexError(vertex_err) => {
            assert_eq!(vertex_err.code, 400);
            assert_eq!(vertex_err.message, "Invalid TTL duration");
            assert_eq!(vertex_err.status, "INVALID_ARGUMENT");
        }
        other => panic!("expected VertexError, got {other:?}"),
    }
    Ok(())
}

#[tokio::test]
async fn error_for_status_preserves_raw_body_on_unstructured_error() -> Result<()> {
    let raw_response = http::Response::builder()
        .status(502)
        .body(reqwest::Body::from("Bad Gateway"))?;
    let response: reqwest::Response = raw_response.into();

    let err = GeminiClient::error_for_status(response)
        .await
        .err()
        .ok_or("expected error")?;

    match err {
        GeminiError::ApiError { status, body } => {
            assert_eq!(status, reqwest::StatusCode::BAD_GATEWAY);
            assert_eq!(body, "Bad Gateway");
        }
        other => panic!("expected ApiError, got {other:?}"),
    }
    Ok(())
}

#[tokio::test]
async fn error_for_status_passes_through_success() -> Result<()> {
    let raw_response = http::Response::builder()
        .status(200)
        .body(reqwest::Body::from("{}"))?;
    let response: reqwest::Response = raw_response.into();

    let result = GeminiClient::error_for_status(response).await;
    assert!(result.is_ok());
    Ok(())
}
