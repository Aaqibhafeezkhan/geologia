// Copyright 2026 Andre Cipriani Bandarra
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};

use crate::types::Status;
use crate::types::common::opt_int64_str;

/// A container of documents indexed for semantic retrieval and grounding via the
/// Gemini File Search tool.
///
/// See <https://ai.google.dev/api/file-search/file-search-stores>.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FileSearchStore {
    /// Immutable identifier for the `FileSearchStore` resource, in the format
    /// `fileSearchStores/{id}` (e.g. `fileSearchStores/my-store-123`).
    ///
    /// If omitted on creation, a unique name is derived from `display_name` along with
    /// a random suffix. Output only on server responses.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Optional human-readable display name for the store (up to 512 characters).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    /// Optional embedding model resource name (e.g. `models/gemini-embedding-2` or
    /// `models/text-embedding-004`). If unspecified, the server default is used.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embedding_model: Option<String>,
    /// Total number of active documents indexed and ready for retrieval. Output only.
    #[serde(
        default,
        with = "opt_int64_str",
        skip_serializing_if = "Option::is_none"
    )]
    pub active_documents_count: Option<i64>,
    /// Number of documents currently being processed. Output only.
    #[serde(
        default,
        with = "opt_int64_str",
        skip_serializing_if = "Option::is_none"
    )]
    pub pending_documents_count: Option<i64>,
    /// Number of documents that failed processing. Output only.
    #[serde(
        default,
        with = "opt_int64_str",
        skip_serializing_if = "Option::is_none"
    )]
    pub failed_documents_count: Option<i64>,
    /// Total size in raw bytes of all documents ingested into the store. Output only.
    #[serde(
        default,
        with = "opt_int64_str",
        skip_serializing_if = "Option::is_none"
    )]
    pub size_bytes: Option<i64>,
    /// RFC 3339 timestamp of when the store was created. Output only.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub create_time: Option<String>,
    /// RFC 3339 timestamp of when the store was last updated. Output only.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub update_time: Option<String>,
}

impl FileSearchStore {
    /// Returns a new [`FileSearchStoreBuilder`].
    pub fn builder() -> FileSearchStoreBuilder {
        FileSearchStoreBuilder::default()
    }
}

/// Builder for [`FileSearchStore`].
#[derive(Clone, Debug, Default)]
pub struct FileSearchStoreBuilder {
    store: FileSearchStore,
}

impl FileSearchStoreBuilder {
    /// Sets the resource name (`fileSearchStores/{id}` or `{id}`).
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.store.name = Some(name.into());
        self
    }

    /// Sets the human-readable display name.
    pub fn display_name(mut self, display_name: impl Into<String>) -> Self {
        self.store.display_name = Some(display_name.into());
        self
    }

    /// Sets the embedding model resource name (e.g. `models/gemini-embedding-2`).
    pub fn embedding_model(mut self, embedding_model: impl Into<String>) -> Self {
        self.store.embedding_model = Some(embedding_model.into());
        self
    }

    /// Sets the active documents count.
    pub fn active_documents_count(mut self, count: i64) -> Self {
        self.store.active_documents_count = Some(count);
        self
    }

    /// Sets the pending documents count.
    pub fn pending_documents_count(mut self, count: i64) -> Self {
        self.store.pending_documents_count = Some(count);
        self
    }

    /// Sets the failed documents count.
    pub fn failed_documents_count(mut self, count: i64) -> Self {
        self.store.failed_documents_count = Some(count);
        self
    }

    /// Sets the total size in bytes.
    pub fn size_bytes(mut self, size_bytes: i64) -> Self {
        self.store.size_bytes = Some(size_bytes);
        self
    }

    /// Sets the creation timestamp (RFC 3339 format).
    pub fn create_time(mut self, create_time: impl Into<String>) -> Self {
        self.store.create_time = Some(create_time.into());
        self
    }

    /// Sets the last update timestamp (RFC 3339 format).
    pub fn update_time(mut self, update_time: impl Into<String>) -> Self {
        self.store.update_time = Some(update_time.into());
        self
    }

    /// Consumes the builder and returns the constructed [`FileSearchStore`].
    pub fn build(self) -> FileSearchStore {
        self.store
    }
}

