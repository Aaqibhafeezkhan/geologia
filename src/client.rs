// Copyright 2026 Andre Cipriani Bandarra
// SPDX-License-Identifier: Apache-2.0

use crate::error::{Error as GeminiError, Result as GeminiResult};
use crate::network::event_source::{EventSource, ServerSentEvent};
use crate::prelude::*;

use tokio_stream::{Stream, StreamExt};
use tokio_util::codec::LinesCodecError;
use tracing::error;

/// Default base URL for the Gemini Developer API.
pub const DEFAULT_BASE_URL: &str = "https://generativelanguage.googleapis.com/v1beta";

/// Async client for the Google Gemini API.
///
/// Provides methods for content generation, streaming, token counting, text embeddings,
/// image prediction, and context caching. All requests are authenticated with an API key
/// passed at construction time.
///
/// Use [`GeminiClient::builder`] for advanced configuration, such as pointing at a mock
/// server or sharing a configured [`reqwest::Client`].
///
/// # Example
///
/// ```no_run
/// use geologia::prelude::*;
///
/// # async fn run() -> geologia::error::Result<()> {
/// let client = GeminiClient::new("YOUR_API_KEY".into());
/// let request = GenerateContentRequest::builder()
///     .contents(vec![Content::builder().add_text_part("Hi!").build()])
///     .build();
/// let response = client.generate_content(&request, "gemini-2.0-flash").await?;
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Debug)]
pub struct GeminiClient {
    client: reqwest::Client,
    api_key: String,
    base_url: String,
}

impl GeminiClient {
    /// Creates a new [`GeminiClient`] with the given API key.
    pub fn new(api_key: String) -> Self {
        Self::builder(api_key).build()
    }

    /// Creates a builder for configuring and constructing a [`GeminiClient`].
    pub fn builder(api_key: impl Into<String>) -> GeminiClientBuilder {
        GeminiClientBuilder::new(api_key)
    }

    /// Returns the configured base URL for this client.
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Returns a reference to the underlying [`reqwest::Client`].
    pub fn http_client(&self) -> &reqwest::Client {
        &self.client
    }

    /// Returns the API key used by this client.
    pub fn api_key(&self) -> &str {
        &self.api_key
    }

    /// Returns the response unchanged on a success status, otherwise reads the response
    /// body and maps it to a structured error.
    ///
    /// Unlike [`reqwest::Response::error_for_status`], this consumes the body so the
    /// provider's error detail (e.g. the Gemini `INVALID_ARGUMENT` message naming the
    /// offending field) is surfaced to the caller rather than discarded. When the body
    /// parses as a Gemini error envelope it becomes a [`GeminiError::VertexError`];
    /// otherwise the raw body is preserved in a [`GeminiError::ApiError`].
    async fn error_for_status(resp: reqwest::Response) -> GeminiResult<reqwest::Response> {
        let status = resp.status();
        if !status.is_client_error() && !status.is_server_error() {
            return Ok(resp);
        }
        let body = resp.text().await?;
        match serde_json::from_str::<VertexApiErrorResponse>(&body) {
            Ok(envelope) => Err(GeminiError::VertexError(envelope.error)),
            Err(_) => Err(GeminiError::ApiError { status, body }),
        }
    }

    /// Sends a content generation request and returns a stream of response chunks via SSE.
    ///
    /// Each item in the stream is a [`GenerateContentResponseResult`] containing one or more
    /// candidates (or [`PromptFeedback`] when the prompt is blocked). Useful for displaying
    /// incremental output as it is generated.
    ///
    /// SSE events without a `data:` field (such as comments and keep-alives) are skipped.
    /// Mid-stream transport or SSE decoding failures are yielded as
    /// `Err(`[`Error::EventSourceError`](crate::error::Error::EventSourceError)`)` items rather
    /// than silently terminating the stream.
    ///
    /// Note that a stream ending cleanly (`None` with no preceding `Err`) only indicates that the
    /// underlying HTTP body reached EOF between SSE events; it does not by itself guarantee that
    /// the model finished generating. Callers should inspect [`Candidate::finish_reason`] on the
    /// last chunk (or [`GenerateContentResponseResult::prompt_feedback`] if no candidates were
    /// returned) to confirm the response is complete.
    pub async fn stream_generate_content(
        &self,
        request: &GenerateContentRequest,
        model: &str,
    ) -> GeminiResult<impl Stream<Item = GeminiResult<GenerateContentResponseResult>>> {
        let endpoint_url =
            self.endpoint_url(&format!("models/{model}:streamGenerateContent?alt=sse"));
        let client = self.client.clone();
        let request = request.clone();
        let resp = client
            .post(&endpoint_url)
            .header("x-goog-api-key", &self.api_key)
            .json(&request)
            .send()
            .await?;
        Ok(Self::response_event_stream(
            Self::error_for_status(resp).await?,
        ))
    }

