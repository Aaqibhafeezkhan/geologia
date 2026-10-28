// Copyright 2026 Andre Cipriani Bandarra
// SPDX-License-Identifier: Apache-2.0

use std::{env, error::Error};

use geologia::prelude::{
    Content, CountTokensRequestBuilder, FunctionDeclaration, GeminiClient, GenerateContentRequest,
    Role, Tools,
};
use serde_json::json;

const MODEL: &str = "gemini-3.1-flash-lite";
const PROMPT: &str = "What is the airspeed of an unladen swallow?";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt().init();
    let _ = dotenvy::dotenv();

    let api_key = env::var("GEMINI_API_KEY")?;
    let gemini_client = GeminiClient::new(api_key);

    // Count a plain text prompt.
    let request = CountTokensRequestBuilder::from_prompt(PROMPT).build();
    let prompt_only = gemini_client.count_tokens(&request, MODEL).await?;
    println!("Prompt only: {prompt_only:#?}");

    // Count a full generateContent request, including system instruction and tools.
    let speed_function = FunctionDeclaration {
        name: String::from("bird_airspeed"),
        description: String::from("Looks up the cruising airspeed of a bird species"),
        behavior: None,
        parameters: None,
        parameters_json_schema: Some(json!({
            "type": "object",
            "properties": {
                "species": {
                    "type": "string",
                    "description": "The bird species, e.g. 'European swallow'"
                }
            },
            "required": ["species"]
        })),
        response: None,
        response_json_schema: None,
    };

    let generate_content_request = GenerateContentRequest::builder()
        .system_instruction(
            Content::builder()
                .add_text_part("You are an ornithologist. Answer in one sentence.")
                .build(),
        )
        .contents(vec![
            Content::builder()
                .role(Role::User)
                .add_text_part(PROMPT)
                .build(),
        ])
        .tools(vec![Tools {
            function_declarations: Some(vec![speed_function]),
            ..Default::default()
        }])
        .build();

    let request =
        CountTokensRequestBuilder::from_generate_content_request(generate_content_request).build();
    let full_request = gemini_client.count_tokens(&request, MODEL).await?;
    println!("Full request: {full_request:#?}");

    println!(
        "System instruction and tools add {} tokens",
        full_request.total_tokens - prompt_only.total_tokens
    );
    Ok(())
}
