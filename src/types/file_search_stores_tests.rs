// Copyright 2026 Andre Cipriani Bandarra
// SPDX-License-Identifier: Apache-2.0

use serde_json::json;

use super::*;

#[test]
fn parses_minimal_file_search_store_and_omits_unset_fields_on_serialize() -> serde_json::Result<()>
{
    let payload = json!({
        "name": "fileSearchStores/store-123"
    });

    let store: FileSearchStore = serde_json::from_value(payload.clone())?;
    assert_eq!(store.name.as_deref(), Some("fileSearchStores/store-123"));
    assert_eq!(store.display_name, None);
    assert_eq!(store.embedding_model, None);
    assert_eq!(store.active_documents_count, None);
    assert_eq!(store.pending_documents_count, None);
    assert_eq!(store.failed_documents_count, None);
    assert_eq!(store.size_bytes, None);
    assert_eq!(store.create_time, None);
    assert_eq!(store.update_time, None);

    assert_eq!(serde_json::to_value(&store)?, payload);
    Ok(())
}

#[test]
fn parses_full_file_search_store_and_round_trips() -> serde_json::Result<()> {
    let payload = json!({
        "name": "fileSearchStores/knowledge-base-abc",
        "displayName": "Knowledge Base",
        "embeddingModel": "models/gemini-embedding-2",
        "activeDocumentsCount": "42",
        "pendingDocumentsCount": "3",
        "failedDocumentsCount": "1",
        "sizeBytes": "1048576",
        "createTime": "2026-03-30T10:00:00Z",
        "updateTime": "2026-03-30T10:15:00Z"
    });

    let store: FileSearchStore = serde_json::from_value(payload.clone())?;
    assert_eq!(
        store.name.as_deref(),
        Some("fileSearchStores/knowledge-base-abc")
    );
    assert_eq!(store.display_name.as_deref(), Some("Knowledge Base"));
    assert_eq!(
        store.embedding_model.as_deref(),
        Some("models/gemini-embedding-2")
    );
    assert_eq!(store.active_documents_count, Some(42));
    assert_eq!(store.pending_documents_count, Some(3));
    assert_eq!(store.failed_documents_count, Some(1));
    assert_eq!(store.size_bytes, Some(1_048_576));
    assert_eq!(store.create_time.as_deref(), Some("2026-03-30T10:00:00Z"));
    assert_eq!(store.update_time.as_deref(), Some("2026-03-30T10:15:00Z"));

    assert_eq!(serde_json::to_value(&store)?, payload);
    Ok(())
}

#[test]
fn builder_constructs_file_search_store() {
    let store = FileSearchStore::builder()
        .name("fileSearchStores/custom-store")
        .display_name("Custom Store")
        .embedding_model("models/gemini-embedding-2")
        .active_documents_count(10)
        .pending_documents_count(2)
        .failed_documents_count(0)
        .size_bytes(2048)
        .create_time("2026-03-30T12:00:00Z")
        .update_time("2026-03-30T12:05:00Z")
        .build();

    assert_eq!(store.name.as_deref(), Some("fileSearchStores/custom-store"));
    assert_eq!(store.display_name.as_deref(), Some("Custom Store"));
    assert_eq!(
        store.embedding_model.as_deref(),
        Some("models/gemini-embedding-2")
    );
    assert_eq!(store.active_documents_count, Some(10));
    assert_eq!(store.pending_documents_count, Some(2));
    assert_eq!(store.failed_documents_count, Some(0));
    assert_eq!(store.size_bytes, Some(2048));
    assert_eq!(store.create_time.as_deref(), Some("2026-03-30T12:00:00Z"));
    assert_eq!(store.update_time.as_deref(), Some("2026-03-30T12:05:00Z"));
}

