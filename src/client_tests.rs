// Copyright 2026 Andre Cipriani Bandarra
// SPDX-License-Identifier: Apache-2.0

use super::*;
use crate::network::event_source::ServerSentEvent;
use wiremock::matchers::{body_json, header, method, path, query_param};
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
    assert_eq!(
        client.model_resource_url("models/gemini-2.5-flash"),
        "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash"
    );
    assert_eq!(
        client.model_resource_url("gemini-2.5-flash"),
        "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash"
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
        client.endpoint_url("models/gemini-embedding-001:embedContent"),
        "https://custom.endpoint.example/v1/models/gemini-embedding-001:embedContent"
    );
    assert_eq!(
        client.endpoint_url("models/gemini-embedding-001:batchEmbedContents"),
        "https://custom.endpoint.example/v1/models/gemini-embedding-001:batchEmbedContents"
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
    assert_eq!(
        client.endpoint_url("models"),
        "https://custom.endpoint.example/v1/models"
    );
    assert_eq!(
        client.model_resource_url("models/gemini-2.5-flash"),
        "https://custom.endpoint.example/v1/models/gemini-2.5-flash"
    );
    assert_eq!(
        client.model_resource_url("gemini-2.5-flash"),
        "https://custom.endpoint.example/v1/models/gemini-2.5-flash"
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

    let response = client
        .generate_content(&request, "gemini-2.0-flash")
        .await?;
    let candidate = response.candidates.first().ok_or("expected candidate")?;
    assert_eq!(
        candidate.get_text().as_deref(),
        Some("Hello from mock server!")
    );
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

    let response = client
        .generate_content(&request, "gemini-2.0-flash")
        .await?;
    let candidate = response.candidates.first().ok_or("expected candidate")?;
    assert_eq!(
        candidate.get_text().as_deref(),
        Some("Custom HTTP client used!")
    );
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
async fn embed_content_against_mock_server_checks_path_body_and_response() -> Result<()> {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/models/gemini-embedding-001:embedContent"))
        .and(header("x-goog-api-key", "test-api-key"))
        .and(body_json(serde_json::json!({
            "content": {
                "parts": [{"text": "Embed this query"}]
            },
            "taskType": "RETRIEVAL_QUERY",
            "outputDimensionality": 3
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "embedding": {
                "values": [0.1, 0.2, 0.3]
            },
            "usageMetadata": {
                "promptTokenCount": 3
            }
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = GeminiClient::builder("test-api-key")
        .base_url(mock_server.uri())
        .build();

    let request = EmbedContentRequest::builder()
        .text("Embed this query")
        .task_type(TaskType::RetrievalQuery)
        .output_dimensionality(3)
        .build();

    let response = client
        .embed_content(&request, "gemini-embedding-001")
        .await?;
    assert_eq!(response.embedding.values, vec![0.1, 0.2, 0.3]);
    assert_eq!(
        response
            .usage_metadata
            .ok_or("expected usage_metadata")?
            .prompt_token_count,
        Some(3)
    );
    Ok(())
}

#[tokio::test]
async fn batch_embed_contents_against_mock_server_sets_inner_models_and_parses_response()
-> Result<()> {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/models/gemini-embedding-001:batchEmbedContents"))
        .and(header("x-goog-api-key", "test-api-key"))
        .and(body_json(serde_json::json!({
            "requests": [
                {
                    "model": "models/gemini-embedding-001",
                    "content": {
                        "parts": [{"text": "First document"}]
                    },
                    "taskType": "RETRIEVAL_DOCUMENT",
                    "title": "Doc 1"
                },
                {
                    "model": "models/gemini-embedding-001",
                    "content": {
                        "parts": [{"text": "Second document"}]
                    }
                }
            ]
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "embeddings": [
                {"values": [0.5, -0.5]},
                {"values": [0.25, 0.75]}
            ],
            "usageMetadata": {
                "promptTokenCount": 5
            }
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = GeminiClient::builder("test-api-key")
        .base_url(mock_server.uri())
        .build();

    let batch = BatchEmbedContentsRequest::builder()
        .add_request(
            EmbedContentRequest::builder()
                .text("First document")
                .task_type(TaskType::RetrievalDocument)
                .title("Doc 1")
                .build(),
        )
        .add_request(EmbedContentRequest::from_text("Second document"))
        .build();

    // Passing `models/gemini-embedding-001` also exercises prefix normalization.
    let response = client
        .batch_embed_contents(&batch, "models/gemini-embedding-001")
        .await?;
    assert_eq!(response.embeddings.len(), 2);
    assert_eq!(response.embeddings[0].values, vec![0.5, -0.5]);
    assert_eq!(response.embeddings[1].values, vec![0.25, 0.75]);
    assert_eq!(
        response
            .usage_metadata
            .ok_or("expected usage_metadata")?
            .prompt_token_count,
        Some(5)
    );
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

#[tokio::test]
async fn stream_yields_ok_then_err_on_mid_stream_io_error() -> Result<()> {
    use tokio_util::bytes::Bytes;

    let chunks: Vec<std::result::Result<Bytes, std::io::Error>> = vec![
        Ok(Bytes::from_static(
            b"data: {\"candidates\":[{\"index\":0,\"content\":{\"parts\":[{\"text\":\"Hello\"}],\"role\":\"model\"}}]}\n\n",
        )),
        Err(std::io::Error::new(
            std::io::ErrorKind::ConnectionReset,
            "connection reset mid-stream",
        )),
    ];
    let body = reqwest::Body::wrap_stream(tokio_stream::iter(chunks));
    let response: reqwest::Response = http::Response::builder().status(200).body(body)?.into();

    let mut stream = std::pin::pin!(GeminiClient::response_event_stream(response));

    let first = stream
        .next()
        .await
        .ok_or("expected first Ok item from stream")??;
    assert_eq!(
        first
            .candidates
            .first()
            .and_then(|c| c.get_text())
            .as_deref(),
        Some("Hello")
    );

    let second = stream
        .next()
        .await
        .ok_or("expected Err item after mid-stream I/O error")?;
    match second {
        Err(GeminiError::EventSourceError(LinesCodecError::Io(ref io_err))) => {
            let mut has_root_cause = false;
            let mut current = std::error::Error::source(io_err);
            while let Some(cause) = current {
                if cause.to_string().contains("connection reset mid-stream") {
                    has_root_cause = true;
                    break;
                }
                current = cause.source();
            }
            assert!(
                has_root_cause,
                "expected I/O error source chain to contain root cause, got {io_err:?}"
            );
        }
        other => panic!("expected Err(EventSourceError(LinesCodecError::Io(_))), got {other:?}"),
    }

    assert!(stream.next().await.is_none());
    Ok(())
}

#[tokio::test]
async fn stream_yields_err_when_io_error_interrupts_partial_event() -> Result<()> {
    use tokio_util::bytes::Bytes;

    let chunks: Vec<std::result::Result<Bytes, std::io::Error>> = vec![
        Ok(Bytes::from_static(
            b"data: {\"candidates\":[{\"index\":0,\"content\":{\"parts\":[{\"text\":\"Part 1\"}],\"role\":\"model\"}}]}\n\n",
        )),
        Ok(Bytes::from_static(b"data: {\"candidates\":[{\"index\":0")),
        Err(std::io::Error::new(
            std::io::ErrorKind::UnexpectedEof,
            "stream truncated mid-event",
        )),
    ];
    let body = reqwest::Body::wrap_stream(tokio_stream::iter(chunks));
    let response: reqwest::Response = http::Response::builder().status(200).body(body)?.into();

    let mut stream = std::pin::pin!(GeminiClient::response_event_stream(response));

    let first = stream
        .next()
        .await
        .ok_or("expected first Ok item from stream")??;
    assert_eq!(
        first
            .candidates
            .first()
            .and_then(|c| c.get_text())
            .as_deref(),
        Some("Part 1")
    );

    let second = stream
        .next()
        .await
        .ok_or("expected Err item after truncated partial event")?;
    assert!(matches!(
        second,
        Err(GeminiError::EventSourceError(LinesCodecError::Io(_)))
    ));
    Ok(())
}

#[tokio::test]
async fn stream_skips_events_without_data_line_without_error() -> Result<()> {
    use tokio_util::bytes::Bytes;

    assert!(
        GeminiClient::parse_event(Ok(ServerSentEvent {
            event: Some("ping".into()),
            data: None,
            id: Some("1".into()),
            retry: Some(1000),
        }))
        .is_none()
    );

    let chunks: Vec<std::result::Result<Bytes, std::io::Error>> = vec![Ok(Bytes::from_static(
        b": keep-alive\n\nevent: ping\nid: 1\nretry: 1000\n\n\ndata: {\"candidates\":[{\"index\":0,\"content\":{\"parts\":[{\"text\":\"Done\"}],\"role\":\"model\"},\"finishReason\":\"STOP\"}]}\n\n",
    ))];
    let body = reqwest::Body::wrap_stream(tokio_stream::iter(chunks));
    let response: reqwest::Response = http::Response::builder().status(200).body(body)?.into();

    let mut stream = std::pin::pin!(GeminiClient::response_event_stream(response));

    let item = stream
        .next()
        .await
        .ok_or("expected Ok item after skipped no-data events")??;
    let candidate = item.candidates.first().ok_or("expected candidate")?;
    assert_eq!(candidate.get_text().as_deref(), Some("Done"));
    assert_eq!(candidate.finish_reason.as_deref(), Some("STOP"));

    assert!(stream.next().await.is_none());
    Ok(())
}

#[test]
fn parse_event_surfaces_codec_max_line_length_error() {
    let result = GeminiClient::parse_event(Err(LinesCodecError::MaxLineLengthExceeded));
    assert!(matches!(
        result,
        Some(Err(GeminiError::EventSourceError(
            LinesCodecError::MaxLineLengthExceeded
        )))
    ));
}

#[tokio::test]
async fn stream_generate_content_against_mock_server_skips_keepalives_and_yields_chunks()
-> Result<()> {
    let mock_server = MockServer::start().await;

    let sse_body = concat!(
        ": keep-alive\n\n",
        "data: {\"candidates\":[{\"index\":0,\"content\":{\"parts\":[{\"text\":\"Hello, \"}],\"role\":\"model\"}}]}\n\n",
        "event: ping\n\n",
        "data: {\"candidates\":[{\"index\":0,\"content\":{\"parts\":[{\"text\":\"world!\"}],\"role\":\"model\"},\"finishReason\":\"STOP\"}]}\n\n",
    );

    Mock::given(method("POST"))
        .and(path("/models/gemini-2.0-flash:streamGenerateContent"))
        .and(query_param("alt", "sse"))
        .and(header("x-goog-api-key", "test-api-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(sse_body, "text/event-stream; charset=utf-8"),
        )
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = GeminiClient::builder("test-api-key")
        .base_url(mock_server.uri())
        .build();

    let request = GenerateContentRequest::builder()
        .contents(vec![Content::builder().add_text_part("Greet me").build()])
        .build();

    let stream = client
        .stream_generate_content(&request, "gemini-2.0-flash")
        .await?;
    let mut stream = std::pin::pin!(stream);

    let first = stream.next().await.ok_or("expected first chunk")??;
    assert_eq!(
        first
            .candidates
            .first()
            .and_then(|c| c.get_text())
            .as_deref(),
        Some("Hello, ")
    );
    assert_eq!(
        first
            .candidates
            .first()
            .and_then(|c| c.finish_reason.as_deref()),
        None
    );

    let second = stream.next().await.ok_or("expected second chunk")??;
    assert_eq!(
        second
            .candidates
            .first()
            .and_then(|c| c.get_text())
            .as_deref(),
        Some("world!")
    );
    assert_eq!(
        second
            .candidates
            .first()
            .and_then(|c| c.finish_reason.as_deref()),
        Some("STOP")
    );

    assert!(stream.next().await.is_none());
    Ok(())
}

#[test]
fn encode_query_value_percent_encodes_reserved_characters() {
    assert_eq!(encode_query_value("plain-token_1.0~"), "plain-token_1.0~");
    assert_eq!(
        encode_query_value("ChFtb2RlbHM+1/2==&x=y"),
        "ChFtb2RlbHM%2B1%2F2%3D%3D%26x%3Dy"
    );
}

#[tokio::test]
async fn list_models_against_mock_server_without_pagination_params() -> Result<()> {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/models"))
        .and(header("x-goog-api-key", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "models": [
                {
                    "name": "models/gemini-2.5-flash",
                    "version": "001",
                    "displayName": "Gemini 2.5 Flash",
                    "inputTokenLimit": 1_048_576,
                    "outputTokenLimit": 65_536,
                    "supportedGenerationMethods": ["generateContent", "countTokens"],
                    "thinking": true
                }
            ]
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = GeminiClient::builder("test-api-key")
        .base_url(mock_server.uri())
        .build();

    let response = client.list_models(None, None).await?;
    assert_eq!(response.models.len(), 1);
    assert_eq!(response.models[0].name, "models/gemini-2.5-flash");
    assert_eq!(response.models[0].input_token_limit, Some(1_048_576));
    assert_eq!(response.models[0].output_token_limit, Some(65_536));
    assert_eq!(response.models[0].thinking, Some(true));
    assert!(response.next_page_token.is_none());
    Ok(())
}

#[tokio::test]
async fn list_models_against_mock_server_with_pagination_params() -> Result<()> {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/models"))
        .and(query_param("pageSize", "2"))
        .and(query_param("pageToken", "tok+1/2=="))
        .and(header("x-goog-api-key", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "models": [
                {
                    "name": "models/gemini-2.5-pro",
                    "displayName": "Gemini 2.5 Pro",
                    "supportedGenerationMethods": ["generateContent"]
                },
                {
                    "name": "models/gemini-embedding-001",
                    "displayName": "Gemini Embedding 001",
                    "supportedGenerationMethods": ["embedContent"]
                }
            ],
            "nextPageToken": "next-tok=="
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = GeminiClient::builder("test-api-key")
        .base_url(mock_server.uri())
        .build();

    let response = client.list_models(Some(2), Some("tok+1/2==")).await?;
    assert_eq!(response.models.len(), 2);
    assert_eq!(response.models[0].name, "models/gemini-2.5-pro");
    assert_eq!(response.models[1].name, "models/gemini-embedding-001");
    assert_eq!(response.next_page_token.as_deref(), Some("next-tok=="));
    Ok(())
}

#[tokio::test]
async fn get_model_against_mock_server_handles_prefixed_and_unprefixed_name() -> Result<()> {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/models/gemini-2.5-flash"))
        .and(header("x-goog-api-key", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "name": "models/gemini-2.5-flash",
            "baseModelId": "gemini-2.5-flash",
            "version": "001",
            "displayName": "Gemini 2.5 Flash",
            "description": "Fast multimodal model.",
            "inputTokenLimit": 1_048_576,
            "outputTokenLimit": 65_536,
            "supportedGenerationMethods": ["generateContent", "countTokens"],
            "temperature": 1.0,
            "maxTemperature": 2.0,
            "topP": 0.95,
            "topK": 64,
            "thinking": true
        })))
        .expect(2)
        .mount(&mock_server)
        .await;

    let client = GeminiClient::builder("test-api-key")
        .base_url(mock_server.uri())
        .build();

    let unprefixed = client.get_model("gemini-2.5-flash").await?;
    let prefixed = client.get_model("models/gemini-2.5-flash").await?;

    assert_eq!(unprefixed, prefixed);
    assert_eq!(unprefixed.name, "models/gemini-2.5-flash");
    assert_eq!(
        unprefixed.base_model_id.as_deref(),
        Some("gemini-2.5-flash")
    );
    assert_eq!(unprefixed.version.as_deref(), Some("001"));
    assert_eq!(unprefixed.display_name.as_deref(), Some("Gemini 2.5 Flash"));
    assert_eq!(
        unprefixed.description.as_deref(),
        Some("Fast multimodal model.")
    );
    assert_eq!(unprefixed.input_token_limit, Some(1_048_576));
    assert_eq!(unprefixed.output_token_limit, Some(65_536));
    assert_eq!(
        unprefixed.supported_generation_methods,
        vec!["generateContent", "countTokens"]
    );
    assert_eq!(unprefixed.temperature, Some(1.0));
    assert_eq!(unprefixed.max_temperature, Some(2.0));
    assert_eq!(unprefixed.top_p, Some(0.95));
    assert_eq!(unprefixed.top_k, Some(64));
    assert_eq!(unprefixed.thinking, Some(true));
    Ok(())
}

#[tokio::test]
async fn get_model_maps_not_found_error() -> Result<()> {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/models/does-not-exist"))
        .and(header("x-goog-api-key", "test-api-key"))
        .respond_with(ResponseTemplate::new(404).set_body_json(serde_json::json!({
            "error": {
                "code": 404,
                "message": "models/does-not-exist is not found for API version v1beta",
                "status": "NOT_FOUND"
            }
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = GeminiClient::builder("test-api-key")
        .base_url(mock_server.uri())
        .build();

    let err = client
        .get_model("does-not-exist")
        .await
        .err()
        .ok_or("expected 404 error")?;

    match err {
        GeminiError::VertexError(vertex_err) => {
            assert_eq!(vertex_err.code, 404);
            assert_eq!(vertex_err.status, "NOT_FOUND");
        }
        other => panic!("expected VertexError, got {other:?}"),
    }
    Ok(())
}