/// Paginated list of [`FileSearchStore`] resources returned by
/// [`GeminiClient::list_file_search_stores`](crate::prelude::GeminiClient::list_file_search_stores).
///
/// See <https://ai.google.dev/api/file-search/file-search-stores#method:-filesearchstores.list>.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ListFileSearchStoresResponse {
    /// The retrieved file search stores.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub file_search_stores: Vec<FileSearchStore>,
    /// A token to retrieve the next page of results, or `None` if there are no more pages.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_page_token: Option<String>,
}

/// Processing state of a [`Document`] in a [`FileSearchStore`].
///
/// See <https://ai.google.dev/api/file-search/file-search-stores#State>.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DocumentState {
    /// Default unspecified state.
    #[serde(rename = "STATE_UNSPECIFIED")]
    Unspecified,
    /// Some chunks of the document are being processed (embedding and vector storage).
    #[serde(rename = "STATE_PENDING", alias = "PENDING")]
    Pending,
    /// All chunks of the document are processed and available for querying.
    #[serde(rename = "STATE_ACTIVE", alias = "ACTIVE")]
    Active,
    /// Some chunks of the document failed processing.
    #[serde(rename = "STATE_FAILED", alias = "FAILED")]
    Failed,
}

/// User-provided custom metadata stored as key-value pairs on a [`Document`].
///
/// See <https://ai.google.dev/api/file-search/file-search-stores#CustomMetadata>.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CustomMetadata {
    /// The key of the metadata to store.
    pub key: String,
    /// The string value of the metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub string_value: Option<String>,
    /// The numeric (float) value of the metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub numeric_value: Option<f32>,
    /// The string list value of the metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub string_list_value: Option<StringList>,
}

impl CustomMetadata {
    /// Creates a [`CustomMetadata`] entry with a string value.
    pub fn string(key: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            string_value: Some(value.into()),
            numeric_value: None,
            string_list_value: None,
        }
    }

    /// Creates a [`CustomMetadata`] entry with a numeric (f32) value.
    pub fn numeric(key: impl Into<String>, value: f32) -> Self {
        Self {
            key: key.into(),
            string_value: None,
            numeric_value: Some(value),
            string_list_value: None,
        }
    }

    /// Creates a [`CustomMetadata`] entry with a list of strings.
    pub fn string_list(
        key: impl Into<String>,
        values: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        Self {
            key: key.into(),
            string_value: None,
            numeric_value: None,
            string_list_value: Some(StringList {
                values: values.into_iter().map(Into::into).collect(),
            }),
        }
    }
}

/// A list of string values for [`CustomMetadata`].
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct StringList {
    /// The string values.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub values: Vec<String>,
}

impl<I, S> From<I> for StringList
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    fn from(values: I) -> Self {
        Self {
            values: values.into_iter().map(Into::into).collect(),
        }
    }
}

/// Configuration for how documents are split into chunks for semantic retrieval.
///
/// See <https://ai.google.dev/api/file-search/file-search-stores#ChunkingConfig>.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ChunkingConfig {
    /// White space chunking configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub white_space_config: Option<WhiteSpaceConfig>,
}

impl ChunkingConfig {
    /// Creates a [`ChunkingConfig`] with whitespace chunking parameters.
    pub fn white_space(max_tokens_per_chunk: u32, max_overlap_tokens: u32) -> Self {
        Self {
            white_space_config: Some(WhiteSpaceConfig {
                max_tokens_per_chunk: Some(max_tokens_per_chunk),
                max_overlap_tokens: Some(max_overlap_tokens),
            }),
        }
    }
}

