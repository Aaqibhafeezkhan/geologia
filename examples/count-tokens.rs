// Copyright 2026 Andre Cipriani Bandarra
// SPDX-License-Identifier: Apache-2.0

use std::{env, error::Error};

use geologia::prelude::{CountTokensRequestBuilder, GeminiClient};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt().init();
    let _ = dotenvy::dotenv();

    let api_key = env::var("GEMINI_API_KEY")?;
    let gemini_client = GeminiClient::new(api_key);

    let request =
        CountTokensRequestBuilder::from_prompt("What is the airspeed of an unladen swallow?")
            .build();
    let response = gemini_client
        .count_tokens(&request, "gemini-3.1-flash-lite")
        .await?;

    println!("Response: {response:#?}");
    Ok(())
}
