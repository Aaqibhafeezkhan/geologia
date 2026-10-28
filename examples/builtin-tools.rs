// Copyright 2026 Andre Cipriani Bandarra
// SPDX-License-Identifier: Apache-2.0

use std::{env, error::Error};

use geologia::prelude::{
    CodeExecution, Content, GeminiClient, GenerateContentRequest, Role, Tools, UrlContext,
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
                "Summarize https://www.rust-lang.org in one sentence, then use code to \
                 count the words in your summary.",
            )
            .build(),
    ];

    let tools = Tools {
        code_execution: Some(CodeExecution {}),
        url_context: Some(UrlContext {}),
        ..Default::default()
    };

    let request = GenerateContentRequest::builder()
        .contents(prompt)
        .tools(vec![tools])
        .build();
    let response = gemini_client
        .generate_content(&request, "gemini-3.1-flash-lite")
        .await?;

    let candidate = &response.candidates[0];
    println!("Response: {:?}", candidate.get_text());
    println!("Finish reason: {:?}", candidate.finish_reason);
    if let Some(metadata) = &candidate.url_context_metadata {
        println!(
            "URL context metadata: {}",
            serde_json::to_string_pretty(metadata)?
        );
    }
    Ok(())
}
