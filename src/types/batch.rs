// Copyright 2026 Andre Cipriani Bandarra
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::types::common::opt_int64_str;
use crate::types::{GenerateContentRequest, GenerateContentResponse, Status};

/// Request envelope for creating a batch of content generation requests via
/// `POST /v1beta/models/{model}:batchGenerateContent`.
///
/// See <https://ai.google.dev/gemini-api/docs/batch-mode>.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchGenerateContentRequest {
    /// The batch configuration and input to create.
    pub batch: GenerateContentBatch,
}

impl BatchGenerateContentRequest {
    /// Creates a new [`BatchGenerateContentRequest`] wrapping the given [`GenerateContentBatch`].
    pub fn new(batch: GenerateContentBatch) -> Self {
        Self { batch }
    }
}

/// A batch processing job representing asynchronous execution of `generateContent` requests.
///
/// Batch generation runs large workloads asynchronously at reduced cost. Processing lifecycle
/// transitions through [`BatchState::Pending`] -> [`BatchState::Running`] ->
/// [`BatchState::Succeeded`] (or [`BatchState::Failed`] / [`BatchState::Cancelled`]).
///
/// Use [`GenerateContentBatch::builder`] for ergonomic construction.
///
/// See <https://ai.google.dev/api/batch>.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerateContentBatch {
    /// Resource name of the batch job, formatted as `batches/{batchId}`. Output only.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Name of the model to use for completion, formatted as `models/{model}`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// User-defined friendly name for this batch job.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    /// Input configuration specifying either inline requests or an uploaded input file.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_config: Option<InputConfig>,
    /// Output results of the batch job once completed. Output only.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<GenerateContentBatchOutput>,
    /// Current execution state of the batch. Output only.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<BatchState>,
    /// Statistics and counters about batch request progress. Output only.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub batch_stats: Option<BatchStats>,
    /// Optional priority for batch execution. Higher values are processed before lower values.
    #[serde(
        default,
        with = "opt_int64_str",
        skip_serializing_if = "Option::is_none"
    )]
    pub priority: Option<i64>,
    /// RFC 3339 timestamp of when the batch was created. Output only.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub create_time: Option<String>,
    /// RFC 3339 timestamp of when the batch was last updated. Output only.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub update_time: Option<String>,
    /// RFC 3339 timestamp of when the batch finished processing. Output only.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_time: Option<String>,
}

impl GenerateContentBatch {
    /// Returns a new [`GenerateContentBatchBuilder`].
    pub fn builder() -> GenerateContentBatchBuilder {
        GenerateContentBatchBuilder::default()
    }
}

/// Builder for [`GenerateContentBatch`].
#[derive(Clone, Debug, Default)]
pub struct GenerateContentBatchBuilder {
    batch: GenerateContentBatch,
}

impl GenerateContentBatchBuilder {
    /// Sets the resource name (`batches/{id}` or `{id}`).
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.batch.name = Some(name.into());
        self
    }

    /// Sets the model name (`models/{model}` or `{model}`).
    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.batch.model = Some(model.into());
        self
    }

    /// Sets the user-defined display name.
    pub fn display_name(mut self, display_name: impl Into<String>) -> Self {
        self.batch.display_name = Some(display_name.into());
        self
    }

    /// Sets the input configuration.
    pub fn input_config(mut self, input_config: InputConfig) -> Self {
        self.batch.input_config = Some(input_config);
        self
    }

    /// Sets the batch output.
    pub fn output(mut self, output: GenerateContentBatchOutput) -> Self {
        self.batch.output = Some(output);
        self
    }

    /// Sets the execution state.
    pub fn state(mut self, state: BatchState) -> Self {
        self.batch.state = Some(state);
        self
    }

    /// Sets the batch statistics.
    pub fn batch_stats(mut self, batch_stats: BatchStats) -> Self {
        self.batch.batch_stats = Some(batch_stats);
        self
    }

    /// Sets the batch priority.
    pub fn priority(mut self, priority: i64) -> Self {
        self.batch.priority = Some(priority);
        self
    }

    /// Sets the creation timestamp (RFC 3339 format).
    pub fn create_time(mut self, create_time: impl Into<String>) -> Self {
        self.batch.create_time = Some(create_time.into());
        self
    }

    /// Sets the last update timestamp (RFC 3339 format).
    pub fn update_time(mut self, update_time: impl Into<String>) -> Self {
        self.batch.update_time = Some(update_time.into());
        self
    }

    /// Sets the completion timestamp (RFC 3339 format).
    pub fn end_time(mut self, end_time: impl Into<String>) -> Self {
        self.batch.end_time = Some(end_time.into());
        self
    }

    /// Consumes the builder and returns the constructed [`GenerateContentBatch`].
    pub fn build(self) -> GenerateContentBatch {
        self.batch
    }
}

