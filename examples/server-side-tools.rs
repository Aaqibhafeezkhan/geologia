// Copyright 2026 Andre Cipriani Bandarra
// SPDX-License-Identifier: Apache-2.0

use std::{env, error::Error};

use geologia::prelude::{
    Content, GeminiClient, GenerateContentRequest, GoogleSearch, PartData, Role, ToolConfig, Tools,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt().init();
    let _ = dotenvy::dotenv();

    let api_key = env::var("GEMINI_API_KEY")?;
    let gemini_client = GeminiClient::new(api_key);

    let prompt = vec![
        Content::builder()
            .role(Role::User)
            .add_text_part(
                "What is the current stable release version of the Rust programming language?",
            )
            .build(),
    ];

    let tools = Tools {
        google_search: Some(GoogleSearch::default()),
        ..Default::default()
    };

    let tool_config = ToolConfig {
        include_server_side_tool_invocations: Some(true),
        ..Default::default()
    };

    let request = GenerateContentRequest::builder()
        .contents(prompt)
        .tools(vec![tools])
        .tool_config(tool_config)
        .build();

    let response = gemini_client
        .generate_content(&request, "gemini-3.1-flash-lite")
        .await?;

    let candidate = &response.candidates[0];
    if let Some(content) = &candidate.content
        && let Some(parts) = &content.parts
    {
        println!("Number of parts in candidate: {}", parts.len());
        for (i, part) in parts.iter().enumerate() {
            match &part.data {
                PartData::Text(text) => println!("  Part {i}: Text ({})", text.trim()),
                PartData::ToolCall(call) => {
                    println!(
                        "  Part {i}: ToolCall (id: {:?}, name: {:?}, type: {:?})",
                        call.id, call.tool_name, call.tool_type
                    );
                }
                PartData::ToolResponse(resp) => {
                    println!(
                        "  Part {i}: ToolResponse (id: {:?}, type: {:?})",
                        resp.id, resp.tool_type
                    );
                }
                other => println!("  Part {i}: Other ({other:?})"),
            }
        }
    }

    Ok(())
}
