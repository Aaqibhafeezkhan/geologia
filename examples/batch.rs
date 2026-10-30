// Copyright 2026 Andre Cipriani Bandarra
// SPDX-License-Identifier: Apache-2.0

use std::{env, error::Error, time::Duration};

use geologia::prelude::{
    BatchGenerateContentRequest, BatchOperation, BatchState, Content, GeminiClient,
    GenerateContentBatch, GenerateContentRequest, InputConfig, Role,
};

const MODEL: &str = "gemini-2.5-flash";
const MAX_POLL_ATTEMPTS: usize = 60;
const POLL_INTERVAL: Duration = Duration::from_secs(5);

async fn wait_until_complete(
    client: &GeminiClient,
    batch_name: &str,
) -> Result<BatchOperation, Box<dyn Error>> {
    let mut attempts = 0;
    loop {
        let op = client.get_batch(batch_name).await?;
        let state = op.state().unwrap_or(BatchState::Unspecified);
        println!(
            "  Poll #{attempts}: is_done = {}, state = {state:?}",
            op.is_done()
        );

        if op.is_done()
            || matches!(
                state,
                BatchState::Succeeded
                    | BatchState::Failed
                    | BatchState::Cancelled
                    | BatchState::Expired
            )
        {
            if state == BatchState::Succeeded && op.error.is_none() {
                return Ok(op);
            }
            return Err(format!(
                "Batch {batch_name} finished in state {state:?} (error: {:?})",
                op.error
            )
            .into());
        }

        attempts += 1;
        if attempts >= MAX_POLL_ATTEMPTS {
            return Err(format!(
                "Timed out after {attempts} attempts waiting for batch {batch_name} to complete"
            )
            .into());
        }

        tokio::time::sleep(POLL_INTERVAL).await;
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt().init();
    let _ = dotenvy::dotenv();

    let api_key = env::var("GEMINI_API_KEY")?;
    let client = GeminiClient::new(api_key);

    // 1. Build inline requests for batch processing.
    println!("1. Creating inline batch generation request...");
    let req1 = GenerateContentRequest::builder()
        .contents(vec![
            Content::builder()
                .role(Role::User)
                .add_text_part("Explain photosynthesis in one concise sentence.")
                .build(),
        ])
        .build();

    let req2 = GenerateContentRequest::builder()
        .contents(vec![
            Content::builder()
                .role(Role::User)
                .add_text_part("Explain cellular respiration in one concise sentence.")
                .build(),
        ])
        .build();

    let batch_request = BatchGenerateContentRequest::new(
        GenerateContentBatch::builder()
            .display_name("biology-comparison-batch")
            .input_config(InputConfig::from_inline(vec![req1, req2]))
            .build(),
    );

    // 2. Submit the batch job via `POST models/{model}:batchGenerateContent`.
    println!("\n2. Submitting batch request via batch_generate_content...");
    let initial_op = client.batch_generate_content(&batch_request, MODEL).await?;
    let batch_name = initial_op
        .name
        .clone()
        .ok_or("Initial batch operation did not include a resource name")?;
    println!(
        "  Submitted batch: {batch_name} (initial state: {:?})",
        initial_op.state()
    );

    // 3. List recent batches (`list_batches`) to demonstrate project-level discovery.
    println!("\n3. Listing recent batches via list_batches(page_size = 5)...");
    let list_response = client.list_batches(Some(5), None).await?;
    let present_in_list = list_response
        .batches()
        .any(|b| b.name.as_deref() == Some(&batch_name));
    println!(
        "  Found {} batch operation(s) on first page (new batch visible = {present_in_list})",
        list_response.operations.len()
    );

    // 4. Poll `get_batch` until the asynchronous job completes.
    println!(
        "\n4. Polling get_batch for {batch_name} until complete (poll interval: {POLL_INTERVAL:?})..."
    );
    let completed_op = wait_until_complete(&client, &batch_name).await;

    // 5. Inspect and print results if batch polling succeeded.
    if let Ok(ref op) = completed_op {
        println!("\n5. Inspecting batch results...");
        if let Some(batch) = op.batch()
            && let Some(stats) = &batch.batch_stats
        {
            println!(
                "  Batch Stats: total = {:?}, successful = {:?}, failed = {:?}",
                stats.request_count, stats.successful_request_count, stats.failed_request_count
            );
        }

        if let Some(inlined_responses) = op.inlined_responses() {
            println!(
                "  Received {} inlined response(s):",
                inlined_responses.len()
            );
            for (idx, resp) in inlined_responses.iter().enumerate() {
                if let Some(result) = resp.result() {
                    let text = result
                        .candidates
                        .first()
                        .and_then(|c| c.get_text())
                        .unwrap_or_default();
                    println!("    [{idx}] Answer: {}", text.trim());
                } else if let Some(err) = &resp.error {
                    println!(
                        "    [{idx}] Error: code = {:?}, message = {:?}",
                        err.code, err.message
                    );
                }
            }
        } else if let Some(output) = op.output()
            && let Some(responses_file) = &output.responses_file
        {
            println!("  Results stored in file: {responses_file}");
        }
    }

    // 6. Clean up the batch resource via `DELETE batches/{name}`.
    println!("\n6. Cleaning up batch resource via delete_batch...");
    let delete_result = client.delete_batch(&batch_name).await;
    match delete_result {
        Ok(()) => println!("  Batch {batch_name} deleted successfully."),
        Err(ref err) => eprintln!("  Warning: failed to delete batch {batch_name}: {err}"),
    }

    completed_op.map(|_| ())?;
    delete_result?;
    Ok(())
}