/// Execution lifecycle state of a [`GenerateContentBatch`].
///
/// See <https://ai.google.dev/api/batch#State>.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BatchState {
    /// The batch state is unspecified.
    #[serde(rename = "BATCH_STATE_UNSPECIFIED")]
    Unspecified,
    /// The service is preparing to run the batch.
    #[serde(rename = "BATCH_STATE_PENDING", alias = "PENDING")]
    Pending,
    /// The batch is in progress.
    #[serde(rename = "BATCH_STATE_RUNNING", alias = "RUNNING")]
    Running,
    /// The batch completed successfully.
    #[serde(rename = "BATCH_STATE_SUCCEEDED", alias = "SUCCEEDED")]
    Succeeded,
    /// The batch failed during execution.
    #[serde(rename = "BATCH_STATE_FAILED", alias = "FAILED")]
    Failed,
    /// The batch was cancelled by the caller.
    #[serde(rename = "BATCH_STATE_CANCELLED", alias = "CANCELLED")]
    Cancelled,
    /// The batch expired before completion.
    #[serde(rename = "BATCH_STATE_EXPIRED", alias = "EXPIRED")]
    Expired,
}

/// Configuration detailing the input instances on which batch generation runs.
///
/// Either `file_name` (referencing an uploaded JSONL file via Files API) or `requests` (inline)
/// should be provided.
///
/// See <https://ai.google.dev/api/batch#InputConfig>.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InputConfig {
    /// The resource name of a `File` (e.g. `files/abc-123`) containing the input requests in JSONL format.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_name: Option<String>,
    /// Inline requests to be processed in the batch (recommended for total payloads under 20MB).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requests: Option<InlinedRequests>,
}

impl InputConfig {
    /// Creates an [`InputConfig`] referencing an uploaded input file by name or ID.
    pub fn from_file(file_name: impl Into<String>) -> Self {
        Self {
            file_name: Some(file_name.into()),
            requests: None,
        }
    }

    /// Creates an [`InputConfig`] wrapping [`InlinedRequests`].
    pub fn from_requests(requests: impl Into<InlinedRequests>) -> Self {
        Self {
            file_name: None,
            requests: Some(requests.into()),
        }
    }

    /// Creates an [`InputConfig`] from an iterator of [`GenerateContentRequest`]s.
    pub fn from_inline(requests: impl IntoIterator<Item = GenerateContentRequest>) -> Self {
        Self::from_requests(InlinedRequests::from_inline(requests))
    }
}

/// List of requests to be processed in the batch when provided inline.
///
/// See <https://ai.google.dev/api/batch#InlinedRequests>.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InlinedRequests {
    /// The requests to be processed in the batch.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub requests: Vec<InlinedRequest>,
}

impl InlinedRequests {
    /// Creates a new [`InlinedRequests`] collection.
    pub fn new(requests: Vec<InlinedRequest>) -> Self {
        Self { requests }
    }

    /// Creates an [`InlinedRequests`] collection from an iterator of [`GenerateContentRequest`]s.
    pub fn from_inline(requests: impl IntoIterator<Item = GenerateContentRequest>) -> Self {
        Self {
            requests: requests.into_iter().map(InlinedRequest::from).collect(),
        }
    }
}

impl From<Vec<InlinedRequest>> for InlinedRequests {
    fn from(requests: Vec<InlinedRequest>) -> Self {
        Self { requests }
    }
}

impl From<Vec<GenerateContentRequest>> for InlinedRequests {
    fn from(requests: Vec<GenerateContentRequest>) -> Self {
        Self::from_inline(requests)
    }
}

/// An individual content generation request to be processed in a batch, with optional caller metadata.
///
/// See <https://ai.google.dev/api/batch#InlinedRequest>.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InlinedRequest {
    /// The generation request to process.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request: Option<GenerateContentRequest>,
    /// Optional arbitrary metadata associated with this request and returned alongside its response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, Value>>,
}

impl InlinedRequest {
    /// Creates a new [`InlinedRequest`] wrapping the given [`GenerateContentRequest`].
    pub fn new(request: GenerateContentRequest) -> Self {
        Self {
            request: Some(request),
            metadata: None,
        }
    }

