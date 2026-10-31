// Copyright 2026 Andre Cipriani Bandarra
// SPDX-License-Identifier: Apache-2.0

use std::{env, error::Error, time::Duration};

use geologia::prelude::{
    Content, DocumentState, FileSearch, FileSearchStore, GeminiClient, GenerateContentRequest,
    Role, Tools,
};

const MODEL: &str = "gemini-2.5-flash";
const POLL_INTERVAL: Duration = Duration::from_secs(2);
const MAX_POLL_ATTEMPTS: usize = 30;

const FIELD_NOTES: &str = "\
Project Geologia Field Notes: Mantiqueira Ridge
================================================
Observation Point: Pico das Agulhas Negras, 2,791 meters elevation.
Primary Mineralogy: Highly alkaline nepheline syenite with abundant aegirine-augite pyroxene.
Key Discovery: Hydrothermal quartz-carbonate veins contain anomalous rare earth element (REE) concentrations.
Recommendations: Further isotopic dating is scheduled for the northeastern exposure.
";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt().init();
    let _ = dotenvy::dotenv();

    let api_key = match env::var("GEMINI_API_KEY") {
        Ok(key) if !key.trim().is_empty() => key,
        _ => {
            eprintln!("GEMINI_API_KEY environment variable not set; skipping live example run.");
            return Ok(());
        }
    };

    let client = GeminiClient::new(api_key);

    // 1. Create a File Search store
    println!("1. Creating File Search store...");
    let store_req = FileSearchStore::builder()
        .display_name("Geologia Mantiqueira Research")
        .build();
    let store = client.create_file_search_store(&store_req).await?;
    let store_name = store.name.as_deref().ok_or("Store missing name")?;
    println!("   Created store: {store_name}");

    // Use a clean-up guard pattern to ensure the store is deleted even if subsequent steps fail.
    let result = run_file_search_workflow(&client, store_name).await;

    // 5. Clean up: delete the store and all contained documents
    println!("\n5. Cleaning up store {store_name} (force = true)...");
    if let Err(e) = client.delete_file_search_store(store_name, true).await {
        eprintln!("   Warning: failed to delete store {store_name}: {e}");
    } else {
        println!("   Store deleted successfully.");
    }

    result
}

async fn run_file_search_workflow(
    client: &GeminiClient,
    store_name: &str,
) -> Result<(), Box<dyn Error>> {
    // 2. Upload a document directly into the File Search store
    println!("\n2. Uploading document to {store_name}...");
    let mut operation = client
        .upload_to_file_search_store(
            store_name,
            FIELD_NOTES.as_bytes(),
            Some("text/plain"),
            Some("mantiqueira_field_notes.txt"),
        )
        .await?;
    println!(
        "   Initiated upload operation: {}",
        operation.name.as_deref().unwrap_or("unknown")
    );

    // 3. Wait for indexing to complete (polling operation and document)
    println!("\n3. Waiting for document indexing to complete...");
    let op_name = operation.name.clone().ok_or("Operation missing name")?;
    let mut attempts = 0;
    while !operation.is_done() {
        attempts += 1;
        if attempts > MAX_POLL_ATTEMPTS {
            return Err("Timed out waiting for file search operation to complete".into());
        }
        tokio::time::sleep(POLL_INTERVAL).await;
        operation = client.get_file_search_operation(&op_name).await?;
        println!(
            "   Operation polling attempt {attempts}: is_done = {}",
            operation.is_done()
        );
    }

    if let Some(err) = operation.error() {
        return Err(format!("Upload operation failed: {err:?}").into());
    }

    let store_details = client.get_file_search_store(store_name).await?;
    println!(
        "   Store after upload: active_docs = {:?}, size_bytes = {:?}",
        store_details.active_documents_count, store_details.size_bytes
    );

    let doc_name = operation
        .document_name()
        .ok_or("Could not determine document resource name from operation")?;
    println!("   Document resource name: {doc_name}");

    // Demonstrate document polling via get_document
    let mut doc = client.get_document(&doc_name).await?;
    let mut doc_attempts = 0;
    while doc.state != Some(DocumentState::Active) {
        if doc.state == Some(DocumentState::Failed) {
            return Err("Document processing failed on the server".into());
        }
        doc_attempts += 1;
        if doc_attempts > MAX_POLL_ATTEMPTS {
            return Err("Timed out waiting for document to become ACTIVE".into());
        }
        println!(
            "   Document state is {:?}; polling get_document again (attempt {doc_attempts}/{MAX_POLL_ATTEMPTS})...",
            doc.state
        );
        tokio::time::sleep(POLL_INTERVAL).await;
        doc = client.get_document(&doc_name).await?;
    }
    println!(
        "   Document is ACTIVE: {:?} (size: {:?} bytes)",
        doc.display_name.as_deref().unwrap_or(&doc_name),
        doc.size_bytes
    );

    // 4. Query the model through the fileSearch tool
    println!("\n4. Querying model {MODEL} with fileSearch tool...");
    let tools = Tools {
        file_search: Some(FileSearch::from_store(store_name)),
        ..Default::default()
    };

    let prompt =
        vec![Content::builder()
        .role(Role::User)
        .add_text_part(
            "What primary mineralogy and key discovery are noted at Pico das Agulhas Negras?",
        )
        .build()];

    let request = GenerateContentRequest::builder()
        .contents(prompt)
        .tools(vec![tools])
        .build();

    let response = client.generate_content(&request, MODEL).await?;

    if let Some(candidate) = response.candidates.first() {
        if let Some(text) = candidate.get_text() {
            println!("\nModel Answer:\n{text}");
        }
        if let Some(citations) = &candidate.citation_metadata {
            println!("\nCitations: {:?}", citations);
        }
        if let Some(grounding) = &candidate.grounding_metadata {
            println!(
                "\nGrounding Metadata: {}",
                serde_json::to_string_pretty(grounding)?
            );
        }
    } else {
        println!("No candidate returned in response.");
    }

    Ok(())
}
