// Copyright 2026 Andre Cipriani Bandarra
// SPDX-License-Identifier: Apache-2.0

use std::{env, error::Error};

use geologia::prelude::{BatchEmbedContentsRequest, EmbedContentRequest, GeminiClient, TaskType};

const MODEL: &str = "gemini-embedding-001";
const DIMENSIONS: i32 = 256;

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot: f32 = a.iter().zip(b).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        dot / (norm_a * norm_b)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt().init();
    let _ = dotenvy::dotenv();

    let api_key = env::var("GEMINI_API_KEY")?;
    let gemini_client = GeminiClient::new(api_key);

    // 1. Embed a single search query with `embed_content`.
    let query = "How does Rust prevent data races without a garbage collector?";
    let query_request = EmbedContentRequest::builder()
        .text(query)
        .task_type(TaskType::RetrievalQuery)
        .output_dimensionality(DIMENSIONS)
        .build();

    let query_response = gemini_client.embed_content(&query_request, MODEL).await?;
    println!(
        "Query embedding dimension: {} (first 5 values: {:?})",
        query_response.embedding.values.len(),
        &query_response.embedding.values[..query_response.embedding.values.len().min(5)]
    );
    if let Some(usage) = &query_response.usage_metadata {
        println!("Query usage metadata: {usage:?}");
    }

    // 2. Embed a batch of candidate documents with `batch_embed_contents`.
    let documents = [
        (
            "Rust Ownership and Borrowing",
            "Rust enforces ownership and borrowing rules at compile time so mutable aliasing and data races are rejected before the program runs.",
        ),
        (
            "Sourdough Bread Baking",
            "Fermenting flour and water with wild yeast and lactic acid bacteria produces an airy crumb and tangy crust.",
        ),
        (
            "Plate Tectonics",
            "Earth's lithosphere is divided into tectonic plates that move slowly over the ductile asthenosphere.",
        ),
    ];

    let batch_request = BatchEmbedContentsRequest::builder()
        .requests(
            documents
                .iter()
                .map(|(title, body)| {
                    EmbedContentRequest::builder()
                        .text(*body)
                        .task_type(TaskType::RetrievalDocument)
                        .title(*title)
                        .output_dimensionality(DIMENSIONS)
                        .build()
                })
                .collect(),
        )
        .build();

    let batch_response = gemini_client
        .batch_embed_contents(&batch_request, MODEL)
        .await?;
    println!(
        "\nEmbedded {} documents in a single batch.",
        batch_response.embeddings.len()
    );

    println!("\nCosine similarity to query ({query:?}):");
    for ((title, _), embedding) in documents.iter().zip(&batch_response.embeddings) {
        let score = cosine_similarity(&query_response.embedding.values, &embedding.values);
        println!("  {score:.4} — {title}");
    }

    Ok(())
}