#[test]
fn count_and_size_fields_support_numbers_strings_and_null() -> serde_json::Result<()> {
    // Numbers as JSON integers
    let int_payload = json!({
        "name": "fileSearchStores/s1",
        "activeDocumentsCount": 5,
        "pendingDocumentsCount": 1,
        "failedDocumentsCount": 0,
        "sizeBytes": 1024
    });
    let parsed: FileSearchStore = serde_json::from_value(int_payload)?;
    assert_eq!(parsed.active_documents_count, Some(5));
    assert_eq!(parsed.pending_documents_count, Some(1));
    assert_eq!(parsed.failed_documents_count, Some(0));
    assert_eq!(parsed.size_bytes, Some(1024));

    // Null values
    let null_payload = json!({
        "name": "fileSearchStores/s2",
        "activeDocumentsCount": null,
        "pendingDocumentsCount": null,
        "failedDocumentsCount": null,
        "sizeBytes": null
    });
    let parsed_null: FileSearchStore = serde_json::from_value(null_payload)?;
    assert_eq!(parsed_null.active_documents_count, None);
    assert_eq!(parsed_null.pending_documents_count, None);
    assert_eq!(parsed_null.failed_documents_count, None);
    assert_eq!(parsed_null.size_bytes, None);
    Ok(())
}

#[test]
fn parses_empty_and_populated_list_file_search_stores_response() -> serde_json::Result<()> {
    // Empty response
    let empty_payload = json!({});
    let parsed_empty: ListFileSearchStoresResponse = serde_json::from_value(empty_payload)?;
    assert!(parsed_empty.file_search_stores.is_empty());
    assert_eq!(parsed_empty.next_page_token, None);
    assert_eq!(serde_json::to_value(&parsed_empty)?, json!({}));

    // Populated response with next page token
    let populated_payload = json!({
        "fileSearchStores": [
            {
                "name": "fileSearchStores/store-1",
                "displayName": "Store One"
            },
            {
                "name": "fileSearchStores/store-2",
                "displayName": "Store Two"
            }
        ],
        "nextPageToken": "page-token-xyz"
    });

    let parsed: ListFileSearchStoresResponse = serde_json::from_value(populated_payload.clone())?;
    assert_eq!(parsed.file_search_stores.len(), 2);
    assert_eq!(
        parsed.file_search_stores[0].name.as_deref(),
        Some("fileSearchStores/store-1")
    );
    assert_eq!(
        parsed.file_search_stores[1].name.as_deref(),
        Some("fileSearchStores/store-2")
    );
    assert_eq!(parsed.next_page_token.as_deref(), Some("page-token-xyz"));
    assert_eq!(serde_json::to_value(&parsed)?, populated_payload);
    Ok(())
}

#[test]
fn document_state_round_trip_all_variants_and_aliases() -> serde_json::Result<()> {
    let cases = [
        (DocumentState::Unspecified, "STATE_UNSPECIFIED"),
        (DocumentState::Pending, "STATE_PENDING"),
        (DocumentState::Active, "STATE_ACTIVE"),
        (DocumentState::Failed, "STATE_FAILED"),
    ];

    for (state, expected_str) in cases {
        assert_eq!(serde_json::to_value(state)?, json!(expected_str));
        assert_eq!(
            serde_json::from_value::<DocumentState>(json!(expected_str))?,
            state
        );
    }

    // Aliases
    assert_eq!(
        serde_json::from_value::<DocumentState>(json!("PENDING"))?,
        DocumentState::Pending
    );
    assert_eq!(
        serde_json::from_value::<DocumentState>(json!("ACTIVE"))?,
        DocumentState::Active
    );
    assert_eq!(
        serde_json::from_value::<DocumentState>(json!("FAILED"))?,
        DocumentState::Failed
    );
    Ok(())
}

