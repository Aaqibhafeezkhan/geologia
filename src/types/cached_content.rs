// Copyright 2026 Andre Cipriani Bandarra
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};

use super::{Content, ToolConfig, Tools};

/// An explicit context cache resource (`cachedContents`).
///
/// Context caching allows pre-computing and reusing large, stable prefixes (such as system
/// instructions, document collections, or tool declarations) across requests, reducing latency
/// and cost.
///
/// Use [`CachedContent::builder`] for ergonomic construction.
///
/// See <https://ai.google.dev/api/caching>.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CachedContent {
    /// The server-generated resource name, in the format `cachedContents/{id}`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// The name of the Gemini model to use for this cache (e.g. `models/gemini-1.5-flash`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// The user-visible display name for this cache.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    /// Optional developer instruction (system prompt) to cache.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_instruction: Option<Content>,
    /// The contents to cache.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contents: Option<Vec<Content>>,
    /// Tools (e.g. function declarations) available to the model.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tools>>,
    /// Tool configuration for the cache.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_config: Option<ToolConfig>,
    /// Time-to-live duration string (e.g. "300s"). Input only.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ttl: Option<String>,
    /// Timestamp when this cache expires (RFC 3339 format).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expire_time: Option<String>,
    /// Token usage metadata for this cache. Output only.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage_metadata: Option<CachedContentUsageMetadata>,
    /// Creation timestamp (RFC 3339 format). Output only.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub create_time: Option<String>,
    /// Last update timestamp (RFC 3339 format). Output only.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub update_time: Option<String>,
}

impl CachedContent {
    /// Returns a new [`CachedContentBuilder`].
    pub fn builder() -> CachedContentBuilder {
        CachedContentBuilder::default()
    }
}

/// Token usage metadata for a [`CachedContent`] resource.
///
/// See <https://ai.google.dev/api/caching#CachedContent>.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CachedContentUsageMetadata {
    /// Total number of tokens stored in the cache.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_token_count: Option<u32>,
}

/// Paginated list of [`CachedContent`] resources returned by [`GeminiClient::list_cached_contents`](crate::prelude::GeminiClient::list_cached_contents).
///
/// See <https://ai.google.dev/api/caching#response-body>.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListCachedContentsResponse {
    /// The list of cached content resources.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cached_contents: Vec<CachedContent>,
    /// A token to retrieve the next page of results, or `None` if there are no more pages.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_page_token: Option<String>,
}

/// Builder for [`CachedContent`].
#[derive(Clone, Debug, Default)]
pub struct CachedContentBuilder {
    cached_content: CachedContent,
}

impl CachedContentBuilder {
    /// Sets the server-generated resource name (e.g. `cachedContents/{id}`).
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.cached_content.name = Some(name.into());
        self
    }

    /// Sets the model name (e.g. `gemini-1.5-flash` or `models/gemini-1.5-flash`).
    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.cached_content.model = Some(model.into());
        self
    }

    /// Sets the user-visible display name.
    pub fn display_name(mut self, display_name: impl Into<String>) -> Self {
        self.cached_content.display_name = Some(display_name.into());
        self
    }

    /// Sets the developer instruction (system prompt).
    pub fn system_instruction(mut self, system_instruction: Content) -> Self {
        self.cached_content.system_instruction = Some(system_instruction);
        self
    }

    /// Sets the contents to cache.
    pub fn contents(mut self, contents: Vec<Content>) -> Self {
        self.cached_content.contents = Some(contents);
        self
    }

    /// Sets the tools available to the model.
    pub fn tools(mut self, tools: Vec<Tools>) -> Self {
        self.cached_content.tools = Some(tools);
        self
    }

    /// Sets the tool configuration.
    pub fn tool_config(mut self, tool_config: ToolConfig) -> Self {
        self.cached_content.tool_config = Some(tool_config);
        self
    }

    /// Sets the time-to-live duration string (e.g. "300s").
    pub fn ttl(mut self, ttl: impl Into<String>) -> Self {
        self.cached_content.ttl = Some(ttl.into());
        self
    }

    /// Sets the expiration timestamp (RFC 3339 format).
    pub fn expire_time(mut self, expire_time: impl Into<String>) -> Self {
        self.cached_content.expire_time = Some(expire_time.into());
        self
    }

    /// Sets the token usage metadata.
    pub fn usage_metadata(mut self, usage_metadata: CachedContentUsageMetadata) -> Self {
        self.cached_content.usage_metadata = Some(usage_metadata);
        self
    }

    /// Consumes the builder and returns the constructed [`CachedContent`].
    pub fn build(self) -> CachedContent {
        self.cached_content
    }
}

#[cfg(test)]
#[path = "cached_content_tests.rs"]
mod tests;