/// Parameters for splitting text on whitespace boundaries.
///
/// See <https://ai.google.dev/api/file-search/file-search-stores#WhiteSpaceConfig>.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WhiteSpaceConfig {
    /// Maximum tokens per chunk.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens_per_chunk: Option<u32>,
    /// Number of tokens that overlap between adjacent chunks.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_overlap_tokens: Option<u32>,
}

/// A document ingested into a [`FileSearchStore`].
///
/// A document is split into chunks, embedded, and stored for retrieval. Newly uploaded or
/// imported documents are processed asynchronously: `state` begins as [`DocumentState::Pending`]
/// and transitions to [`DocumentState::Active`] (ready for querying) or [`DocumentState::Failed`].
///
/// See <https://ai.google.dev/api/file-search/file-search-stores#Document>.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Document {
    /// Identifier for the `Document` resource, formatted as
    /// `fileSearchStores/{store_id}/documents/{doc_id}`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Optional human-readable display name for the document (up to 512 characters).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    /// MIME type of the document. Output only.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    /// Total raw bytes ingested into the document. Output only.
    #[serde(
        default,
        with = "opt_int64_str",
        skip_serializing_if = "Option::is_none"
    )]
    pub size_bytes: Option<i64>,
    /// User-provided custom metadata (up to 20 key-value pairs).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_metadata: Option<Vec<CustomMetadata>>,
    /// Current processing state of the document. Output only.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<DocumentState>,
    /// RFC 3339 timestamp of when the document was created. Output only.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub create_time: Option<String>,
    /// RFC 3339 timestamp of when the document was last updated. Output only.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub update_time: Option<String>,
}

impl Document {
    /// Returns a new [`DocumentBuilder`].
    pub fn builder() -> DocumentBuilder {
        DocumentBuilder::default()
    }
}

/// Builder for [`Document`].
#[derive(Clone, Debug, Default)]
pub struct DocumentBuilder {
    document: Document,
}

impl DocumentBuilder {
    /// Sets the document resource name (`fileSearchStores/{store_id}/documents/{doc_id}`).
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.document.name = Some(name.into());
        self
    }

    /// Sets the human-readable display name.
    pub fn display_name(mut self, display_name: impl Into<String>) -> Self {
        self.document.display_name = Some(display_name.into());
        self
    }

    /// Sets the MIME type of the document.
    pub fn mime_type(mut self, mime_type: impl Into<String>) -> Self {
        self.document.mime_type = Some(mime_type.into());
        self
    }

    /// Sets the size of the document in bytes.
    pub fn size_bytes(mut self, size_bytes: i64) -> Self {
        self.document.size_bytes = Some(size_bytes);
        self
    }

    /// Sets the custom metadata key-value pairs.
    pub fn custom_metadata(mut self, custom_metadata: Vec<CustomMetadata>) -> Self {
        self.document.custom_metadata = Some(custom_metadata);
        self
    }

    /// Sets the document processing state.
    pub fn state(mut self, state: DocumentState) -> Self {
        self.document.state = Some(state);
        self
    }

    /// Sets the creation timestamp (RFC 3339 format).
    pub fn create_time(mut self, create_time: impl Into<String>) -> Self {
        self.document.create_time = Some(create_time.into());
        self
    }

    /// Sets the update timestamp (RFC 3339 format).
    pub fn update_time(mut self, update_time: impl Into<String>) -> Self {
        self.document.update_time = Some(update_time.into());
        self
    }

    /// Consumes the builder and returns the constructed [`Document`].
    pub fn build(self) -> Document {
        self.document
    }
}

/// Request payload for importing a file from the Files API into a [`FileSearchStore`] via
/// `POST v1beta/{fileSearchStoreName}:importFile`.
///
/// See <https://ai.google.dev/api/file-search/file-search-stores#method:-filesearchstores.importfile>.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ImportFileRequest {
    /// The resource name of the `File` to import (e.g. `files/abc-123`).
    pub file_name: String,
    /// Custom metadata to associate with the imported file.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_metadata: Option<Vec<CustomMetadata>>,
    /// Optional chunking configuration. If omitted, default server parameters are used.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chunking_config: Option<ChunkingConfig>,
}

