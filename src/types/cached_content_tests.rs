// Copyright 2026 Andre Cipriani Bandarra
// SPDX-License-Identifier: Apache-2.0

use serde_json::json;

use super::*;
use crate::types::{
    Content, FunctionCallingConfig, FunctionCallingMode, FunctionDeclaration, Role, ToolConfig,
    Tools,
};

type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>;

#[test]
fn serializes_create_request_with_all_fields() -> Result<()> {
    let function = FunctionDeclaration {
        name: String::from("lookup"),
        description: String::from("Look up a document"),
        behavior: None,
        parameters: None,
        parameters_json_schema: Some(json!({ "type": "object" })),
        response: None,
        response_json_schema: None,
    };

    let request = CachedContent::builder()
        .model("models/gemini-1.5-flash-001")
        .display_name("Analysis cache")
        .system_instruction(
            Content::builder()
                .add_text_part("You are an expert analyst.")
                .build(),
        )
        .contents(vec![
            Content::builder()
                .role(Role::User)
                .add_text_part("Document data to cache...")
                .build(),
        ])
        .tools(vec![Tools {
            function_declarations: Some(vec![function]),
            ..Default::default()
        }])
        .tool_config(ToolConfig {
            function_calling_config: Some(FunctionCallingConfig {
                mode: Some(FunctionCallingMode::Auto),
                allowed_function_names: None,
            }),
            retrieval_config: None,
            include_server_side_tool_invocations: None,
        })
        .ttl("300s")
        .build();

    let json_value = serde_json::to_value(&request)?;

    assert_eq!(
        json_value,
        json!({
            "model": "models/gemini-1.5-flash-001",
            "displayName": "Analysis cache",
            "systemInstruction": {
                "parts": [{ "text": "You are an expert analyst." }]
            },
            "contents": [{
                "role": "user",
                "parts": [{ "text": "Document data to cache..." }]
            }],
            "tools": [{
                "function_declarations": [{
                    "name": "lookup",
                    "description": "Look up a document",
                    "parametersJsonSchema": { "type": "object" }
                }]
            }],
            "toolConfig": {
                "functionCallingConfig": {
                    "mode": "AUTO"
                }
            },
            "ttl": "300s"
        })
    );

    // Verify unset output-only fields were omitted from the serialized JSON.
    let obj = json_value.as_object().ok_or("expected object")?;
    assert!(!obj.contains_key("name"));
    assert!(!obj.contains_key("createTime"));
    assert!(!obj.contains_key("updateTime"));
    assert!(!obj.contains_key("expireTime"));
    assert!(!obj.contains_key("usageMetadata"));

    Ok(())
}

#[test]
fn serializes_create_request_with_expire_time() -> Result<()> {
    let request = CachedContent::builder()
        .model("models/gemini-1.5-flash-001")
        .expire_time("2026-10-28T23:00:00Z")
        .build();

    let json_value = serde_json::to_value(&request)?;

    assert_eq!(
        json_value,
        json!({
            "model": "models/gemini-1.5-flash-001",
            "expireTime": "2026-10-28T23:00:00Z"
        })
    );

    let obj = json_value.as_object().ok_or("expected object")?;
    assert!(!obj.contains_key("ttl"));

    Ok(())
}

#[test]
fn omits_unset_optional_fields_on_serialize() -> Result<()> {
    let request = CachedContent::default();
    let json_value = serde_json::to_value(&request)?;

    assert_eq!(json_value, json!({}));
    Ok(())
}

#[test]
fn parses_get_response() -> Result<()> {
    let response_json = json!({
        "name": "cachedContents/ab12cd34ef56",
        "model": "models/gemini-1.5-flash-001",
        "displayName": "Analysis cache",
        "usageMetadata": {
            "totalTokenCount": 35000
        },
        "createTime": "2026-10-28T20:00:00Z",
        "updateTime": "2026-10-28T20:05:00Z",
        "expireTime": "2026-10-28T21:00:00Z"
    });

    let cached_content: CachedContent = serde_json::from_value(response_json)?;

    assert_eq!(
        cached_content.name.as_deref(),
        Some("cachedContents/ab12cd34ef56")
    );
    assert_eq!(
        cached_content.model.as_deref(),
        Some("models/gemini-1.5-flash-001")
    );
    assert_eq!(
        cached_content.display_name.as_deref(),
        Some("Analysis cache")
    );
    assert_eq!(
        cached_content.create_time.as_deref(),
        Some("2026-10-28T20:00:00Z")
    );
    assert_eq!(
        cached_content.update_time.as_deref(),
        Some("2026-10-28T20:05:00Z")
    );
    assert_eq!(
        cached_content.expire_time.as_deref(),
        Some("2026-10-28T21:00:00Z")
    );

    let usage = cached_content
        .usage_metadata
        .ok_or("missing usageMetadata")?;
    assert_eq!(usage.total_token_count, Some(35000));

    // Input-only fields should be None on get response.
    assert!(cached_content.contents.is_none());
    assert!(cached_content.tools.is_none());
    assert!(cached_content.system_instruction.is_none());
    assert!(cached_content.ttl.is_none());

    Ok(())
}

#[test]
fn parses_list_response() -> Result<()> {
    let response_json = json!({
        "cachedContents": [{
            "name": "cachedContents/ab12cd34ef56",
            "model": "models/gemini-1.5-flash-001",
            "displayName": "Analysis cache",
            "usageMetadata": {
                "totalTokenCount": 35000
            },
            "createTime": "2026-10-28T20:00:00Z",
            "updateTime": "2026-10-28T20:05:00Z",
            "expireTime": "2026-10-28T21:00:00Z"
        }],
        "nextPageToken": "page-token-123"
    });

    let list_response: ListCachedContentsResponse = serde_json::from_value(response_json)?;

    assert_eq!(list_response.cached_contents.len(), 1);
    assert_eq!(
        list_response.cached_contents[0].name.as_deref(),
        Some("cachedContents/ab12cd34ef56")
    );
    assert_eq!(
        list_response.next_page_token.as_deref(),
        Some("page-token-123")
    );

    Ok(())
}

#[test]
fn parses_empty_list_response() -> Result<()> {
    let response_json = json!({});

    let list_response: ListCachedContentsResponse = serde_json::from_value(response_json)?;

    assert!(list_response.cached_contents.is_empty());
    assert!(list_response.next_page_token.is_none());

    Ok(())
}

#[test]
fn round_trips_cached_content() -> Result<()> {
    let original = CachedContent::builder()
        .name("cachedContents/test1234")
        .model("models/gemini-2.0-flash")
        .display_name("Round trip test")
        .ttl("600s")
        .usage_metadata(CachedContentUsageMetadata {
            total_token_count: Some(5000),
        })
        .build();

    let json_string = serde_json::to_string(&original)?;
    let parsed: CachedContent = serde_json::from_str(&json_string)?;

    assert_eq!(parsed.name, original.name);
    assert_eq!(parsed.model, original.model);
    assert_eq!(parsed.display_name, original.display_name);
    assert_eq!(parsed.ttl, original.ttl);
    assert_eq!(parsed.usage_metadata, original.usage_metadata);
    Ok(())
}
