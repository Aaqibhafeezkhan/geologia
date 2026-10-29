// Copyright 2026 Andre Cipriani Bandarra
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};

use super::{Content, ModalityTokenCount};

/// Type of task for which an embedding will be used.
///
/// Specifying a task type helps the embedding model optimize the resulting vector for the
/// intended downstream use case.
///
/// See <https://ai.google.dev/api/embeddings#v1beta.TaskType>.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TaskType {
    /// Unset value, which defaults to one of the other enum values.
    #[serde(rename = "TASK_TYPE_UNSPECIFIED")]
    Unspecified,
    /// Specifies the given text is a query in a search/retrieval setting.
    RetrievalQuery,
    /// Specifies the given text is a document from the corpus being searched.
    RetrievalDocument,
    /// Specifies the given text will be used for semantic textual similarity (STS).
    SemanticSimilarity,
    /// Specifies that the given text will be classified.
    Classification,
    /// Specifies that the embeddings will be used for clustering.
    Clustering,
    /// Specifies that the given text will be used for question answering.
    QuestionAnswering,
    /// Specifies that the given text will be used for fact verification.
    FactVerification,
    /// Specifies that the given text will be used for code retrieval.
    CodeRetrievalQuery,
}

/// Request body for the `embedContent` endpoint.
///
/// Use [`EmbedContentRequest::builder`] or [`EmbedContentRequest::from_text`] for ergonomic
/// construction.
///
/// See <https://ai.google.dev/api/embeddings#request-body>.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmbedContentRequest {
    /// The content to embed. Only the `parts.text` fields are counted.
    pub content: Content,
    /// Optional task type for which the embeddings will be used.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_type: Option<TaskType>,
    /// Optional title for the text. Only applicable when `task_type` is
    /// [`TaskType::RetrievalDocument`].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Optional reduced dimension for the output embedding. If set, excessive values in the
    /// output embedding are truncated from the end.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_dimensionality: Option<i32>,
}

impl EmbedContentRequest {
    /// Returns a new [`EmbedContentRequestBuilder`].
    pub fn builder() -> EmbedContentRequestBuilder {
        EmbedContentRequestBuilder::default()
    }

    /// Creates an [`EmbedContentRequest`] containing a single text part with default options.
    pub fn from_text(text: impl Into<String>) -> Self {
        Self::builder().text(text).build()
    }
}

/// Builder for [`EmbedContentRequest`].
#[derive(Clone, Debug, Default)]
pub struct EmbedContentRequestBuilder {
    request: EmbedContentRequest,
}

impl EmbedContentRequestBuilder {
    /// Creates a builder pre-populated with a single text part.
    pub fn from_text(text: impl Into<String>) -> Self {
        Self::default().text(text)
    }

    /// Sets the [`Content`] to embed.
    pub fn content(mut self, content: Content) -> Self {
        self.request.content = content;
        self
    }

    /// Sets the content to embed from a single text string.
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.request.content = Content::builder().add_text_part(text).build();
        self
    }

    /// Sets the [`TaskType`] for which the embedding will be used.
    pub fn task_type(mut self, task_type: TaskType) -> Self {
        self.request.task_type = Some(task_type);
        self
    }

    /// Sets an optional title for the text (only applicable when `task_type` is
    /// [`TaskType::RetrievalDocument`]).
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.request.title = Some(title.into());
        self
    }

    /// Sets a reduced dimension for the output embedding vector.
    pub fn output_dimensionality(mut self, output_dimensionality: i32) -> Self {
        self.request.output_dimensionality = Some(output_dimensionality);
        self
    }

    /// Consumes the builder and returns the constructed [`EmbedContentRequest`].
    pub fn build(self) -> EmbedContentRequest {
        self.request
    }
}

/// A list of floats representing an embedding vector.
///
/// See <https://ai.google.dev/api/embeddings#v1beta.ContentEmbedding>.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ContentEmbedding {
    /// The embedding values.
    #[serde(default)]
    pub values: Vec<f32>,
}

/// Token usage metadata for an embedding request (`embedContent` or `batchEmbedContents`).
///
/// See <https://ai.google.dev/api/embeddings>.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct EmbeddingUsageMetadata {
    /// Number of tokens in the prompt.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_token_count: Option<u32>,
    /// Per-modality breakdown of tokens processed in the request input.
    #[serde(alias = "promptTokensDetails", skip_serializing_if = "Option::is_none")]
    pub prompt_token_details: Option<Vec<ModalityTokenCount>>,
}

/// Response from the `embedContent` endpoint.
///
/// See <https://ai.google.dev/api/embeddings#v1beta.EmbedContentResponse>.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct EmbedContentResponse {
    /// The embedding generated from the input content.
    pub embedding: ContentEmbedding,
    /// Token usage metadata for the request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage_metadata: Option<EmbeddingUsageMetadata>,
}