#[test]
fn custom_metadata_variants_and_constructors_round_trip() -> serde_json::Result<()> {
    let str_meta = CustomMetadata::string("author", "Alice");
    assert_eq!(
        serde_json::to_value(&str_meta)?,
        json!({
            "key": "author",
            "stringValue": "Alice"
        })
    );

    let num_meta = CustomMetadata::numeric("score", 9.5);
    assert_eq!(
        serde_json::to_value(&num_meta)?,
        json!({
            "key": "score",
            "numericValue": 9.5
        })
    );

    let list_meta = CustomMetadata::string_list("tags", ["tag1", "tag2"]);
    assert_eq!(
        serde_json::to_value(&list_meta)?,
        json!({
            "key": "tags",
            "stringListValue": {
                "values": ["tag1", "tag2"]
            }
        })
    );

    let deserialized: CustomMetadata = serde_json::from_value(json!({
        "key": "tags",
        "stringListValue": {
            "values": ["tag1", "tag2"]
        }
    }))?;
    assert_eq!(deserialized, list_meta);
    Ok(())
}

#[test]
fn chunking_config_round_trip() -> serde_json::Result<()> {
    let config = ChunkingConfig::white_space(200, 20);
    let value = serde_json::to_value(&config)?;
    assert_eq!(
        value,
        json!({
            "whiteSpaceConfig": {
                "maxTokensPerChunk": 200,
                "maxOverlapTokens": 20
            }
        })
    );
    assert_eq!(serde_json::from_value::<ChunkingConfig>(value)?, config);
    Ok(())
}

#[test]
fn parses_document_minimal_and_full() -> serde_json::Result<()> {
    // Minimal
    let min_payload = json!({
        "name": "fileSearchStores/s1/documents/d1"
    });
    let min_doc: Document = serde_json::from_value(min_payload.clone())?;
    assert_eq!(
        min_doc.name.as_deref(),
        Some("fileSearchStores/s1/documents/d1")
    );
    assert_eq!(min_doc.state, None);
    assert_eq!(serde_json::to_value(&min_doc)?, min_payload);

    // Full
    let full_payload = json!({
        "name": "fileSearchStores/s1/documents/d1",
        "displayName": "Field Notes",
        "mimeType": "text/plain",
        "sizeBytes": "2048",
        "customMetadata": [
            { "key": "author", "stringValue": "Alice" }
        ],
        "state": "STATE_ACTIVE",
        "createTime": "2026-03-30T10:00:00Z",
        "updateTime": "2026-03-30T10:05:00Z"
    });
    let full_doc: Document = serde_json::from_value(full_payload.clone())?;
    assert_eq!(full_doc.display_name.as_deref(), Some("Field Notes"));
    assert_eq!(full_doc.state, Some(DocumentState::Active));
    assert_eq!(full_doc.size_bytes, Some(2048));
    assert_eq!(serde_json::to_value(&full_doc)?, full_payload);

    // Builder test
    let built = Document::builder()
        .name("fileSearchStores/s1/documents/d1")
        .display_name("Field Notes")
        .mime_type("text/plain")
        .size_bytes(2048)
        .custom_metadata(vec![CustomMetadata::string("author", "Alice")])
        .state(DocumentState::Active)
        .create_time("2026-03-30T10:00:00Z")
        .update_time("2026-03-30T10:05:00Z")
        .build();
    assert_eq!(built, full_doc);
    Ok(())
}

#[test]
fn import_file_request_constructs_and_serializes() -> serde_json::Result<()> {
    let req = ImportFileRequest::new("abc-123")
        .with_custom_metadata(vec![CustomMetadata::string("env", "prod")])
        .with_chunking_config(ChunkingConfig::white_space(100, 10));

    assert_eq!(req.file_name, "files/abc-123");
    let val = serde_json::to_value(&req)?;
    assert_eq!(
        val,
        json!({
            "fileName": "files/abc-123",
            "customMetadata": [{ "key": "env", "stringValue": "prod" }],
            "chunkingConfig": {
                "whiteSpaceConfig": {
                    "maxTokensPerChunk": 100,
                    "maxOverlapTokens": 10
                }
            }
        })
    );
    assert_eq!(serde_json::from_value::<ImportFileRequest>(val)?, req);
    Ok(())
}

