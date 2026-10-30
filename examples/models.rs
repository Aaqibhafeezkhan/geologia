// Copyright 2026 Andre Cipriani Bandarra
// SPDX-License-Identifier: Apache-2.0

use std::{env, error::Error};

use geologia::prelude::{GeminiClient, Model};

const PREFERRED_MODEL: &str = "gemini-2.5-flash";
const PAGE_SIZE: u32 = 25;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt().init();
    let _ = dotenvy::dotenv();

    let api_key = env::var("GEMINI_API_KEY")?;
    let gemini_client = GeminiClient::new(api_key);

    // 1. List all available models across pages using `list_models`.
    println!("1. Listing available models (page_size = {PAGE_SIZE})...");
    let mut all_models: Vec<Model> = Vec::new();
    let mut page_token: Option<String> = None;
    let mut page_num = 1;

    loop {
        let response = gemini_client
            .list_models(Some(PAGE_SIZE), page_token.as_deref())
            .await?;
        println!(
            "  Page {page_num}: received {} model(s) (has_next_page = {})",
            response.models.len(),
            response.next_page_token.is_some()
        );
        all_models.extend(response.models);

        match response.next_page_token {
            Some(token) => {
                page_token = Some(token);
                page_num += 1;
            }
            None => break,
        }
    }

    println!("\nTotal models discovered: {}", all_models.len());

    let content_models: Vec<&Model> = all_models
        .iter()
        .filter(|m| {
            m.supported_generation_methods
                .iter()
                .any(|method| method == "generateContent")
        })
        .collect();
    let embedding_models: Vec<&Model> = all_models
        .iter()
        .filter(|m| {
            m.supported_generation_methods
                .iter()
                .any(|method| method == "embedContent")
        })
        .collect();
    let thinking_models: Vec<&Model> = all_models
        .iter()
        .filter(|m| m.thinking == Some(true))
        .collect();

    println!(
        "  Supports generateContent: {} | Supports embedContent: {} | Supports thinking: {}",
        content_models.len(),
        embedding_models.len(),
        thinking_models.len()
    );

    println!("\nSample models supporting generateContent:");
    for model in content_models.iter().take(10) {
        println!(
            "  - {} ({}) | in: {:?} tokens, out: {:?} tokens, thinking: {:?}",
            model.name,
            model.display_name.as_deref().unwrap_or("unnamed"),
            model.input_token_limit,
            model.output_token_limit,
            model.thinking.unwrap_or(false)
        );
    }

    // 2. Fetch detailed metadata for a single model using `get_model`.
    // Pass either a short model ID (e.g. `gemini-2.5-flash`) or full `models/...` name.
    let target_name = all_models
        .iter()
        .find(|m| m.name == format!("models/{PREFERRED_MODEL}"))
        .map(|_| PREFERRED_MODEL)
        .or_else(|| content_models.first().map(|m| m.name.as_str()))
        .or_else(|| all_models.first().map(|m| m.name.as_str()));

    if let Some(model_name) = target_name {
        println!("\n2. Inspecting single model via get_model({model_name:?})...");
        let model = gemini_client.get_model(model_name).await?;

        println!("  Resource name:     {}", model.name);
        println!("  Base model ID:     {:?}", model.base_model_id);
        println!("  Version:           {:?}", model.version);
        println!("  Display name:      {:?}", model.display_name);
        println!("  Description:       {:?}", model.description);
        println!("  Input token limit: {:?}", model.input_token_limit);
        println!("  Output limit:      {:?}", model.output_token_limit);
        println!(
            "  Supported methods: {:?}",
            model.supported_generation_methods
        );
        println!(
            "  Defaults:          temperature={:?} (max={:?}), top_p={:?}, top_k={:?}, thinking={:?}",
            model.temperature, model.max_temperature, model.top_p, model.top_k, model.thinking
        );
    }

    Ok(())
}