    fn response_event_stream(
        resp: reqwest::Response,
    ) -> impl Stream<Item = GeminiResult<GenerateContentResponseResult>> {
        resp.event_stream().filter_map(Self::parse_event)
    }

    fn parse_event(
        event_result: std::result::Result<ServerSentEvent, LinesCodecError>,
    ) -> Option<GeminiResult<GenerateContentResponseResult>> {
        let event = match event_result {
            Ok(event) => event,
            Err(err) => return Some(Err(err.into())),
        };
        let data = event.data?;

        Some(
            serde_json::from_str::<GenerateContentResponse>(&data)
                .map_err(Into::into)
                .and_then(|resp| resp.into_result()),
        )
    }

    /// Sends a content generation request and returns the complete response.
    ///
    /// For streaming responses, use [`stream_generate_content`](Self::stream_generate_content).
    pub async fn generate_content(
        &self,
        request: &GenerateContentRequest,
        model: &str,
    ) -> GeminiResult<GenerateContentResponseResult> {
        let endpoint_url = self.endpoint_url(&format!("models/{model}:generateContent"));
        let resp = self
            .client
            .post(&endpoint_url)
            .header("x-goog-api-key", &self.api_key)
            .json(&request)
            .send()
            .await?;
        let resp = Self::error_for_status(resp).await?;

        let txt_json = resp.text().await?;
        tracing::debug!("generate_content response: {:?}", txt_json);

        match serde_json::from_str::<GenerateContentResponse>(&txt_json) {
            Ok(response) => Ok(response.into_result()?),
            Err(e) => {
                tracing::error!("Failed to parse response: {} with error {}", txt_json, e);
                Err(e.into())
            }
        }
    }

    /// Generates an embedding vector for the given content.
    ///
    /// See <https://ai.google.dev/api/embeddings#method:-models.embedcontent>.
    pub async fn embed_content(
        &self,
        request: &EmbedContentRequest,
        model: &str,
    ) -> GeminiResult<EmbedContentResponse> {
        let model_id = model.strip_prefix("models/").unwrap_or(model);
        let endpoint_url = self.endpoint_url(&format!("models/{model_id}:embedContent"));
        let resp = self
            .client
            .post(&endpoint_url)
            .header("x-goog-api-key", &self.api_key)
            .json(request)
            .send()
            .await?;
        let resp = Self::error_for_status(resp).await?;

        let txt_json = resp.text().await?;
        tracing::debug!("embed_content response: {:?}", txt_json);

        match serde_json::from_str::<EmbedContentResponse>(&txt_json) {
            Ok(response) => Ok(response),
            Err(e) => {
                error!(response = txt_json, error = ?e, "Failed to parse response");
                Err(e.into())
            }
        }
    }

    /// Generates multiple embedding vectors in a single batch request.
    ///
    /// Sets `model` (as `models/{model}`) on each inner [`EmbedContentRequest`] in the batch
    /// as required by the API.
    ///
    /// See <https://ai.google.dev/api/embeddings#method:-models.batchembedcontents>.
    pub async fn batch_embed_contents(
        &self,
        request: &BatchEmbedContentsRequest,
        model: &str,
    ) -> GeminiResult<BatchEmbedContentsResponse> {
        let model_id = model.strip_prefix("models/").unwrap_or(model);
        let endpoint_url = self.endpoint_url(&format!("models/{model_id}:batchEmbedContents"));
        let resp = self
            .client
            .post(&endpoint_url)
            .header("x-goog-api-key", &self.api_key)
            .json(&request.with_model(model_id))
            .send()
            .await?;
        let resp = Self::error_for_status(resp).await?;

        let txt_json = resp.text().await?;
        tracing::debug!("batch_embed_contents response: {:?}", txt_json);

        match serde_json::from_str::<BatchEmbedContentsResponse>(&txt_json) {
            Ok(response) => Ok(response),
            Err(e) => {
                error!(response = txt_json, error = ?e, "Failed to parse response");
                Err(e.into())
            }
        }
    }

