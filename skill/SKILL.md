---
name: geologia
description: >
  Help Rust developers use the geologia crate to integrate Google's Generative AI (Gemini) API
  into their projects. Use this skill whenever the user is working with geologia, building Rust
  apps that call Gemini models, generating text or images from Rust, using function calling with
  Gemini in Rust, or asking how to add AI features to a Rust project using the geologia crate.
  Also trigger when the user asks about listing or inspecting available Gemini models, uploading
  or managing media files via the Gemini Files API, asynchronous batch generation via the Gemini
  Batch API, streaming Gemini responses, token counting, text embeddings, or structured output in
  Rust — even if they don't explicitly mention geologia.
---

# geologia

`geologia` is an async Rust client for Google's Generative AI API (Gemini). It supports model
discovery (`models.list` and `models.get`), media file management (`files` / `media.upload`),
asynchronous batch generation (`batches` / `batchGenerateContent`), File Search stores for semantic
retrieval and grounding (`fileSearchStores`), text generation (including streaming), token counting,
text embeddings, image generation (Imagen), context caching, and function calling.

## Adding the dependency

```toml
[dependencies]
geologia = "0.1"
tokio = { version = "1", features = ["full"] }
```

For streaming, also add:
```toml
tokio-stream = "0.1"
```

## Authentication

`GeminiClient` takes an API key string. The standard pattern is to read it from the environment:

```rust
let api_key = std::env::var("GEMINI_API_KEY")?;
let client = GeminiClient::new(api_key);

// Or configure a custom base URL or reqwest::Client via the builder:
let client = GeminiClient::builder(api_key)
    .base_url("https://my-proxy.internal/v1beta")
    .http_client(custom_http_client)
    .build();
```

## Imports

Everything is re-exported from `geologia::prelude`:

```rust
use geologia::prelude::*;
```

---

## Text generation

```rust
use geologia::prelude::{Content, GeminiClient, GenerateContentRequest, Role};

let client = GeminiClient::new(std::env::var("GEMINI_API_KEY")?);

let request = GenerateContentRequest::builder()
    .contents(vec![
        Content::builder()
            .role(Role::User)
            .add_text_part("Explain Rust lifetimes in simple terms.")
            .build(),
    ])
    .build();

let response = client.generate_content(&request, "gemini-3.1-pro-preview").await?;
println!("{}", response.candidates[0].get_text().unwrap());
```

## Streaming text generation

```rust
use geologia::prelude::{Content, GeminiClient, GenerateContentRequest, Role};
use tokio_stream::StreamExt;

let mut stream = client
    .stream_generate_content(&request, "gemini-3.1-pro-preview")
    .await?;

while let Some(chunk) = stream.next().await {
    match chunk {
        Ok(response) => print!("{}", response.candidates[0].get_text().unwrap()),
        Err(err) => eprintln!("Error: {}", err),
    }
}
```

## Multi-turn conversation

Build `contents` as a `Vec<Content>`, alternating `Role::User` and `Role::Model`:

```rust
let contents = vec![
    Content::builder()
        .role(Role::User)
        .add_text_part("Hello!")
        .build(),
    Content::builder()
        .role(Role::Model)
        .add_text_part("Hi! How can I help?")
        .build(),
    Content::builder()
        .role(Role::User)
        .add_text_part("What's 2 + 2?")
        .build(),
];

let request = GenerateContentRequest::builder().contents(contents).build();
```

## System instructions

```rust
let request = GenerateContentRequest::builder()
    .system_instruction(
        Content::builder()
            .add_text_part("You are a terse Rust expert. Be concise.")
            .build(),
    )
    .contents(vec![...])
    .build();
```

## Generation config

```rust
use geologia::prelude::{
    GenerationConfig, MediaResolution, Modality, ThinkingConfig, ThinkingLevel,
};

let config = GenerationConfig::builder()
    .temperature(0.7_f32)
    .max_output_tokens(1024)
    .top_p(0.9_f32)
    .seed(42)
    .response_modalities([Modality::Text])
    .media_resolution(MediaResolution::Medium)
    .thinking_config(
        ThinkingConfig::builder()
            .thinking_level(ThinkingLevel::Minimal)
            .include_thoughts(false)
            .build(),
    )
    .build();

let request = GenerateContentRequest::builder()
    .generation_config(config)
    .contents(vec![...])
    .build();
```

