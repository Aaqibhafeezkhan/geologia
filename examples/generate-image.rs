// Copyright 2026 Andre Cipriani Bandarra
// SPDX-License-Identifier: Apache-2.0

use std::{env, error::Error, io::Cursor};

use geologia::prelude::{
    Content, GeminiClient, GenerateContentRequest, GenerationConfig, ImageConfig, Modality, Role,
};
use image::{ImageFormat, ImageReader};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt().init();
    let _ = dotenvy::dotenv();

    let api_key = env::var("GEMINI_API_KEY")?;
    let gemini_client = GeminiClient::new(api_key);

    let request = GenerateContentRequest::builder()
        .contents(vec![
            Content::builder()
                .role(Role::User)
                .add_text_part("Create an image of a tuxedo cat riding a rocket to the moon.")
                .build(),
        ])
        .generation_config(
            GenerationConfig::builder()
                .response_modalities([Modality::Image])
                .image_config(ImageConfig::builder().aspect_ratio("1:1").build())
                .build(),
        )
        .build();

    println!("Request: {}", serde_json::to_string(&request)?);

    let response = gemini_client
        .generate_content(&request, "gemini-2.5-flash-image")
        .await?;

    let candidate = response.candidates.first().ok_or_else(|| {
        format!(
            "No candidates returned in response (prompt_feedback: {:?})",
            response.prompt_feedback
        )
    })?;
    let (mime_type, image_bytes) = candidate.get_inline_data().ok_or_else(|| {
        format!(
            "No inline image data returned in candidate (finish_reason: {:?}, finish_message: {:?}, prompt_feedback: {:?})",
            candidate.finish_reason, candidate.finish_message, response.prompt_feedback
        )
    })??;

    let format = ImageFormat::from_mime_type(mime_type)
        .ok_or_else(|| format!("Unsupported image MIME type: {mime_type}"))?;
    let ext = format.extensions_str().first().copied().unwrap_or("png");
    let output_path = format!("output.{ext}");
    let img = ImageReader::with_format(Cursor::new(image_bytes), format).decode()?;
    img.save(&output_path)?;
    println!("Saved generated image ({mime_type}) to {output_path}");
    Ok(())
}