    /// Counts the number of tokens in the given content or `generateContent` request.
    ///
    /// When `request` carries a [`GenerateContentRequest`],
    /// its `model` is set from `model`.
    pub async fn count_tokens(
        &self,
        request: &CountTokensRequest,
        model: &str,
    ) -> GeminiResult<CountTokensResponseResult> {
        let endpoint_url = self.endpoint_url(&format!("models/{model}:countTokens"));
        let resp = self
            .client
            .post(&endpoint_url)
            .header("x-goog-api-key", &self.api_key)
            .json(&request.with_model(model))
            .send()
            .await?;
        let resp = Self::error_for_status(resp).await?;

        let txt_json = resp.text().await?;
        tracing::debug!("count_tokens response: {:?}", txt_json);

        match serde_json::from_str::<CountTokensResponse>(&txt_json) {
            Ok(response) => Ok(response.into_result()?),
            Err(e) => {
                error!(response = txt_json, error = ?e, "Failed to parse response");
                Err(e.into())
            }
        }
    }

    /// Generates images from a text prompt using an Imagen model.
    pub async fn predict_image(
        &self,
        request: &PredictImageRequest,
        model: &str,
    ) -> GeminiResult<PredictImageResponse> {
        let endpoint_url = self.endpoint_url(&format!("models/{model}:predict"));

        let resp = self
            .client
            .post(&endpoint_url)
            .header("x-goog-api-key", &self.api_key)
            .json(&request)
            .send()
            .await?;
        let resp = Self::error_for_status(resp).await?;

        let txt_json = resp.text().await?;

        match serde_json::from_str::<PredictImageResponse>(&txt_json) {
            Ok(response) => Ok(response),
            Err(e) => {
                error!(response = txt_json, error = ?e, "Failed to parse response");
                Err(e.into())
            }
        }
    }

    /// Creates a new explicit context cache resource.
    ///
    /// Context caching allows pre-computing and reusing large prefixes (e.g. documents, system
    /// instructions, or tool declarations) across requests.
    ///
    /// The `model` field on `request` is required and will be automatically prefixed with
    /// `models/` if omitted.
    ///
    /// See <https://ai.google.dev/api/caching#cachedContents.create>.
    pub async fn create_cached_content(
        &self,
        request: &CachedContent,
    ) -> GeminiResult<CachedContent> {
        let endpoint_url = self.endpoint_url("cachedContents");
        let mut body = request.clone();
        if let Some(ref model) = body.model
            && !model.starts_with("models/")
        {
            body.model = Some(format!("models/{model}"));
        }
        let resp = self
            .client
            .post(&endpoint_url)
            .header("x-goog-api-key", &self.api_key)
            .json(&body)
            .send()
            .await?;
        let resp = Self::error_for_status(resp).await?;

        let txt_json = resp.text().await?;
        tracing::debug!("create_cached_content response: {:?}", txt_json);

        match serde_json::from_str::<CachedContent>(&txt_json) {
            Ok(content) => Ok(content),
            Err(e) => {
                error!(response = txt_json, error = ?e, "Failed to parse response");
                Err(e.into())
            }
        }
    }

    /// Retrieves metadata for an existing cached content resource by name or ID.
    ///
    /// `name` can be either the full resource name (e.g. `cachedContents/abc123xyz`)
    /// or just the resource ID (e.g. `abc123xyz`).
    ///
    /// See <https://ai.google.dev/api/caching#cachedContents.get>.
    pub async fn get_cached_content(&self, name: &str) -> GeminiResult<CachedContent> {
        let endpoint_url = self.cached_content_resource_url(name);
        let resp = self
            .client
            .get(&endpoint_url)
            .header("x-goog-api-key", &self.api_key)
            .send()
            .await?;
        let resp = Self::error_for_status(resp).await?;

        let txt_json = resp.text().await?;
        tracing::debug!("get_cached_content response: {:?}", txt_json);

        match serde_json::from_str::<CachedContent>(&txt_json) {
            Ok(content) => Ok(content),
            Err(e) => {
                error!(response = txt_json, error = ?e, "Failed to parse response");
                Err(e.into())
            }
        }
    }

    /// Lists existing cached content resources, with optional pagination.
    ///
    /// Pass `page_size` (up to 50) and `page_token` to paginate through results.
    ///
    /// See <https://ai.google.dev/api/caching#cachedContents.list>.
    pub async fn list_cached_contents(
        &self,
        page_size: Option<u32>,
        page_token: Option<&str>,
    ) -> GeminiResult<ListCachedContentsResponse> {
        let mut endpoint_url = self.endpoint_url("cachedContents");
        let mut query_params = Vec::new();
        if let Some(size) = page_size {
            query_params.push(format!("pageSize={size}"));
        }
        if let Some(token) = page_token {
            query_params.push(format!("pageToken={token}"));
        }
        if !query_params.is_empty() {
            endpoint_url.push('?');
            endpoint_url.push_str(&query_params.join("&"));
        }

        let resp = self
            .client
            .get(&endpoint_url)
            .header("x-goog-api-key", &self.api_key)
            .send()
            .await?;
        let resp = Self::error_for_status(resp).await?;

        let txt_json = resp.text().await?;
        tracing::debug!("list_cached_contents response: {:?}", txt_json);

        match serde_json::from_str::<ListCachedContentsResponse>(&txt_json) {
            Ok(response) => Ok(response),
            Err(e) => {
                error!(response = txt_json, error = ?e, "Failed to parse response");
                Err(e.into())
            }
        }
    }