## Other request options

```rust
use std::collections::HashMap;
use geologia::prelude::{GenerateContentRequest, ServiceTier};

let request = GenerateContentRequest::builder()
    .contents(vec![...])
    .cached_content("cachedContents/my-cache") // reuse a context cache
    .service_tier(ServiceTier::Flex)
    .store(false)
    .labels(HashMap::from([("team".to_string(), "search".to_string())]))
    .build();
```

## Response metadata

```rust
let response = client.generate_content(&request, "gemini-3.8-flash").await?;
println!("{:?} {:?}", response.model_version, response.response_id);
if let Some(status) = &response.model_status {
    println!("stage: {:?}, retires: {:?}", status.model_stage, status.retirement_time);
}
```

## Continuing generation

When a candidate stops with `finish_reason == Some("CONTINUATION")`, it carries a
`continuation_token`. Resend the **identical** `contents` with that token to resume:

```rust
let candidate = &response.candidates[0];
if candidate.finish_reason.as_deref() == Some("CONTINUATION") {
    if let Some(token) = &candidate.continuation_token {
        let request = GenerateContentRequest::builder()
            .contents(original_contents.clone()) // must match the initial request exactly
            .continuation_token(token.clone())
            .build();
        let next = client.generate_content(&request, "gemini-3.8-flash").await?;
    }
}
```

---

## Built-in tools (code execution, URL context)

```rust
use geologia::prelude::{CodeExecution, Tools, UrlContext};

let tools = Tools {
    code_execution: Some(CodeExecution {}),
    url_context: Some(UrlContext {}),
    ..Default::default()
};

let request = GenerateContentRequest::builder()
    .contents(vec![...])
    .tools(vec![tools])
    .build();
let response = client.generate_content(&request, "gemini-3.8-flash").await?;
// Grounding and URL context details are raw JSON (`serde_json::Value`) for now.
println!("{:?}", response.candidates[0].url_context_metadata);
```

`google_maps`, `computer_use` and `mcp_servers` are also on `Tools` as raw
`serde_json::Value` configs. For semantic retrieval and grounding over File Search stores, use the
strongly typed `Tools { file_search: Some(FileSearch::from_store("fileSearchStores/my-store")), .. }`.

---

## Function calling

Declare functions using JSON Schema, then handle the model's `FunctionCall` response:

```rust
use geologia::prelude::{
    Content, FunctionDeclaration, FunctionResponse, GeminiClient,
    GenerateContentRequest, Part, PartData, Role, Tools,
};
use serde_json::json;

let function = FunctionDeclaration {
    name: "get_weather".to_string(),
    description: "Get current weather for a city".to_string(),
    behavior: None,
    parameters: None,
    parameters_json_schema: Some(json!({
        "type": "object",
        "properties": {
            "city": { "type": "string", "description": "City name" }
        },
        "required": ["city"]
    })),
    response: None,
    response_json_schema: Some(json!({
        "type": "object",
        "properties": {
            "temperature": { "type": "number" },
            "condition": { "type": "string" }
        }
    })),
};

let tools = Tools {
    function_declarations: Some(vec![function]),
    ..Default::default()
};

let mut contents = vec![
    Content::builder()
        .role(Role::User)
        .add_text_part("What's the weather in London?")
        .build(),
];

let request = GenerateContentRequest::builder()
    .contents(contents.clone())
    .tools(vec![tools.clone()])
    .build();

let response = client.generate_content(&request, "gemini-3.1-pro-preview").await?;

// Check if the model wants to call a function
if let Some(candidate) = response.candidates.last()
    && let Some(content) = &candidate.content
    && let Some(parts) = &content.parts
    && let Some(part) = parts.last()
    && let PartData::FunctionCall { id, name, args, .. } = &part.data
{
    // Execute your function, then send back the result
    contents.push(content.clone());
    contents.push(Content {
        role: Some(Role::User),
        parts: Some(vec![Part {
            data: PartData::FunctionResponse(FunctionResponse {
                id: id.clone(),
                name: name.clone(),
                response: json!({ "temperature": 15.0, "condition": "cloudy" }),
                will_continue: None,
                parts: None,
                scheduling: None,
            }),
            media_resolution: None,
            part_metadata: None,
            video_metadata: None,
            speech_metadata: None,
            thought: None,
            thought_signature: part.thought_signature.clone(),
        }]),
    });

    let request = GenerateContentRequest::builder()
        .contents(contents)
        .tools(vec![tools])
        .build();
    let final_response = client.generate_content(&request, "gemini-3.1-pro-preview").await?;
    println!("{}", final_response.candidates[0].get_text().unwrap());
}
```