    /// Sets caller metadata for this request.
    pub fn with_metadata(mut self, metadata: HashMap<String, Value>) -> Self {
        self.metadata = Some(metadata);
        self
    }
}

impl From<GenerateContentRequest> for InlinedRequest {
    fn from(request: GenerateContentRequest) -> Self {
        Self::new(request)
    }
}

/// The output of a completed batch generation request.
///
/// See <https://ai.google.dev/api/batch#GenerateContentBatchOutput>.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerateContentBatchOutput {
    /// The file ID or resource name of the JSONL file containing the responses when file-based output is used.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub responses_file: Option<String>,
    /// Inline responses returned when the batch was submitted using inlined requests.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inlined_responses: Option<InlinedResponses>,
}

/// The responses to the requests in the batch when inline output is used.
///
/// See <https://ai.google.dev/api/batch#InlinedResponses>.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InlinedResponses {
    /// The list of responses in the same order as the input requests.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub inlined_responses: Vec<InlinedResponse>,
}

/// The response to a single request in an inline batch.
///
/// See <https://ai.google.dev/api/batch#InlinedResponse>.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InlinedResponse {
    /// The generated response if execution succeeded.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response: Option<GenerateContentResponse>,
    /// The metadata associated with the input request, returned verbatim.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, Value>>,
    /// The error status encountered if this specific request failed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<Status>,
}

impl InlinedResponse {
    /// Returns the successful [`GenerateContentResponseResult`](crate::types::GenerateContentResponseResult) if response is present and succeeded.
    pub fn result(&self) -> Option<&crate::types::GenerateContentResponseResult> {
        match self.response.as_ref() {
            Some(GenerateContentResponse::Ok(result)) => Some(result),
            _ => None,
        }
    }
}

/// Statistics detailing progress and counts for a batch job.
///
/// See <https://ai.google.dev/api/batch#BatchStats>.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct BatchStats {
    /// Total number of requests in the batch.
    #[serde(
        default,
        with = "opt_int64_str",
        skip_serializing_if = "Option::is_none"
    )]
    pub request_count: Option<i64>,
    /// Number of requests that were successfully processed.
    #[serde(
        default,
        with = "opt_int64_str",
        skip_serializing_if = "Option::is_none"
    )]
    pub successful_request_count: Option<i64>,
    /// Number of requests that failed processing.
    #[serde(
        default,
        with = "opt_int64_str",
        skip_serializing_if = "Option::is_none"
    )]
    pub failed_request_count: Option<i64>,
    /// Number of requests currently pending execution.
    #[serde(
        default,
        with = "opt_int64_str",
        skip_serializing_if = "Option::is_none"
    )]
    pub pending_request_count: Option<i64>,
}

/// Long-running operation envelope returned by batch endpoints (`models:batchGenerateContent`
/// and `batches/{name}`).
///
/// The inner [`GenerateContentBatch`] is contained in `metadata` while the batch is pending,
/// running, or complete. Upon completion, `done` is set to `true` and the output is accessible
/// via `response` or `metadata.output`.
///
/// See <https://ai.google.dev/api/batch>.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchOperation {
    /// Server-assigned operation name, formatted as `batches/{batchId}`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Service-specific metadata detailing batch configuration, progress, and output.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<GenerateContentBatch>,
    /// `true` once the batch has finished processing (either successfully or with error).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub done: Option<bool>,
    /// Logical error status if the overall operation failed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<Status>,
    /// Final output of the batch operation upon successful completion.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response: Option<GenerateContentBatchOutput>,
}

impl BatchOperation {
    /// Returns a reference to the inner [`GenerateContentBatch`] metadata if present.
    pub fn batch(&self) -> Option<&GenerateContentBatch> {
        self.metadata.as_ref()
    }

    /// Returns `true` if the batch has finished processing.
    pub fn is_done(&self) -> bool {
        self.done.unwrap_or(false)
    }

    /// Returns the current lifecycle [`BatchState`] of the batch.
    pub fn state(&self) -> Option<BatchState> {
        self.metadata.as_ref().and_then(|m| m.state)
    }

    /// Returns the output of the completed batch if present (from either `response` or `metadata.output`).
    pub fn output(&self) -> Option<&GenerateContentBatchOutput> {
        self.response
            .as_ref()
            .or_else(|| self.metadata.as_ref().and_then(|m| m.output.as_ref()))
    }

