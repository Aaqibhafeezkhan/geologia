// Copyright 2026 Andre Cipriani Bandarra
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};

/// Information about a Generative Language [`Model`].
///
/// Returned by [`GeminiClient::get_model`](crate::prelude::GeminiClient::get_model) and
/// [`GeminiClient::list_models`](crate::prelude::GeminiClient::list_models).
///
/// See <https://ai.google.dev/api/models#Model>.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Model {
    /// The resource name of the `Model`, in the format `models/{model}`
    /// (e.g. `models/gemini-2.5-flash`).
    pub name: String,
    /// The name of the base model to pass to generation requests (e.g. `gemini-2.5-flash`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_model_id: Option<String>,
    /// The version number of the model (e.g. `"2.5"` or `"001"`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// The human-readable name of the model (e.g. `"Gemini 2.5 Flash"`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    /// A short description of the model.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Maximum number of input tokens allowed for this model.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_token_limit: Option<i32>,
    /// Maximum number of output tokens available for this model.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_token_limit: Option<i32>,
    /// The model's supported generation methods (e.g. `"generateContent"`, `"countTokens"`,
    /// `"embedContent"`, `"createCachedContent"`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub supported_generation_methods: Vec<String>,
    /// Default sampling temperature used by the backend when calling the model.
    /// Values can range over `[0.0, max_temperature]`, inclusive.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    /// The maximum temperature this model can use.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_temperature: Option<f32>,
    /// Default nucleus sampling (`top_p`) probability threshold used by the backend.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    /// Default top-k sampling count used by the backend. If `None`, the model does not use
    /// top-k sampling and `top_k` is not allowed as a generation parameter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_k: Option<i32>,
    /// Whether the model supports thinking (chain-of-thought reasoning).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking: Option<bool>,
}