---

## Image generation (Imagen)

```rust
use geologia::prelude::{
    GeminiClient, PersonGeneration, PredictImageRequest, PredictImageRequestParameters,
    PredictImageRequestParametersOutputOptions, PredictImageRequestPrompt,
    PredictImageSafetySetting,
};

let request = PredictImageRequest {
    instances: vec![PredictImageRequestPrompt {
        prompt: "A tuxedo cat riding a rocket to the moon".to_string(),
    }],
    parameters: PredictImageRequestParameters {
        sample_count: 1,
        aspect_ratio: Some("16:9".to_string()),
        output_options: Some(PredictImageRequestParametersOutputOptions {
            mime_type: Some("image/jpeg".to_string()),
            compression_quality: Some(85),
        }),
        person_generation: Some(PersonGeneration::AllowAdult),
        safety_setting: Some(PredictImageSafetySetting::BlockLowAndAbove),
        ..Default::default()
    },
};

let mut result = client
    .predict_image(&request, "imagen-4.0-generate-001")
    .await?;

let image = result.predictions.pop().unwrap();
// image.bytes_base64_encoded: Vec<u8> — the raw image bytes
// image.mime_type: String — e.g., "image/jpeg"
std::fs::write("output.jpg", &image.bytes_base64_encoded)?;
```

---

## Token counting

```rust
use geologia::prelude::{CountTokensRequestBuilder, GeminiClient};

let request = CountTokensRequestBuilder::from_prompt("How many tokens is this?").build();

let result = client.count_tokens(&request, "gemini-3.1-pro-preview").await?;
println!("Total tokens: {}", result.total_tokens);
// Optional per-modality breakdown, e.g. [ModalityTokenCount { modality: Text, token_count: 6 }]
println!("Details: {:?}", result.prompt_tokens_details);
```

Only `total_tokens` is always present. `total_billable_characters` is Vertex AI only and is
`None` on the Gemini API.

To count a full request, including system instruction, tools and multi-turn history, wrap a
`GenerateContentRequest`. `count_tokens` sets the inner request's `model` from its `model`
argument, so you don't repeat it:

```rust
use geologia::prelude::{Content, CountTokensRequestBuilder, GenerateContentRequest, Role};

let generate_content_request = GenerateContentRequest::builder()
    .system_instruction(Content::builder().add_text_part("Answer in one sentence.").build())
    .contents(vec![
        Content::builder().role(Role::User).add_text_part("How many tokens is this?").build(),
    ])
    .tools(tools)
    .build();

let request =
    CountTokensRequestBuilder::from_generate_content_request(generate_content_request).build();
let result = client.count_tokens(&request, "gemini-3.1-pro-preview").await?;
```

`CountTokensRequest::contents` is a `Vec<Content>`; use `CountTokensRequestBuilder::contents` to
count a multi-turn history without the rest of a request. See `examples/count-tokens.rs` for a
runnable version.

---

## Context caching (explicit caching)

Pre-compute and reuse large prefix tokens (such as documents, system instructions, or tool
declarations) across requests:

```rust
use geologia::prelude::{CachedContent, Content, GeminiClient, GenerateContentRequest, Role};

let client = GeminiClient::new(std::env::var("GEMINI_API_KEY")?);

// 1. Create a cache (must meet the model's minimum token threshold, e.g. 2,048 tokens).
let cache_request = CachedContent::builder()
    .model("gemini-2.5-flash")
    .display_name("Reference Doc")
    .system_instruction(Content::builder().add_text_part("You are a helpful analyst.").build())
    .contents(vec![
        Content::builder().role(Role::User).add_text_part(large_document_text).build(),
    ])
    .ttl("300s")
    .build();

let cache = client.create_cached_content(&cache_request).await?;
let cache_name = cache.name.unwrap();

// 2. Reference the cache in generate_content.
let request = GenerateContentRequest::builder()
    .cached_content(&cache_name)
    .contents(vec![
        Content::builder().role(Role::User).add_text_part("Summarize the key points.").build(),
    ])
    .build();

let response = client.generate_content(&request, "gemini-2.5-flash").await?;

// 3. Delete the cache when done.
client.delete_cached_content(&cache_name).await?;
```

