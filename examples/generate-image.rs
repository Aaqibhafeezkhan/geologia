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

    let candidate = response
        .candidates
        .first()
        .ok_or("No candidates returned in response")?;
    let (mime_type, image_bytes) = candidate
        .get_inline_data()
        .ok_or("No inline image data returned in candidate")??;

    let format = ImageFormat::from_mime_type(mime_type)
        .ok_or_else(|| format!("Unsupported image MIME type: {mime_type}"))?;
    let img = ImageReader::with_format(Cursor::new(image_bytes), format).decode()?;
    img.save("output.jpg")?;
    println!("Saved generated image ({mime_type}) to output.jpg");
    Ok(())
}