/// Request body for the `batchEmbedContents` endpoint.
///
/// The API requires each inner [`EmbedContentRequest`] to carry its `model` resource name
/// (e.g. `models/gemini-embedding-001`);
/// [`GeminiClient::batch_embed_contents`](crate::prelude::GeminiClient::batch_embed_contents)
/// populates it automatically from its `model` argument.
///
/// Use [`BatchEmbedContentsRequest::builder`] for ergonomic construction.
///
/// See <https://ai.google.dev/api/embeddings#request-body_1>.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchEmbedContentsRequest {
    /// Embed requests for the batch.
    pub requests: Vec<EmbedContentRequest>,
}

impl BatchEmbedContentsRequest {
    /// Returns a new [`BatchEmbedContentsRequestBuilder`].
    pub fn builder() -> BatchEmbedContentsRequestBuilder {
        BatchEmbedContentsRequestBuilder::default()
    }

    /// Returns the wire body for this request, with `model` (e.g. `models/gemini-embedding-001`)
    /// set on each inner [`EmbedContentRequest`] as the API requires.
    pub(crate) fn with_model<'a>(&'a self, model: &str) -> BatchEmbedContentsRequestBody<'a> {
        let model_name = if model.starts_with("models/") {
            model.to_string()
        } else {
            format!("models/{model}")
        };
        BatchEmbedContentsRequestBody {
            requests: self
                .requests
                .iter()
                .map(|request| ModelEmbedContentRequest {
                    model: model_name.clone(),
                    request,
                })
                .collect(),
        }
    }
}

/// Wire form of [`BatchEmbedContentsRequest`], carrying `model` on each inner request.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct BatchEmbedContentsRequestBody<'a> {
    requests: Vec<ModelEmbedContentRequest<'a>>,
}

#[derive(Debug, Serialize)]
struct ModelEmbedContentRequest<'a> {
    model: String,
    #[serde(flatten)]
    request: &'a EmbedContentRequest,
}

/// Builder for [`BatchEmbedContentsRequest`].
#[derive(Clone, Debug, Default)]
pub struct BatchEmbedContentsRequestBuilder {
    request: BatchEmbedContentsRequest,
}

impl BatchEmbedContentsRequestBuilder {
    /// Replaces the list of embed requests in the batch.
    pub fn requests(mut self, requests: Vec<EmbedContentRequest>) -> Self {
        self.request.requests = requests;
        self
    }

    /// Appends a single [`EmbedContentRequest`] to the batch.
    pub fn add_request(mut self, request: EmbedContentRequest) -> Self {
        self.request.requests.push(request);
        self
    }

    /// Consumes the builder and returns the constructed [`BatchEmbedContentsRequest`].
    pub fn build(self) -> BatchEmbedContentsRequest {
        self.request
    }
}