Other client methods:
- `client.get_cached_content(name)`: fetch cache metadata and token usage.
- `client.list_cached_contents(page_size, page_token)`: paginated listing of caches.
- `client.update_cached_content(name, &patch, update_mask)`: update cache TTL or expiration.
- `client.delete_cached_content(name)`: explicitly remove a cache.

See `examples/context-caching.rs` for a runnable version.

---

## Uploading and referencing files (Files API)

Upload media or documents via the resumable Files API (`upload_file` or
`upload_file_with_metadata`) and reuse their URI across `generate_content` requests without
re-sending raw bytes inline. Note that `upload_file` and `upload_file_with_metadata` copy the
input `&[u8]` slice into an in-memory request body and send it in a single `upload, finalize`
request, so the payload must fit comfortably in memory.

Uploaded files (especially video, audio, and large PDFs) are processed asynchronously
(`FileState::Processing` -> `FileState::Active`, or `FileState::Failed`). Always wait until
`file.state == Some(FileState::Active)` before referencing `file.uri`:

```rust
use std::time::Duration;
use geologia::prelude::{
    Content, FileState, GeminiClient, GenerateContentRequest, Part, PartData, Role,
};
use serde_json::json;

let client = GeminiClient::new(std::env::var("GEMINI_API_KEY")?);
let bytes = std::fs::read("report.pdf")?;

// 1. Upload bytes via the resumable upload endpoint (`POST /upload/v1beta/files`).
let mut file = client
    .upload_file(&bytes, "application/pdf", Some("Quarterly Report.pdf"))
    .await?;
let file_name = file.name.clone().expect("server assigns files/{id}");

// 2. Poll `get_file` until processing completes (`FileState::Active`).
while file.state != Some(FileState::Active) {
    if file.state == Some(FileState::Failed) {
        return Err(format!("File processing failed: {:?}", file.error).into());
    }
    tokio::time::sleep(Duration::from_secs(2)).await;
    file = client.get_file(&file_name).await?;
}

// 3. Reference `file.uri` in `generate_content` via `PartData::FileData`.
let file_part = Part {
    thought: None,
    thought_signature: None,
    part_metadata: None,
    media_resolution: None,
    video_metadata: None,
    speech_metadata: None,
    data: PartData::FileData(json!({
        "fileUri": file.uri.as_deref().unwrap(),
        "mimeType": file.mime_type.as_deref().unwrap_or("application/pdf"),
    })),
};

let request = GenerateContentRequest::builder()
    .contents(vec![
        Content::builder()
            .role(Role::User)
            .add_part(file_part)
            .add_text_part("Summarize the key findings in three bullet points.")
            .build(),
    ])
    .build();

let response = client.generate_content(&request, "gemini-2.5-flash").await?;

// 4. Clean up when finished (files otherwise expire automatically after ~48 hours).
client.delete_file(&file_name).await?;
```

Other client methods:
- `client.upload_file_with_metadata(data, mime_type, &File::builder().name("my-id").display_name("...").build())`: upload with a custom resource ID (`files/my-id`).
- `client.get_file(name)`: fetch `File` metadata and check `state` / `error` / `video_metadata`.
- `client.list_files(page_size, page_token)`: paginated listing of uploaded/generated files (default page size 10, max 100).
- `client.delete_file(name)`: delete an uploaded file immediately.

See `examples/files.rs` for a runnable version.

---

## Batch generation (Batch API)

Process large volumes of content generation requests asynchronously at reduced cost via the Gemini Batch API.
Batches can be submitted with inlined requests (recommended for total payload sizes < 20MB) or by referencing
a JSONL file previously uploaded with the Files API:

