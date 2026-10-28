// Copyright 2026 Andre Cipriani Bandarra
// SPDX-License-Identifier: Apache-2.0

use std::{env, error::Error};

use geologia::prelude::{CachedContent, Content, GeminiClient, GenerateContentRequest, Role};

const MODEL: &str = "gemini-2.5-flash";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt().init();
    let _ = dotenvy::dotenv();

    let api_key = env::var("GEMINI_API_KEY")?;
    let gemini_client = GeminiClient::new(api_key);

    // Explicit caching requires meeting the minimum token threshold (at least 2,048 tokens).
    // We assemble a large document prefix to cache.
    let document = "The quick brown fox jumps over the lazy dog. \
                    Context caching allows developers to store large amounts of precomputed input \
                    tokens (such as system instructions, extensive documentation, or books) \
                    server-side to reduce both latency and cost on repeated queries. \n"
        .repeat(100);

    println!("1. Creating explicit context cache with model {MODEL}...");
    let cache_request = CachedContent::builder()
        .model(MODEL)
        .display_name("Context Caching Example")
        .system_instruction(
            Content::builder()
                .add_text_part(
                    "You are an assistant answering questions about the cached document.",
                )
                .build(),
        )
        .contents(vec![
            Content::builder()
                .role(Role::User)
                .add_text_part(document)
                .build(),
        ])
        .ttl("300s")
        .build();

    let cache = gemini_client.create_cached_content(&cache_request).await?;
    let cache_name = cache
        .name
        .as_deref()
        .ok_or("Created cache did not include a resource name")?;
    println!("Created cache: {cache_name}");
    if let Some(usage) = &cache.usage_metadata {
        println!("Cached token count: {:?}", usage.total_token_count);
    }
    println!("Expires at: {:?}", cache.expire_time);

    // 2. Query the cache metadata using get_cached_content.
    println!("\n2. Fetching cache metadata via get_cached_content...");
    let fetched = gemini_client.get_cached_content(cache_name).await?;
    println!("Fetched cache display name: {:?}", fetched.display_name);

    // 3. Use the cache in a generateContent request.
    println!("\n3. Generating content using the cached context...");
    let generate_request = GenerateContentRequest::builder()
        .cached_content(cache_name)
        .contents(vec![
            Content::builder()
                .role(Role::User)
                .add_text_part("Summarize what this document is about in one short sentence.")
                .build(),
        ])
        .build();

    let response = gemini_client
        .generate_content(&generate_request, MODEL)
        .await?;

    if let Some(candidate) = response.candidates.first() {
        println!("Response: {:?}", candidate.get_text());
    }
    if let Some(usage) = &response.usage_metadata {
        println!(
            "Prompt tokens: {:?}, Cached tokens: {:?}",
            usage.prompt_token_count, usage.cached_content_token_count
        );
    }

    // 4. Delete the cache explicitly.
    println!("\n4. Deleting the cache...");
    gemini_client.delete_cached_content(cache_name).await?;
    println!("Cache {cache_name} deleted successfully.");

    Ok(())
}
