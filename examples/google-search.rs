// Copyright 2026 Andre Cipriani Bandarra
// SPDX-License-Identifier: Apache-2.0

use std::{env, error::Error};

use geologia::prelude::{
    Content, GeminiClient, GenerateContentRequest, GoogleSearch, Interval, Role, SearchTypes,
    Tools, WebSearch,
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
                "What are the latest developments in Rust programming language this year?",
            )
            .build(),
    ];

    // Configure the Google Search grounding tool with search types and a time range filter.
    let google_search = GoogleSearch {
        search_types: Some(SearchTypes {
            web_search: Some(WebSearch {}),
            image_search: None,
        }),
        time_range_filter: Some(Interval::new(
            "2026-01-01T00:00:00Z",
            "2026-10-30T00:00:00Z",
        )),
    };

    let tools = Tools {
        google_search: Some(google_search),
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

    if let Some(citations) = &candidate.citation_metadata {
        println!("Citations (total: {}):", citations.citation_sources.len());
        for citation in &citations.citation_sources {
            println!(
                "  - [{:?}..{:?}] URI: {:?}",
                citation.start_index, citation.end_index, citation.uri
            );
            if let Some(license) = &citation.license {
                println!("    License: {}", license);
            }
        }
    } else {
        println!("No citation metadata returned for this response.");
    }

    if let Some(ratings) = &candidate.safety_ratings {
        println!("Safety ratings:");
        for rating in ratings {
            println!(
                "  - {}: {} (blocked: {:?})",
                rating.category, rating.probability, rating.blocked
            );
        }
    } else {
        println!("No safety ratings returned on candidate.");
    }

    if let Some(grounding) = &candidate.grounding_metadata {
        println!(
            "Grounding metadata:\n{}",
            serde_json::to_string_pretty(grounding)?
        );
    }

    Ok(())
}