```rust
use std::time::Duration;
use geologia::prelude::{
    BatchGenerateContentRequest, BatchState, Content, GeminiClient, GenerateContentBatch,
    GenerateContentRequest, InputConfig, Role,
};

let client = GeminiClient::new(std::env::var("GEMINI_API_KEY")?);

// 1. Prepare inline requests
let request1 = GenerateContentRequest::builder()
    .contents(vec![
        Content::builder()
            .role(Role::User)
            .add_text_part("Explain photosynthesis in one concise sentence.")
            .build(),
    ])
    .build();

let request2 = GenerateContentRequest::builder()
    .contents(vec![
        Content::builder()
            .role(Role::User)
            .add_text_part("Explain cellular respiration in one concise sentence.")
            .build(),
    ])
    .build();

let batch = BatchGenerateContentRequest::new(
    GenerateContentBatch::builder()
        .display_name("biology-comparison-batch")
        .input_config(InputConfig::from_inline(vec![request1, request2]))
        .build(),
);

// 2. Submit the batch via `POST models/{model}:batchGenerateContent`
let initial_op = client.batch_generate_content(&batch, "gemini-2.5-flash").await?;
let mut op = initial_op;
let batch_name = op
    .name
    .clone()
    .expect("server should assign a batch name");

// 3. Poll get_batch until complete (`BatchState::Succeeded`, `Failed`, or `Cancelled`)
while !op.is_done() {
    tokio::time::sleep(Duration::from_secs(5)).await;
    op = client.get_batch(&batch_name).await?;
}

// 4. Inspect results
if let Some(responses) = op.inlined_responses() {
    for resp in responses {
        if let Some(result) = resp.result() {
            println!("Output: {}", result.candidates[0].get_text().unwrap());
        }
    }
}

// 5. Clean up the batch resource
client.delete_batch(&batch_name).await?;
```

Other batch methods:
- `client.list_batches(page_size, page_token)`: paginated listing of batch operations.
- `client.cancel_batch(name)`: cancel an in-progress batch job.
- `client.delete_batch(name)`: delete a batch resource by name (`batches/abc-123`) or bare ID.

See `examples/batch.rs` for a runnable version.

---

## File Search stores (semantic retrieval & grounding)

Index documents for semantic retrieval and ground `generate_content` responses in private knowledge bases
via the Gemini File Search store API (`fileSearchStores`).

Documents can be directly uploaded with custom chunking parameters or imported from files previously
uploaded through the Files API. Indexing is performed asynchronously by Gemini.

```rust
use std::time::Duration;
use geologia::prelude::{
    ChunkingConfig, Content, DocumentState, FileSearch, FileSearchStore, GeminiClient,
    GenerateContentRequest, Tools,
};

let client = GeminiClient::new(std::env::var("GEMINI_API_KEY")?);

// 1. Create a File Search store
let store = client
    .create_file_search_store(
        &FileSearchStore::builder()
            .display_name("Knowledge Base")
            .build(),
    )
    .await?;
let store_name = store.name.expect("server assigns fileSearchStores/{id}");

// 2. Upload raw bytes directly (initiated as a resumable upload session)
let operation = client
    .upload_to_file_search_store(
        &store_name,
        b"Antigravity is an advanced agentic coding assistant.",
        Some("text/plain"),
        Some("overview.txt"),
    )
    .await?;
let op_name = operation.name.expect("server assigns operation name");

// 3. Poll the operation until indexing completes
let mut op = operation;
while !op.is_done() {
    tokio::time::sleep(Duration::from_secs(2)).await;
    op = client.get_file_search_operation(&op_name).await?;
}

// 4. Poll the document until ACTIVE (ready for semantic querying)
let doc_name = op.document_name().expect("operation response has document_name");
let mut doc = client.get_document(&doc_name).await?;
while doc.state != Some(DocumentState::Active) {
    if doc.state == Some(DocumentState::Failed) {
        return Err("Document processing failed".into());
    }
    tokio::time::sleep(Duration::from_secs(2)).await;
    doc = client.get_document(&doc_name).await?;
}

// 5. Query the model grounded in the File Search store
let request = GenerateContentRequest::builder()
    .contents(vec![
        Content::builder()
            .add_text_part("What is Antigravity?")
            .build(),
    ])
    .tools(vec![Tools {
        file_search: Some(FileSearch::from_store(&store_name)),
        ..Default::default()
    }])
    .build();

let response = client.generate_content(&request, "gemini-2.5-flash").await?;
println!("{}", response.candidates[0].get_text().unwrap());

// Grounding metadata is accessible on the candidate
if let Some(ref meta) = response.candidates[0].grounding_metadata {
    println!("Grounding metadata: {meta:?}");
}

// 6. Clean up: delete store and documents (force = true deletes non-empty stores)
client.delete_file_search_store(&store_name, true).await?;
```