/// Paginated list of [`Model`]s returned by
/// [`GeminiClient::list_models`](crate::prelude::GeminiClient::list_models).
///
/// See <https://ai.google.dev/api/models#response-body>.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ListModelsResponse {
    /// The returned models.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub models: Vec<Model>,
    /// A token, which can be sent as `page_token` to retrieve the next page.
    /// If `None`, there are no more pages.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_page_token: Option<String>,
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::prelude::{ListModelsResponse as PreludeListModelsResponse, Model as PreludeModel};

    use super::*;

    #[test]
    fn parses_single_model_with_all_fields_and_round_trips() -> serde_json::Result<()> {
        let payload = json!({
            "name": "models/gemini-2.5-flash",
            "baseModelId": "gemini-2.5-flash",
            "version": "001",
            "displayName": "Gemini 2.5 Flash",
            "description": "Mid-size multimodal model that supports up to 1 million tokens.",
            "inputTokenLimit": 1_048_576,
            "outputTokenLimit": 65_536,
            "supportedGenerationMethods": [
                "generateContent",
                "countTokens",
                "createCachedContent",
                "batchGenerateContent"
            ],
            "temperature": 1.0,
            "maxTemperature": 2.0,
            "topP": 0.95,
            "topK": 64,
            "thinking": true
        });

        let model: PreludeModel = serde_json::from_value(payload.clone())?;

        assert_eq!(model.name, "models/gemini-2.5-flash");
        assert_eq!(model.base_model_id.as_deref(), Some("gemini-2.5-flash"));
        assert_eq!(model.version.as_deref(), Some("001"));
        assert_eq!(model.display_name.as_deref(), Some("Gemini 2.5 Flash"));
        assert_eq!(
            model.description.as_deref(),
            Some("Mid-size multimodal model that supports up to 1 million tokens.")
        );
        assert_eq!(model.input_token_limit, Some(1_048_576));
        assert_eq!(model.output_token_limit, Some(65_536));
        assert_eq!(
            model.supported_generation_methods,
            vec![
                "generateContent",
                "countTokens",
                "createCachedContent",
                "batchGenerateContent",
            ]
        );
        assert_eq!(model.temperature, Some(1.0));
        assert_eq!(model.max_temperature, Some(2.0));
        assert_eq!(model.top_p, Some(0.95));
        assert_eq!(model.top_k, Some(64));
        assert_eq!(model.thinking, Some(true));

        let serialized = serde_json::to_string(&model)?;
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&serialized)?,
            payload
        );
        assert_eq!(serde_json::from_str::<PreludeModel>(&serialized)?, model);
        Ok(())
    }

    #[test]
    fn parses_minimal_model_and_omits_unset_fields_on_serialize() -> serde_json::Result<()> {
        let payload = json!({
            "name": "models/gemini-embedding-001"
        });

        let model: Model = serde_json::from_value(payload.clone())?;

        assert_eq!(model.name, "models/gemini-embedding-001");
        assert!(model.base_model_id.is_none());
        assert!(model.version.is_none());
        assert!(model.display_name.is_none());
        assert!(model.description.is_none());
        assert!(model.input_token_limit.is_none());
        assert!(model.output_token_limit.is_none());
        assert!(model.supported_generation_methods.is_empty());
        assert!(model.temperature.is_none());
        assert!(model.max_temperature.is_none());
        assert!(model.top_p.is_none());
        assert!(model.top_k.is_none());
        assert!(model.thinking.is_none());

        assert_eq!(serde_json::to_value(&model)?, payload);
        Ok(())
    }

    #[test]
    fn rejects_malformed_model_payloads() {
        // Missing required `name`.
        assert!(serde_json::from_value::<Model>(json!({})).is_err());

        // Invalid field types.
        assert!(
            serde_json::from_value::<Model>(json!({
                "name": "models/gemini-2.5-flash",
                "inputTokenLimit": "not-an-int"
            }))
            .is_err()
        );
        assert!(
            serde_json::from_value::<Model>(json!({
                "name": "models/gemini-2.5-flash",
                "thinking": "yes"
            }))
            .is_err()
        );
    }

    #[test]
    fn parses_list_models_response_with_next_page_token() -> serde_json::Result<()> {
        let payload = json!({
            "models": [
                {
                    "name": "models/gemini-2.5-flash",
                    "version": "001",
                    "displayName": "Gemini 2.5 Flash",
                    "inputTokenLimit": 1_048_576,
                    "outputTokenLimit": 65_536,
                    "supportedGenerationMethods": ["generateContent", "countTokens"],
                    "temperature": 1.0,
                    "maxTemperature": 2.0,
                    "topP": 0.95,
                    "topK": 64,
                    "thinking": true
                },
                {
                    "name": "models/gemini-embedding-001",
                    "version": "001",
                    "displayName": "Gemini Embedding 001",
                    "description": "Obtain a distributed representation of a text.",
                    "inputTokenLimit": 2048,
                    "outputTokenLimit": 1,
                    "supportedGenerationMethods": ["embedContent", "countTextTokens"]
                }
            ],
            "nextPageToken": "ChZtb2RlbHMvZ2VtaW5pLWVtYmVkZGluZw=="
        });

        let response: PreludeListModelsResponse = serde_json::from_value(payload.clone())?;

        assert_eq!(response.models.len(), 2);
        assert_eq!(response.models[0].name, "models/gemini-2.5-flash");
        assert_eq!(response.models[0].thinking, Some(true));
        assert_eq!(response.models[1].name, "models/gemini-embedding-001");
        assert_eq!(response.models[1].thinking, None);
        assert_eq!(
            response.models[1].supported_generation_methods,
            vec!["embedContent", "countTextTokens"]
        );
        assert_eq!(
            response.next_page_token.as_deref(),
            Some("ChZtb2RlbHMvZ2VtaW5pLWVtYmVkZGluZw==")
        );

        let serialized = serde_json::to_string(&response)?;
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&serialized)?,
            payload
        );
        assert_eq!(
            serde_json::from_str::<PreludeListModelsResponse>(&serialized)?,
            response
        );
        Ok(())
    }

    #[test]
    fn parses_empty_list_models_response() -> serde_json::Result<()> {
        let response: ListModelsResponse = serde_json::from_value(json!({}))?;

        assert!(response.models.is_empty());
        assert!(response.next_page_token.is_none());
        assert_eq!(serde_json::to_value(&response)?, json!({}));
        Ok(())
    }
}
