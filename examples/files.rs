// Copyright 2026 Andre Cipriani Bandarra
// SPDX-License-Identifier: Apache-2.0

use std::{env, error::Error, time::Duration};

use geologia::prelude::{
    Content, File, FileState, GeminiClient, GenerateContentRequest, Part, PartData, Role,
};
use serde_json::json;

const MODEL: &str = "gemini-2.5-flash";
const MIME_TYPE: &str = "text/plain";
const MAX_POLL_ATTEMPTS: usize = 30;
const POLL_INTERVAL: Duration = Duration::from_secs(2);

const SAMPLE_DOCUMENT: &str = "\
Project Geologia Field Notes
============================
Observation Site: Serra da Mantiqueira
Elevation: 2,798 meters (Pedra da Mina)
Rock Composition: Alkaline igneous nepheline syenite and phonolite formations.
Key Finding: Distinct hydrothermal alteration zones observed along the northeastern ridge.
";

async fn wait_until_active(
    client: &GeminiClient,
    mut file: File,
    file_name: &str,
) -> Result<File, Box<dyn Error>> {
    let mut attempts = 0;
    while file.state != Some(FileState::Active) {
        if file.state == Some(FileState::Failed) {
            return Err(format!(
                "File {file_name} failed asynchronous processing: {:?}",
                file.error
            )
            .into());
        }
        attempts += 1;
        if attempts > MAX_POLL_ATTEMPTS {
            return Err(format!(
                "Timed out waiting for {file_name} to reach ACTIVE (last state: {:?})",
                file.state
            )
            .into());
        }
        println!(
            "  State is {:?}; polling get_file again in {:?} (attempt {attempts}/{MAX_POLL_ATTEMPTS})...",
            file.state, POLL_INTERVAL
        );
        tokio::time::sleep(POLL_INTERVAL).await;
        file = client.get_file(file_name).await?;
    }
    Ok(file)
}

async fn use_uploaded_file(
    client: &GeminiClient,
    uploaded: File,
    file_name: &str,
) -> Result<(), Box<dyn Error>> {
    // 2. Wait for asynchronous processing (`PROCESSING` -> `ACTIVE`) before using `file.uri`.
    println!("\n2. Ensuring file is ACTIVE before inference...");
    let active_file = wait_until_active(client, uploaded, file_name).await?;
    let file_uri = active_file
        .uri
        .as_deref()
        .ok_or("Active file did not include a URI")?;
    let mime_type = active_file.mime_type.as_deref().unwrap_or(MIME_TYPE);
    println!("  File is ACTIVE! URI: {file_uri}");

    // 3. List project files (`list_files`) and confirm our file is visible.
    println!("\n3. Listing files via list_files(page_size = 10)...");
    let list_response = client.list_files(Some(10), None).await?;
    let present_in_list = list_response
        .files
        .iter()
        .any(|f| f.name.as_deref() == Some(file_name));
    println!(
        "  Returned {} file(s) on first page (uploaded file present = {present_in_list})",
        list_response.files.len()
    );

    // 4. Reference the active file in `generate_content` through `PartData::FileData`.
    println!("\n4. Referencing {file_uri} in generate_content with model {MODEL}...");
    let file_part = Part {
        thought: None,
        thought_signature: None,
        part_metadata: None,
        media_resolution: None,
        video_metadata: None,
        speech_metadata: None,
        data: PartData::FileData(json!({
            "fileUri": file_uri,
            "mimeType": mime_type
        })),
    };

    let request = GenerateContentRequest::builder()
        .contents(vec![
            Content::builder()
                .role(Role::User)
                .add_part(file_part)
                .add_text_part(
                    "Based on the attached field notes, what is the elevation and rock composition?",
                )
                .build(),
        ])
        .build();

    let response = client.generate_content(&request, MODEL).await?;
    if let Some(candidate) = response.candidates.first() {
        println!(
            "  Model response: {}",
            candidate.get_text().unwrap_or_default().trim()
        );
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt().init();
    let _ = dotenvy::dotenv();

    let api_key = env::var("GEMINI_API_KEY")?;
    let gemini_client = GeminiClient::new(api_key);

    // 1. Upload a file via the resumable Files API (`POST /upload/v1beta/files`).
    println!("1. Uploading sample field notes via upload_file...");
    let uploaded = gemini_client
        .upload_file(
            SAMPLE_DOCUMENT.as_bytes(),
            MIME_TYPE,
            Some("Geologia Field Notes.txt"),
        )
        .await?;
    let file_name = uploaded
        .name
        .clone()
        .ok_or("Uploaded file did not include a resource name")?;
    println!(
        "  Uploaded resource: {file_name} (display_name = {:?}, size_bytes = {:?}, state = {:?})",
        uploaded.display_name, uploaded.size_bytes, uploaded.state
    );

    let run_result = use_uploaded_file(&gemini_client, uploaded, &file_name).await;

    // 5. Always delete the temporary file (`delete_file`), even if an earlier step failed.
    println!("\n5. Deleting {file_name} via delete_file...");
    let delete_result = gemini_client.delete_file(&file_name).await;
    if delete_result.is_ok() {
        println!("  Deleted {file_name} successfully.");
    }

    run_result?;
    delete_result?;
    Ok(())
}