impl ImportFileRequest {
    /// Creates a new [`ImportFileRequest`] for the given file name or ID.
    pub fn new(file_name: impl Into<String>) -> Self {
        let name = file_name.into();
        let normalized = if name.starts_with("files/") {
            name
        } else {
            format!("files/{name}")
        };
        Self {
            file_name: normalized,
            custom_metadata: None,
            chunking_config: None,
        }
    }

    /// Sets custom metadata to associate with the imported file.
    pub fn with_custom_metadata(mut self, metadata: Vec<CustomMetadata>) -> Self {
        self.custom_metadata = Some(metadata);
        self
    }

    /// Sets the chunking configuration.
    pub fn with_chunking_config(mut self, config: ChunkingConfig) -> Self {
        self.chunking_config = Some(config);
        self
    }
}

/// Metadata accompanying raw file bytes uploaded directly to a [`FileSearchStore`] via
/// `POST /upload/v1beta/{store}:uploadToFileSearchStore`.
///
/// See <https://ai.google.dev/api/file-search/file-search-stores#method:-media.uploadtofilesearchstore>.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UploadToFileSearchStoreRequest {
    /// Optional display name of the created document.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    /// Optional MIME type of the uploaded data. If omitted, it is inferred by the server.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    /// Optional custom metadata to associate with the uploaded document.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_metadata: Option<Vec<CustomMetadata>>,
    /// Optional chunking configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chunking_config: Option<ChunkingConfig>,
}

impl UploadToFileSearchStoreRequest {
    /// Returns a new [`UploadToFileSearchStoreRequestBuilder`].
    pub fn builder() -> UploadToFileSearchStoreRequestBuilder {
        UploadToFileSearchStoreRequestBuilder::default()
    }
}

/// Builder for [`UploadToFileSearchStoreRequest`].
#[derive(Clone, Debug, Default)]
pub struct UploadToFileSearchStoreRequestBuilder {
    request: UploadToFileSearchStoreRequest,
}

impl UploadToFileSearchStoreRequestBuilder {
    /// Sets the display name of the document.
    pub fn display_name(mut self, display_name: impl Into<String>) -> Self {
        self.request.display_name = Some(display_name.into());
        self
    }

    /// Sets the MIME type of the uploaded content.
    pub fn mime_type(mut self, mime_type: impl Into<String>) -> Self {
        self.request.mime_type = Some(mime_type.into());
        self
    }

    /// Sets custom metadata to associate with the document.
    pub fn custom_metadata(mut self, custom_metadata: Vec<CustomMetadata>) -> Self {
        self.request.custom_metadata = Some(custom_metadata);
        self
    }

    /// Sets the chunking configuration.
    pub fn chunking_config(mut self, chunking_config: ChunkingConfig) -> Self {
        self.request.chunking_config = Some(chunking_config);
        self
    }

    /// Consumes the builder and returns the constructed [`UploadToFileSearchStoreRequest`].
    pub fn build(self) -> UploadToFileSearchStoreRequest {
        self.request
    }
}

/// Response payload of a completed [`FileSearchOperation`].
///
/// Both direct uploads (`uploadToFileSearchStore`) and file imports (`importFile`) return
/// this structure upon successful completion.
///
/// See <https://ai.google.dev/api/file-search/file-search-stores#Operation>.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FileSearchOperationResponse {
    /// The parent `FileSearchStore` resource name (e.g. `fileSearchStores/my-store-123`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,
    /// The resource name of the resulting document (e.g. `fileSearchStores/my-store-123/documents/abc-123`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document_name: Option<String>,
}

