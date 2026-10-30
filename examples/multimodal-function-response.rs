// Copyright 2026 Andre Cipriani Bandarra
// SPDX-License-Identifier: Apache-2.0

use std::{env, error::Error};

use geologia::prelude::{
    Content, FunctionDeclaration, FunctionResponse, FunctionResponseBlob, FunctionResponseFileData,
    FunctionResponsePart, GeminiClient, GenerateContentRequest, Part, PartData, Role, Tools,
};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt().init();
    let _ = dotenvy::dotenv();

    let api_key = env::var("GEMINI_API_KEY")?;
    let gemini_client = GeminiClient::new(api_key);

    let mut contents = vec![
        Content::builder()
            .role(Role::User)
            .add_text_part("Generate a chart of quarterly revenue and describe the visual trend.")
            .build(),
    ];

    let generate_chart_parameters = json!({
        "type": "object",
        "properties": {
            "title": {
                "type": "string",
                "description": "Chart title"
            }
        },
        "required": ["title"]
    });

    let chart_function = FunctionDeclaration {
        name: String::from("generate_chart"),
        description: String::from(
            "Generates a chart and returns both inline image bytes and a file reference",
        ),
        behavior: None,
        parameters: None,
        parameters_json_schema: Some(generate_chart_parameters),
        response: None,
        response_json_schema: None,
    };

    let tools = Tools {
        function_declarations: Some(vec![chart_function]),
        ..Default::default()
    };

    let request = GenerateContentRequest::builder()
        .contents(contents.clone())
        .tools(vec![tools.clone()])
        .build();

    let response = gemini_client
        .generate_content(&request, "gemini-3.1-flash-lite")
        .await?;

    if let Some(candidate) = response.candidates.first()
        && let Some(content) = &candidate.content
        && let Some(parts) = &content.parts
        && let Some(part) = parts.first()
        && let PartData::FunctionCall { id, name, .. } = &part.data
    {
        println!("Received function call: {name}");
        contents.push(content.clone());

        // Return a multimodal function response with an inline blob (e.g. chart image)
        // and a file reference (e.g. raw CSV data), demonstrating the new displayName
        // and fileData support.
        let inline_image = FunctionResponseBlob::new("image/png", "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==")
            .with_display_name("revenue_chart.png");

        // Demonstrates constructing a FunctionResponseFileData with displayName and mimeType.
        // In real-world usage, file_uri points to a file uploaded via the Gemini Files API.
        let _data_file = FunctionResponseFileData::new(
            "https://generativelanguage.googleapis.com/v1beta/files/quarterly_revenue_data",
        )
        .with_display_name("revenue_data.csv")
        .with_mime_type("text/csv");

        // FunctionResponsePart supports both InlineData (inline media bytes) and FileData (URI references).
        let function_response = FunctionResponse {
            id: id.clone(),
            name: name.clone(),
            response: json!({
                "chart": { "$ref": "revenue_chart.png" }
            }),
            parts: Some(vec![
                FunctionResponsePart::InlineData(inline_image),
                // FunctionResponsePart::FileData(_data_file), // Uncomment when referencing a valid Files API URI
            ]),
            will_continue: Some(false),
            scheduling: None,
        };

        contents.push(Content {
            role: Some(Role::User),
            parts: Some(vec![Part {
                thought: None,
                thought_signature: None,
                part_metadata: None,
                media_resolution: None,
                data: PartData::FunctionResponse(function_response),
            }]),
        });

        let follow_up = GenerateContentRequest::builder()
            .contents(contents)
            .tools(vec![tools])
            .build();

        let follow_up_response = gemini_client
            .generate_content(&follow_up, "gemini-3.1-flash-lite")
            .await?;

        if let Some(candidate) = follow_up_response.candidates.first() {
            println!("Model reply: {:?}", candidate.get_text());
        }
    } else {
        println!(
            "Model replied directly: {:?}",
            response.candidates.first().and_then(|c| c.get_text())
        );
    }

    Ok(())
}