    /// Updates the expiration (`ttl` or `expire_time`) of an existing cached content resource.
    ///
    /// Other fields of a cached content resource are immutable.
    ///
    /// If `update_mask` is `None`, the field mask is automatically derived based on whichever
    /// of `request.ttl` and `request.expire_time` are set.
    ///
    /// See <https://ai.google.dev/api/caching#cachedContents.patch>.
    pub async fn update_cached_content(
        &self,
        name: &str,
        request: &CachedContent,
        update_mask: Option<&str>,
    ) -> GeminiResult<CachedContent> {
        let mut endpoint_url = self.cached_content_resource_url(name);
        let mask = update_mask.map(str::to_string).or_else(|| {
            let mut fields = Vec::new();
            if request.ttl.is_some() {
                fields.push("ttl");
            }
            if request.expire_time.is_some() {
                fields.push("expireTime");
            }
            if fields.is_empty() {
                None
            } else {
                Some(fields.join(","))
            }
        });
        if let Some(mask) = mask {
            endpoint_url.push_str(&format!("?updateMask={mask}"));
        }

        let resp = self
            .client
            .patch(&endpoint_url)
            .header("x-goog-api-key", &self.api_key)
            .json(request)
            .send()
            .await?;
        let resp = Self::error_for_status(resp).await?;

        let txt_json = resp.text().await?;
        tracing::debug!("update_cached_content response: {:?}", txt_json);

        match serde_json::from_str::<CachedContent>(&txt_json) {
            Ok(content) => Ok(content),
            Err(e) => {
                error!(response = txt_json, error = ?e, "Failed to parse response");
                Err(e.into())
            }
        }
    }

    /// Deletes an existing cached content resource by name or ID.
    ///
    /// `name` can be either the full resource name (e.g. `cachedContents/abc123xyz`)
    /// or just the resource ID (e.g. `abc123xyz`).
    ///
    /// See <https://ai.google.dev/api/caching#cachedContents.delete>.
    pub async fn delete_cached_content(&self, name: &str) -> GeminiResult<()> {
        let endpoint_url = self.cached_content_resource_url(name);
        let resp = self
            .client
            .delete(&endpoint_url)
            .header("x-goog-api-key", &self.api_key)
            .send()
            .await?;
        let _ = Self::error_for_status(resp).await?;
        Ok(())
    }

    fn cached_content_resource_url(&self, name: &str) -> String {
        let path = if name.starts_with("cachedContents/") {
            name.to_string()
        } else {
            format!("cachedContents/{name}")
        };
        self.endpoint_url(&path)
    }

    fn endpoint_url(&self, path: &str) -> String {
        let base = self.base_url.trim_end_matches('/');
        let path = path.trim_start_matches('/');
        format!("{base}/{path}")
    }
}

/// Builder for constructing a [`GeminiClient`] with custom configuration.
///
/// Created via [`GeminiClient::builder`].
///
/// # Example
///
/// ```no_run
/// use geologia::prelude::*;
///
/// let client = GeminiClient::builder("YOUR_API_KEY")
///     .base_url("https://my-proxy.example.com/v1beta")
///     .build();
/// ```
#[derive(Clone, Debug)]
pub struct GeminiClientBuilder {
    api_key: String,
    base_url: Option<String>,
    client: Option<reqwest::Client>,
}

impl GeminiClientBuilder {
    /// Creates a new builder for the given API key.
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            base_url: None,
            client: None,
        }
    }

    /// Sets the base URL for API requests.
    ///
    /// Defaults to [`DEFAULT_BASE_URL`].
    pub fn base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = Some(base_url.into());
        self
    }

    /// Sets a custom [`reqwest::Client`] for making HTTP requests.
    ///
    /// When omitted, a default client is created via [`reqwest::Client::new`].
    pub fn http_client(mut self, client: reqwest::Client) -> Self {
        self.client = Some(client);
        self
    }

    /// Consumes the builder and returns the configured [`GeminiClient`].
    pub fn build(self) -> GeminiClient {
        GeminiClient {
            client: self.client.unwrap_or_default(),
            api_key: self.api_key,
            base_url: self
                .base_url
                .unwrap_or_else(|| DEFAULT_BASE_URL.to_string()),
        }
    }
}

#[cfg(test)]
#[path = "client_tests.rs"]
mod tests;