/// A long-running operation returned by asynchronous File Search Store actions such as
/// [`upload_to_file_search_store`](crate::prelude::GeminiClient::upload_to_file_search_store) and
/// [`import_file_to_file_search_store`](crate::prelude::GeminiClient::import_file_to_file_search_store).
///
/// Callers can poll [`GeminiClient::get_file_search_operation`](crate::prelude::GeminiClient::get_file_search_operation)
/// until [`FileSearchOperation::is_done`] is `true`. Upon successful completion, the resulting
/// document resource name is available via [`FileSearchOperation::document_name`], and the document
/// can be retrieved using [`GeminiClient::get_document`](crate::prelude::GeminiClient::get_document).
///
/// See <https://ai.google.dev/api/file-search/file-search-stores#Operation>.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FileSearchOperation {
    /// Server-assigned operation name (e.g. `fileSearchStores/{store}/upload/operations/{id}` or
    /// `fileSearchStores/{store}/operations/{id}`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// `true` if the operation has completed, or `false` if it is still in progress.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub done: Option<bool>,
    /// Service-specific metadata detailing operation progress.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
    /// Error status if the operation failed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<Status>,
    /// The response payload upon successful completion.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response: Option<FileSearchOperationResponse>,
}

impl FileSearchOperation {
    /// Returns `true` if the operation has completed (successfully or with error).
    pub fn is_done(&self) -> bool {
        self.done.unwrap_or(false)
    }

    /// Returns a reference to the error status if the operation failed.
    pub fn error(&self) -> Option<&Status> {
        self.error.as_ref()
    }

    /// Returns the document resource name associated with this operation, if present in `response`
    /// or derivable from an upload operation name.
    ///
    /// For direct uploads, the operation name is formatted as
    /// `fileSearchStores/{store_id}/upload/operations/{doc_id}`, corresponding to
    /// the document resource name `fileSearchStores/{store_id}/documents/{doc_id}`.
    pub fn document_name(&self) -> Option<String> {
        if let Some(resp) = &self.response
            && let Some(name) = &resp.document_name
        {
            return Some(name.clone());
        }
        if let Some(op_name) = &self.name
            && let Some((store, doc_id)) = op_name.split_once("/upload/operations/")
        {
            return Some(format!("{store}/documents/{doc_id}"));
        }
        None
    }
}

/// Tool configuration for grounding model generation in one or more [`FileSearchStore`]s.
///
/// See <https://ai.google.dev/api/file-search/file-search-stores#FileSearch>.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FileSearch {
    /// Resource names of the file search stores to query (e.g. `fileSearchStores/my-store-123`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_search_store_names: Option<Vec<String>>,
    /// Optional number of semantic retrieval chunks to retrieve.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_k: Option<i32>,
    /// Optional metadata filter to apply to retrieved documents and chunks.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata_filter: Option<String>,
}

impl FileSearch {
    /// Creates a [`FileSearch`] configuration querying a single file search store.
    pub fn from_store(store: impl Into<String>) -> Self {
        let store_name = store.into();
        let normalized = if store_name.starts_with("fileSearchStores/") {
            store_name
        } else {
            format!("fileSearchStores/{store_name}")
        };
        Self {
            file_search_store_names: Some(vec![normalized]),
            top_k: None,
            metadata_filter: None,
        }
    }

    /// Creates a [`FileSearch`] configuration querying multiple file search stores.
    pub fn from_stores(stores: impl IntoIterator<Item = impl Into<String>>) -> Self {
        let names = stores
            .into_iter()
            .map(|s| {
                let name = s.into();
                if name.starts_with("fileSearchStores/") {
                    name
                } else {
                    format!("fileSearchStores/{name}")
                }
            })
            .collect();
        Self {
            file_search_store_names: Some(names),
            top_k: None,
            metadata_filter: None,
        }
    }

    /// Sets the maximum number of chunks to retrieve.
    pub fn with_top_k(mut self, top_k: i32) -> Self {
        self.top_k = Some(top_k);
        self
    }

    /// Sets the metadata filter.
    pub fn with_metadata_filter(mut self, filter: impl Into<String>) -> Self {
        self.metadata_filter = Some(filter.into());
        self
    }
}

#[cfg(test)]
#[path = "file_search_stores_tests.rs"]
mod tests;