#[test]
fn upload_to_file_search_store_request_constructs_and_serializes() -> serde_json::Result<()> {
    let req = UploadToFileSearchStoreRequest::builder()
        .display_name("spec.txt")
        .mime_type("text/plain")
        .custom_metadata(vec![CustomMetadata::string("dept", "eng")])
        .chunking_config(ChunkingConfig::white_space(300, 30))
        .build();

    let val = serde_json::to_value(&req)?;
    assert_eq!(
        val,
        json!({
            "displayName": "spec.txt",
            "mimeType": "text/plain",
            "customMetadata": [{ "key": "dept", "stringValue": "eng" }],
            "chunkingConfig": {
                "whiteSpaceConfig": {
                    "maxTokensPerChunk": 300,
                    "maxOverlapTokens": 30
                }
            }
        })
    );
    assert_eq!(
        serde_json::from_value::<UploadToFileSearchStoreRequest>(val)?,
        req
    );
    Ok(())
}

#[test]
fn file_search_operation_in_progress_and_completed_and_error() -> serde_json::Result<()> {
    // In progress
    let pending_payload = json!({
        "name": "fileSearchStores/store-1/operations/op-1",
        "done": false
    });
    let pending_op: FileSearchOperation = serde_json::from_value(pending_payload)?;
    assert!(!pending_op.is_done());
    assert!(pending_op.response.is_none());
    assert!(pending_op.document_name().is_none());
    assert!(pending_op.error().is_none());

    // Completed successfully
    let completed_payload = json!({
        "name": "fileSearchStores/store-1/operations/op-1",
        "done": true,
        "response": {
            "parent": "fileSearchStores/store-1",
            "documentName": "fileSearchStores/store-1/documents/doc-1"
        }
    });
    let completed_op: FileSearchOperation = serde_json::from_value(completed_payload)?;
    assert!(completed_op.is_done());
    let resp = completed_op
        .response
        .as_ref()
        .expect("response should be present");
    assert_eq!(resp.parent.as_deref(), Some("fileSearchStores/store-1"));
    assert_eq!(
        resp.document_name.as_deref(),
        Some("fileSearchStores/store-1/documents/doc-1")
    );
    assert_eq!(
        completed_op.document_name().as_deref(),
        Some("fileSearchStores/store-1/documents/doc-1")
    );

    // Derives document_name from upload operation name if response lacks documentName
    let upload_op_payload = json!({
        "name": "fileSearchStores/store-1/upload/operations/doc-abc-123",
        "done": true,
        "response": {
            "parent": "fileSearchStores/store-1"
        }
    });
    let upload_op: FileSearchOperation = serde_json::from_value(upload_op_payload)?;
    assert_eq!(
        upload_op.document_name().as_deref(),
        Some("fileSearchStores/store-1/documents/doc-abc-123")
    );

    // Completed with error
    let error_payload = json!({
        "name": "fileSearchStores/store-1/operations/op-1",
        "done": true,
        "error": {
            "code": 400,
            "message": "Unsupported file format"
        }
    });
    let error_op: FileSearchOperation = serde_json::from_value(error_payload)?;
    assert!(error_op.is_done());
    let err = error_op.error().expect("error status should be present");
    assert_eq!(err.code, Some(400));
    assert_eq!(err.message.as_deref(), Some("Unsupported file format"));
    Ok(())
}

#[test]
fn file_search_tool_config_constructs_and_serializes() -> serde_json::Result<()> {
    let single = FileSearch::from_store("my-store")
        .with_top_k(5)
        .with_metadata_filter("category = 'geology'");

    assert_eq!(
        serde_json::to_value(&single)?,
        json!({
            "fileSearchStoreNames": ["fileSearchStores/my-store"],
            "topK": 5,
            "metadataFilter": "category = 'geology'"
        })
    );

    let multi = FileSearch::from_stores(["store-a", "fileSearchStores/store-b"]);
    assert_eq!(
        serde_json::to_value(&multi)?,
        json!({
            "fileSearchStoreNames": [
                "fileSearchStores/store-a",
                "fileSearchStores/store-b"
            ]
        })
    );
    Ok(())
}