    /// Returns the slice of inlined responses if inline output is present.
    pub fn inlined_responses(&self) -> Option<&[InlinedResponse]> {
        self.output()
            .and_then(|o| o.inlined_responses.as_ref())
            .map(|ir| ir.inlined_responses.as_slice())
    }
}

/// Paginated list of batch operations returned by `list_batches`.
///
/// See <https://ai.google.dev/api/batch#methods>.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListBatchesResponse {
    /// The list of batch operations on this page.
    #[serde(default, alias = "batches", skip_serializing_if = "Vec::is_empty")]
    pub operations: Vec<BatchOperation>,
    /// A token that can be passed as `page_token` to retrieve the next page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_page_token: Option<String>,
}

impl ListBatchesResponse {
    /// Returns an iterator over references to the inner [`GenerateContentBatch`] metadata for all operations.
    pub fn batches(&self) -> impl Iterator<Item = &GenerateContentBatch> {
        self.operations.iter().filter_map(|op| op.batch())
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::prelude::{
        BatchGenerateContentRequest as PreludeBatchGenerateContentRequest,
        BatchOperation as PreludeBatchOperation, BatchState as PreludeBatchState, Content,
        GenerateContentRequest, Role,
    };

    use super::*;

    #[test]
    fn batch_create_request_with_inlined_requests_round_trips() -> serde_json::Result<()> {
        let request = BatchGenerateContentRequest::new(
            GenerateContentBatch::builder()
                .display_name("my-batch-requests")
                .input_config(InputConfig::from_inline(vec![
                    GenerateContentRequest::builder()
                        .contents(vec![
                            Content::builder()
                                .role(Role::User)
                                .add_text_part("Describe photosynthesis.")
                                .build(),
                        ])
                        .build(),
                ]))
                .build(),
        );

        let value = serde_json::to_value(&request)?;
        assert_eq!(
            value,
            json!({
                "batch": {
                    "displayName": "my-batch-requests",
                    "inputConfig": {
                        "requests": {
                            "requests": [
                                {
                                    "request": {
                                        "contents": [
                                            {
                                                "role": "user",
                                                "parts": [{ "text": "Describe photosynthesis." }]
                                            }
                                        ]
                                    }
                                }
                            ]
                        }
                    }
                }
            })
        );

        let parsed: PreludeBatchGenerateContentRequest = serde_json::from_value(value.clone())?;
        assert_eq!(serde_json::to_value(&parsed)?, value);
        assert_eq!(
            parsed.batch.display_name.as_deref(),
            Some("my-batch-requests")
        );
        Ok(())
    }

    #[test]
    fn batch_create_request_with_file_name_round_trips() -> serde_json::Result<()> {
        let request = BatchGenerateContentRequest::new(
            GenerateContentBatch::builder()
                .display_name("file-batch")
                .input_config(InputConfig::from_file("files/123456"))
                .priority(5)
                .build(),
        );

        let value = serde_json::to_value(&request)?;
        assert_eq!(
            value,
            json!({
                "batch": {
                    "displayName": "file-batch",
                    "inputConfig": {
                        "fileName": "files/123456"
                    },
                    "priority": "5"
                }
            })
        );

        let parsed: BatchGenerateContentRequest = serde_json::from_value(value.clone())?;
        assert_eq!(serde_json::to_value(&parsed)?, value);
        assert_eq!(parsed.batch.priority, Some(5));
        Ok(())
    }

    #[test]
    fn parses_in_progress_batch_operation() -> serde_json::Result<()> {
        let payload = json!({
            "name": "batches/abc-123",
            "metadata": {
                "@type": "type.googleapis.com/google.ai.generativelanguage.v1main.GenerateContentBatch",
                "name": "batches/abc-123",
                "model": "models/gemini-2.5-flash",
                "displayName": "test-batch",
                "state": "BATCH_STATE_RUNNING",
                "createTime": "2026-10-30T19:00:00Z",
                "updateTime": "2026-10-30T19:05:00Z",
                "batchStats": {
                    "requestCount": "10",
                    "pendingRequestCount": "8",
                    "successfulRequestCount": "2"
                }
            }
        });

        let op: PreludeBatchOperation = serde_json::from_value(payload)?;
        assert_eq!(op.name.as_deref(), Some("batches/abc-123"));
        assert!(!op.is_done());
        assert_eq!(op.state(), Some(PreludeBatchState::Running));

        let batch = op.batch().expect("batch metadata must be present");
        assert_eq!(batch.model.as_deref(), Some("models/gemini-2.5-flash"));
        assert_eq!(batch.display_name.as_deref(), Some("test-batch"));

        let stats = batch.batch_stats.as_ref().expect("stats must be present");
        assert_eq!(stats.request_count, Some(10));
        assert_eq!(stats.pending_request_count, Some(8));
        assert_eq!(stats.successful_request_count, Some(2));
        assert_eq!(stats.failed_request_count, None);
        Ok(())
    }

    #[test]
    fn parses_completed_batch_operation_with_inline_results() -> serde_json::Result<()> {
        let payload = json!({
            "name": "batches/6dv854jmuytqk8t6uom7zhdxn81t70b6evjq",
            "metadata": {
                "@type": "type.googleapis.com/google.ai.generativelanguage.v1main.GenerateContentBatch",
                "model": "models/gemini-2.5-flash",
                "displayName": "test-batch",
                "createTime": "2026-10-30T19:43:34.521066624Z",
                "updateTime": "2026-10-30T19:44:59.981411212Z",
                "endTime": "2026-10-30T19:44:59.981411178Z",
                "batchStats": {
                    "requestCount": "1",
                    "successfulRequestCount": "1"
                },
                "state": "BATCH_STATE_SUCCEEDED",
                "name": "batches/6dv854jmuytqk8t6uom7zhdxn81t70b6evjq"
            },
            "done": true,
            "response": {
                "@type": "type.googleapis.com/google.ai.generativelanguage.v1main.GenerateContentBatchOutput",
                "inlinedResponses": {
                    "inlinedResponses": [
                        {
                            "response": {
                                "candidates": [
                                    {
                                        "content": {
                                            "parts": [{ "text": "Hi!" }],
                                            "role": "model"
                                        },
                                        "finishReason": "STOP",
                                        "index": 0
                                    }
                                ],
                                "modelVersion": "gemini-2.5-flash"
                            }
                        }
                    ]
                }
            }
        });

        let op: BatchOperation = serde_json::from_value(payload)?;
        assert_eq!(
            op.name.as_deref(),
            Some("batches/6dv854jmuytqk8t6uom7zhdxn81t70b6evjq")
        );
        assert!(op.is_done());
        assert_eq!(op.state(), Some(BatchState::Succeeded));

        let responses = op.inlined_responses().expect("inlined responses");
        assert_eq!(responses.len(), 1);
        let first_resp = responses[0].result().expect("candidate response");
        assert_eq!(
            first_resp.candidates[0]
                .content
                .as_ref()
                .and_then(|c| c.get_text())
                .as_deref(),
            Some("Hi!")
        );
        Ok(())
    }

    #[test]
    fn parses_completed_batch_with_file_output() -> serde_json::Result<()> {
        let payload = json!({
            "name": "batches/file-out-1",
            "metadata": {
                "name": "batches/file-out-1",
                "model": "models/gemini-2.5-flash",
                "displayName": "file-batch",
                "state": "BATCH_STATE_SUCCEEDED",
                "output": {
                    "responsesFile": "files/output-jsonl-999"
                }
            },
            "done": true,
            "response": {
                "responsesFile": "files/output-jsonl-999"
            }
        });

        let op: BatchOperation = serde_json::from_value(payload)?;
        assert!(op.is_done());
        assert_eq!(op.state(), Some(BatchState::Succeeded));
        assert_eq!(
            op.output().and_then(|o| o.responses_file.as_deref()),
            Some("files/output-jsonl-999")
        );
        Ok(())
    }

    #[test]
    fn batch_state_round_trip_all_variants_and_aliases() -> serde_json::Result<()> {
        let pairs = [
            (BatchState::Unspecified, "BATCH_STATE_UNSPECIFIED"),
            (BatchState::Pending, "BATCH_STATE_PENDING"),
            (BatchState::Running, "BATCH_STATE_RUNNING"),
            (BatchState::Succeeded, "BATCH_STATE_SUCCEEDED"),
            (BatchState::Failed, "BATCH_STATE_FAILED"),
            (BatchState::Cancelled, "BATCH_STATE_CANCELLED"),
            (BatchState::Expired, "BATCH_STATE_EXPIRED"),
        ];

        for (variant, wire) in pairs {
            assert_eq!(serde_json::to_value(variant)?, json!(wire));
            assert_eq!(serde_json::from_value::<BatchState>(json!(wire))?, variant);
        }

        // Test aliases without "BATCH_STATE_" prefix
        assert_eq!(
            serde_json::from_value::<BatchState>(json!("PENDING"))?,
            BatchState::Pending
        );
        assert_eq!(
            serde_json::from_value::<BatchState>(json!("RUNNING"))?,
            BatchState::Running
        );
        assert_eq!(
            serde_json::from_value::<BatchState>(json!("SUCCEEDED"))?,
            BatchState::Succeeded
        );
        assert_eq!(
            serde_json::from_value::<BatchState>(json!("FAILED"))?,
            BatchState::Failed
        );
        assert_eq!(
            serde_json::from_value::<BatchState>(json!("CANCELLED"))?,
            BatchState::Cancelled
        );
        assert_eq!(
            serde_json::from_value::<BatchState>(json!("EXPIRED"))?,
            BatchState::Expired
        );
        Ok(())
    }

    #[test]
    fn batch_stats_supports_strings_integers_and_null() -> serde_json::Result<()> {
        let from_str: BatchStats = serde_json::from_value(json!({
            "requestCount": "1000",
            "successfulRequestCount": "995",
            "failedRequestCount": "5",
            "pendingRequestCount": "0"
        }))?;
        assert_eq!(from_str.request_count, Some(1000));
        assert_eq!(from_str.successful_request_count, Some(995));
        assert_eq!(from_str.failed_request_count, Some(5));
        assert_eq!(from_str.pending_request_count, Some(0));

        let from_int: BatchStats = serde_json::from_value(json!({
            "requestCount": 50,
            "successfulRequestCount": 50
        }))?;
        assert_eq!(from_int.request_count, Some(50));
        assert_eq!(from_int.successful_request_count, Some(50));
        assert_eq!(from_int.failed_request_count, None);
        assert_eq!(from_int.pending_request_count, None);
        Ok(())
    }

    #[test]
    fn inlined_request_and_response_with_metadata_and_error() -> serde_json::Result<()> {
        let mut meta = HashMap::new();
        meta.insert("correlationId".to_string(), json!("req-42"));

        let inlined_req = InlinedRequest::new(
            GenerateContentRequest::builder()
                .contents(vec![Content::system_prompt("Hello")])
                .build(),
        )
        .with_metadata(meta.clone());

        let val = serde_json::to_value(&inlined_req)?;
        assert_eq!(val["metadata"]["correlationId"], "req-42");

        let inlined_resp = InlinedResponse {
            response: None,
            metadata: Some(meta),
            error: Some(Status {
                code: Some(4),
                message: Some("Deadline exceeded".to_string()),
                details: None,
            }),
        };
        let resp_val = serde_json::to_value(&inlined_resp)?;
        assert_eq!(resp_val["error"]["code"], 4);
        assert_eq!(resp_val["error"]["message"], "Deadline exceeded");
        Ok(())
    }

    #[test]
    fn list_batches_response_parses_operations_and_alias() -> serde_json::Result<()> {
        let payload = json!({
            "operations": [
                {
                    "name": "batches/op-1",
                    "metadata": {
                        "name": "batches/op-1",
                        "state": "BATCH_STATE_RUNNING"
                    }
                }
            ],
            "nextPageToken": "token-xyz"
        });

        let resp: ListBatchesResponse = serde_json::from_value(payload)?;
        assert_eq!(resp.operations.len(), 1);
        assert_eq!(resp.next_page_token.as_deref(), Some("token-xyz"));
        let batches: Vec<_> = resp.batches().collect();
        assert_eq!(batches.len(), 1);
        assert_eq!(batches[0].state, Some(BatchState::Running));

        // Test alias "batches"
        let payload_batches = json!({
            "batches": [
                {
                    "name": "batches/op-2",
                    "metadata": {
                        "name": "batches/op-2",
                        "state": "BATCH_STATE_SUCCEEDED"
                    }
                }
            ]
        });
        let resp_alias: ListBatchesResponse = serde_json::from_value(payload_batches)?;
        assert_eq!(resp_alias.operations.len(), 1);
        Ok(())
    }

    #[test]
    fn rejects_malformed_batch_payloads() {
        // Unknown batch state variant must fail strictly
        assert!(
            serde_json::from_value::<GenerateContentBatch>(json!({
                "state": "BATCH_STATE_UNKNOWN_NEW"
            }))
            .is_err()
        );

        // Non-integer string in batchStats must fail strictly
        assert!(
            serde_json::from_value::<BatchStats>(json!({
                "requestCount": "invalid-count"
            }))
            .is_err()
        );
    }
}
