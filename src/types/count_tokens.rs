// Copyright 2026 Andre Cipriani Bandarra
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

use super::{Content, GenerateContentRequest, ModalityTokenCount};

/// Request body for the `countTokens` endpoint.
///
/// Either count plain `contents`, or count a full [`GenerateContentRequest`] (system
/// instruction, tools, multi-turn history and so on) via `generate_content_request`. The API
/// ignores `contents` when `generate_content_request` is set.
///
/// Use [`CountTokensRequest::builder`] for ergonomic construction.
///
/// See <https://ai.google.dev/api/tokens#request-body>.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CountTokensRequest {
    /// The content to count tokens for.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub contents: Vec<Content>,
    /// A full `generateContent` request to count tokens for.
    ///
    /// The API requires this inner request to name its model; [`GeminiClient::count_tokens`](crate::prelude::GeminiClient::count_tokens)
    /// fills it in from its `model` argument.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generate_content_request: Option<GenerateContentRequest>,
}

impl CountTokensRequest {
    /// Returns a new [`CountTokensRequestBuilder`].
    pub fn builder() -> CountTokensRequestBuilder {
        CountTokensRequestBuilder::default()
    }

    /// Returns the wire body for this request, with `model` (e.g. `gemini-3.1-flash-lite`)
    /// set on the inner `generateContentRequest` as the API requires.
    pub(crate) fn with_model<'a>(&'a self, model: &str) -> CountTokensRequestBody<'a> {
        CountTokensRequestBody {
            contents: &self.contents,
            generate_content_request: self.generate_content_request.as_ref().map(|request| {
                ModelGenerateContentRequest {
                    model: format!("models/{model}"),
                    request,
                }
            }),
        }
    }
}

/// Wire form of [`CountTokensRequest`], carrying the model name on the inner request.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CountTokensRequestBody<'a> {
    #[serde(skip_serializing_if = "<[Content]>::is_empty")]
    contents: &'a [Content],
    #[serde(skip_serializing_if = "Option::is_none")]
    generate_content_request: Option<ModelGenerateContentRequest<'a>>,
}

#[derive(Debug, Serialize)]
struct ModelGenerateContentRequest<'a> {
    model: String,
    #[serde(flatten)]
    request: &'a GenerateContentRequest,
}

/// Builder for [`CountTokensRequest`].
#[derive(Debug, Default)]
pub struct CountTokensRequestBuilder {
    request: CountTokensRequest,
}

impl CountTokensRequestBuilder {
    /// Creates a builder pre-populated with a single text prompt.
    pub fn from_prompt(prompt: &str) -> Self {
        CountTokensRequestBuilder::default().contents(vec![Content {
            parts: Some(vec![super::Part::from_text(prompt.to_string())]),
            ..Default::default()
        }])
    }

    /// Creates a builder that counts the tokens of a full [`GenerateContentRequest`],
    /// including its system instruction, tools and conversation history.
    pub fn from_generate_content_request(request: GenerateContentRequest) -> Self {
        CountTokensRequestBuilder::default().generate_content_request(request)
    }

    /// Sets the contents to count tokens for.
    pub fn contents(mut self, contents: Vec<Content>) -> Self {
        self.request.contents = contents;
        self
    }

    /// Sets the full `generateContent` request to count tokens for.
    pub fn generate_content_request(mut self, request: GenerateContentRequest) -> Self {
        self.request.generate_content_request = Some(request);
        self
    }

    /// Consumes the builder and returns the constructed [`CountTokensRequest`].
    pub fn build(self) -> CountTokensRequest {
        self.request
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
    use serde_json::json;

    use super::*;
    use crate::types::{FunctionDeclaration, Modality, Role, Tools};

    #[test]
    fn serializes_prompt_as_contents() {
        let request = CountTokensRequestBuilder::from_prompt("Hello").build();
        let body = serde_json::to_value(request.with_model("gemini-3.1-flash-lite")).unwrap();
        assert_eq!(
            body,
            json!({ "contents": [{ "parts": [{ "text": "Hello" }] }] })
        );
    }

    #[test]
    fn serializes_generate_content_request_with_model() {
        let generate_content_request = GenerateContentRequest::builder()
            .system_instruction(Content::builder().add_text_part("Be terse.").build())
            .contents(vec![
                Content::builder()
                    .role(Role::User)
                    .add_text_part("What is 2 + 2?")
                    .build(),
            ])
            .tools(vec![Tools {
                function_declarations: Some(vec![FunctionDeclaration {
                    name: String::from("add"),
                    description: String::from("Adds two numbers"),
                    behavior: None,
                    parameters: None,
                    parameters_json_schema: Some(json!({ "type": "object" })),
                    response: None,
                    response_json_schema: None,
                }]),
                ..Default::default()
            }])
            .build();
        let request =
            CountTokensRequestBuilder::from_generate_content_request(generate_content_request)
                .build();

        let body = serde_json::to_value(request.with_model("gemini-3.1-flash-lite")).unwrap();
        assert_eq!(
            body,
            json!({
                "generateContentRequest": {
                    "model": "models/gemini-3.1-flash-lite",
                    "contents": [{ "role": "user", "parts": [{ "text": "What is 2 + 2?" }] }],
                    "systemInstruction": { "parts": [{ "text": "Be terse." }] },
                    "tools": [{
                        "function_declarations": [{
                            "name": "add",
                            "description": "Adds two numbers",
                            "parametersJsonSchema": { "type": "object" }
                        }]
                    }]
                }
            })
        );
    }

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