/// Response from the `batchEmbedContents` endpoint.
///
/// See <https://ai.google.dev/api/embeddings#response-body_1>.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BatchEmbedContentsResponse {
    /// The embeddings for each request, in the same order as provided in the batch request.
    #[serde(default)]
    pub embeddings: Vec<ContentEmbedding>,
    /// Token usage metadata for the batch request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage_metadata: Option<EmbeddingUsageMetadata>,
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::types::{Modality, Role};

    #[test]
    fn embed_content_request_omits_optional_fields_when_unset() -> serde_json::Result<()> {
        let request = EmbedContentRequest::from_text("What is the meaning of life?");

        let value = serde_json::to_value(&request)?;

        assert_eq!(
            value,
            json!({
                "content": {
                    "parts": [{ "text": "What is the meaning of life?" }]
                }
            })
        );
        Ok(())
    }

    #[test]
    fn embed_content_request_serializes_task_type_title_and_output_dimensionality()
    -> serde_json::Result<()> {
        let request = EmbedContentRequest::builder()
            .content(
                Content::builder()
                    .role(Role::User)
                    .add_text_part("Rust is a systems programming language.")
                    .build(),
            )
            .task_type(TaskType::RetrievalDocument)
            .title("Rust Overview")
            .output_dimensionality(256)
            .build();

        let value = serde_json::to_value(&request)?;

        assert_eq!(
            value,
            json!({
                "content": {
                    "role": "user",
                    "parts": [{ "text": "Rust is a systems programming language." }]
                },
                "taskType": "RETRIEVAL_DOCUMENT",
                "title": "Rust Overview",
                "outputDimensionality": 256
            })
        );
        Ok(())
    }

    #[test]
    fn task_type_round_trips_all_api_strings() -> serde_json::Result<()> {
        let cases = [
            (TaskType::Unspecified, "TASK_TYPE_UNSPECIFIED"),
            (TaskType::RetrievalQuery, "RETRIEVAL_QUERY"),
            (TaskType::RetrievalDocument, "RETRIEVAL_DOCUMENT"),
            (TaskType::SemanticSimilarity, "SEMANTIC_SIMILARITY"),
            (TaskType::Classification, "CLASSIFICATION"),
            (TaskType::Clustering, "CLUSTERING"),
            (TaskType::QuestionAnswering, "QUESTION_ANSWERING"),
            (TaskType::FactVerification, "FACT_VERIFICATION"),
            (TaskType::CodeRetrievalQuery, "CODE_RETRIEVAL_QUERY"),
        ];

        for (variant, wire) in cases {
            let serialized = serde_json::to_value(variant)?;
            assert_eq!(serialized, json!(wire));

            let deserialized: TaskType = serde_json::from_value(json!(wire))?;
            assert_eq!(deserialized, variant);
        }
        Ok(())
    }

    #[test]
    fn batch_embed_contents_request_sets_model_on_each_inner_request() -> serde_json::Result<()> {
        let batch = BatchEmbedContentsRequest::builder()
            .add_request(EmbedContentRequest::from_text("First prompt"))
            .add_request(
                EmbedContentRequest::builder()
                    .text("Second document")
                    .task_type(TaskType::RetrievalDocument)
                    .title("Doc 2")
                    .output_dimensionality(128)
                    .build(),
            )
            .build();

        let value = serde_json::to_value(batch.with_model("gemini-embedding-001"))?;

        assert_eq!(
            value,
            json!({
                "requests": [
                    {
                        "model": "models/gemini-embedding-001",
                        "content": {
                            "parts": [{ "text": "First prompt" }]
                        }
                    },
                    {
                        "model": "models/gemini-embedding-001",
                        "content": {
                            "parts": [{ "text": "Second document" }]
                        },
                        "taskType": "RETRIEVAL_DOCUMENT",
                        "title": "Doc 2",
                        "outputDimensionality": 128
                    }
                ]
            })
        );

        // Passing an already-prefixed model name does not double-prefix `models/`.
        let prefixed = serde_json::to_value(batch.with_model("models/gemini-embedding-001"))?;
        assert_eq!(prefixed, value);
        Ok(())
    }

    #[test]
    fn parses_embed_content_response_without_usage_metadata() -> serde_json::Result<()> {
        let raw = r#"{
            "embedding": {
                "values": [0.125, -0.5, 0.875]
            }
        }"#;

        let response: EmbedContentResponse = serde_json::from_str(raw)?;

        assert_eq!(response.embedding.values, vec![0.125, -0.5, 0.875]);
        assert!(response.usage_metadata.is_none());
        Ok(())
    }

    #[test]
    fn parses_embed_content_response_with_usage_metadata() -> serde_json::Result<()> {
        let raw = r#"{
            "embedding": {
                "values": [0.1, 0.2, 0.3, 0.4]
            },
            "usageMetadata": {
                "promptTokenCount": 6,
                "promptTokenDetails": [
                    { "modality": "TEXT", "tokenCount": 6 }
                ]
            }
        }"#;

        let response: EmbedContentResponse = serde_json::from_str(raw)?;

        assert_eq!(response.embedding.values, vec![0.1, 0.2, 0.3, 0.4]);
        assert_eq!(
            response.usage_metadata,
            Some(EmbeddingUsageMetadata {
                prompt_token_count: Some(6),
                prompt_token_details: Some(vec![ModalityTokenCount {
                    modality: Modality::Text,
                    token_count: 6,
                }]),
            })
        );
        Ok(())
    }

    #[test]
    fn parses_batch_embed_contents_response() -> serde_json::Result<()> {
        let raw = r#"{
            "embeddings": [
                { "values": [0.1, -0.2] },
                { "values": [0.3, 0.4] }
            ],
            "usageMetadata": {
                "promptTokenCount": 11
            }
        }"#;

        let response: BatchEmbedContentsResponse = serde_json::from_str(raw)?;

        assert_eq!(
            response.embeddings,
            vec![
                ContentEmbedding {
                    values: vec![0.1, -0.2]
                },
                ContentEmbedding {
                    values: vec![0.3, 0.4]
                },
            ]
        );
        assert_eq!(
            response.usage_metadata,
            Some(EmbeddingUsageMetadata {
                prompt_token_count: Some(11),
                prompt_token_details: None,
            })
        );
        Ok(())
    }

    #[test]
    fn parses_batch_embed_contents_response_without_usage_metadata() -> serde_json::Result<()> {
        let raw = r#"{
            "embeddings": [
                { "values": [0.5, 0.25, -0.75] }
            ]
        }"#;

        let response: BatchEmbedContentsResponse = serde_json::from_str(raw)?;

        assert_eq!(response.embeddings.len(), 1);
        assert_eq!(response.embeddings[0].values, vec![0.5, 0.25, -0.75]);
        assert!(response.usage_metadata.is_none());
        Ok(())
    }
}
