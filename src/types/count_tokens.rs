// Copyright 2026 Andre Cipriani Bandarra
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

use super::{Content, ModalityTokenCount};

/// Request body for the `countTokens` endpoint.
///
/// Use [`CountTokensRequest::builder`] for ergonomic construction.
///
/// See <https://ai.google.dev/api/tokens#method:-models.counttokens>.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CountTokensRequest {
    /// The content to count tokens for.
    pub contents: Content,
}

impl CountTokensRequest {
    /// Returns a new [`CountTokensRequestBuilder`].
    pub fn builder() -> CountTokensRequestBuilder {
        CountTokensRequestBuilder::default()
    }
}

/// Builder for [`CountTokensRequest`].
#[derive(Debug, Default)]
pub struct CountTokensRequestBuilder {
    contents: Content,
}

impl CountTokensRequestBuilder {
    /// Creates a builder pre-populated with a single text prompt.
    pub fn from_prompt(prompt: &str) -> Self {
        CountTokensRequestBuilder {
            contents: Content {
                parts: Some(vec![super::Part::from_text(prompt.to_string())]),
                ..Default::default()
            },
        }
    }

    /// Consumes the builder and returns the constructed [`CountTokensRequest`].
    pub fn build(self) -> CountTokensRequest {
        CountTokensRequest {
            contents: self.contents,
        }
    }
}

/// The raw response from the `countTokens` endpoint, which may be a success or an error.
///
/// Use [`into_result`](CountTokensResponse::into_result) to convert into a standard `Result`.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CountTokensResponse {
    Ok(CountTokensResponseResult),
    Error { error: super::VertexApiError },
}

impl CountTokensResponse {
    /// Converts this response into a `Result`, mapping the error variant to [`crate::error::Error`].
    pub fn into_result(self) -> Result<CountTokensResponseResult> {
        match self {
            CountTokensResponse::Ok(result) => Ok(result),
            CountTokensResponse::Error { error } => Err(Error::VertexError(error)),
        }
    }
}

/// A successful response from the `countTokens` endpoint.
///
/// See <https://ai.google.dev/api/tokens#response-body>.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CountTokensResponseResult {
    /// The total number of tokens in the input.
    pub total_tokens: i32,
    /// The total number of billable characters in the input. Only returned by Vertex AI.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_billable_characters: Option<u32>,
    /// The number of tokens in the cached part of the prompt.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cached_content_token_count: Option<u32>,
    /// The per-modality breakdown of tokens in the request input.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_tokens_details: Option<Vec<ModalityTokenCount>>,
    /// The per-modality breakdown of tokens in the cached content.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_tokens_details: Option<Vec<ModalityTokenCount>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Modality;

    #[test]
    fn parses_gemini_response() {
        let body = r#"{
            "totalTokens": 2,
            "systemInstructionsTokens": {},
            "contentTokens": [{ "partTokens": [2] }],
            "promptTokensDetails": [{ "modality": "TEXT", "tokenCount": 2 }]
        }"#;

        let result = serde_json::from_str::<CountTokensResponse>(body)
            .unwrap()
            .into_result()
            .unwrap();
        assert_eq!(result.total_tokens, 2);
        assert_eq!(result.total_billable_characters, None);
        assert_eq!(result.cached_content_token_count, None);
        let details = result.prompt_tokens_details.unwrap();
        assert_eq!(details.len(), 1);
        assert_eq!(details[0].modality, Modality::Text);
        assert_eq!(details[0].token_count, 2);
        assert!(result.cache_tokens_details.is_none());
    }

    #[test]
    fn parses_response_with_only_total_tokens() {
        let result: CountTokensResponseResult =
            serde_json::from_str(r#"{"totalTokens": 7}"#).unwrap();
        assert_eq!(result.total_tokens, 7);
        assert_eq!(result.total_billable_characters, None);
        assert!(result.prompt_tokens_details.is_none());
    }
}
