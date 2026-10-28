// Copyright 2026 Andre Cipriani Bandarra
// SPDX-License-Identifier: Apache-2.0

use super::*;
use crate::network::event_source::ServerSentEvent;

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
    assert_eq!(
        cached_content_resource_url("cachedContents/abc123xyz"),
        "https://generativelanguage.googleapis.com/v1beta/cachedContents/abc123xyz"
    );
    assert_eq!(
        cached_content_resource_url("abc123xyz"),
        "https://generativelanguage.googleapis.com/v1beta/cachedContents/abc123xyz"
    );
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