Other File Search methods:
- `client.get_file_search_store(name)`: fetch store metadata and indexed document counts.
- `client.list_file_search_stores(page_size, page_token)`: paginated listing of owned stores.
- `client.upload_to_file_search_store_with_request(store, data, &request)`: upload with custom chunking config (`ChunkingConfig::white_space(max_tokens, overlap)`) or custom metadata.
- `client.import_file_to_file_search_store(store, &request)`: import a file previously uploaded via the Files API.
- `client.get_file_search_operation(name)`: check status of an upload or import operation.
- `client.get_document(name)`: inspect a document's status, chunking stats, and custom metadata.
- `client.delete_file_search_store(name, force)`: delete a store (set `force = true` if the store contains documents).

See `examples/file-search.rs` for a runnable version.

---

## Text embeddings

Embed a single input with `embed_content`, or embed multiple inputs in one round-trip with
`batch_embed_contents` (which automatically sets `model` on each inner request):

```rust
use geologia::prelude::{
    BatchEmbedContentsRequest, EmbedContentRequest, GeminiClient, TaskType,
};

// Single embedding
let query = EmbedContentRequest::builder()
    .text("How does Rust manage memory?")
    .task_type(TaskType::RetrievalQuery)
    .output_dimensionality(256)
    .build();

let response = client
    .embed_content(&query, "gemini-embedding-001")
    .await?;
// response.embedding.values: Vec<f32>

// Batch embeddings
let batch = BatchEmbedContentsRequest::builder()
    .add_request(
        EmbedContentRequest::builder()
            .text("Rust uses ownership and borrowing checked at compile time.")
            .task_type(TaskType::RetrievalDocument)
            .title("Rust Memory Model")
            .output_dimensionality(256)
            .build(),
    )
    .add_request(EmbedContentRequest::from_text("Another document to embed."))
    .build();

let batch_response = client
    .batch_embed_contents(&batch, "gemini-embedding-001")
    .await?;
// batch_response.embeddings: Vec<ContentEmbedding>
```

See `examples/embeddings.rs` for a runnable version.

---

## Listing and inspecting models

Discover which models are available for an API key, which methods they support
(`supported_generation_methods`), and their token limits and default generation parameters:

```rust
use geologia::prelude::GeminiClient;

let client = GeminiClient::new(std::env::var("GEMINI_API_KEY")?);

// Paginated listing (pass `None, None` for the default page size of 50)
let mut page_token: Option<String> = None;
loop {
    let page = client.list_models(Some(25), page_token.as_deref()).await?;
    for model in &page.models {
        println!(
            "{} ({:?}) — methods: {:?}",
            model.name, model.display_name, model.supported_generation_methods
        );
    }
    match page.next_page_token {
        Some(next) => page_token = Some(next),
        None => break,
    }
}

// Fetch a single model by ID or full `models/...` resource name
let model = client.get_model("gemini-2.5-flash").await?;
println!(
    "Input limit: {:?}, Output limit: {:?}, Thinking: {:?}",
    model.input_token_limit, model.output_token_limit, model.thinking
);
```

See `examples/models.rs` for a runnable version.

---

## Structured JSON output

Use `response_mime_type` and `response_json_schema` (or `response_schema`) in `GenerationConfig` to
enforce a JSON Schema:

```rust
use geologia::prelude::GenerationConfig;
use serde_json::json;

let config = GenerationConfig::builder()
    .response_mime_type("application/json")
    .response_json_schema(json!({
        "type": "object",
        "properties": {
            "name": { "type": "string" },
            "age": { "type": "integer" }
        },
        "required": ["name", "age"]
    }))
    .build();
```

---

## Error handling

All methods return `Result<_, geologia::Error>`. The error type covers HTTP errors, API-level
errors (with status codes and messages from Google), and serialization failures. Use standard
`?` propagation or match on the error for specific handling.

---

## Common model IDs

- `"gemini-3.1-pro-preview"` — most capable text model
- `"gemini-3.8-flash"` — fast and cost-efficient
- `"gemini-3.1-flash-lite"` — fastest and cheapest (used by the examples)
- `"imagen-4.0-generate-001"` — image generation
- `"gemini-embedding-001"` — text embeddings
