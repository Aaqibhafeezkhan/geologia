// Copyright 2026 Andre Cipriani Bandarra
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{Content, VertexApiError};
use crate::error::Result;

/// Request body for the `generateContent` and `streamGenerateContent` endpoints.
///
/// Use [`GenerateContentRequest::builder`] for ergonomic construction.
///
/// See <https://ai.google.dev/api/generate-content#request-body>.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerateContentRequest {
    pub contents: Vec<Content>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generation_config: Option<GenerationConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tools>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub safety_settings: Option<Vec<SafetySetting>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_instruction: Option<Content>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_config: Option<ToolConfig>,
    /// Name of cached content to use as context, in the form `cachedContents/{cachedContent}`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cached_content: Option<String>,
    /// The service tier used to serve the request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_tier: Option<ServiceTier>,
    /// Whether the request and response may be stored by the API.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub store: Option<bool>,
    /// User-defined metadata labels attached to the request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<HashMap<String, String>>,
    /// A base64-encoded token from a previous [`Candidate::continuation_token`], used to resume
    /// generation that stopped with `finishReason: CONTINUATION`.
    ///
    /// When this is set, `contents` must identically replicate the prompt and conversation
    /// history from the initial request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub continuation_token: Option<String>,
}

impl GenerateContentRequest {
    /// Returns a new [`GenerateContentRequestBuilder`].
    pub fn builder() -> GenerateContentRequestBuilder {
        GenerateContentRequestBuilder::new()
    }
}

/// Builder for [`GenerateContentRequest`].
#[derive(Debug)]
pub struct GenerateContentRequestBuilder {
    request: GenerateContentRequest,
}

impl GenerateContentRequestBuilder {
    fn new() -> Self {
        GenerateContentRequestBuilder {
            request: GenerateContentRequest::default(),
        }
    }

    /// Sets the conversation contents.
    pub fn contents(mut self, contents: Vec<Content>) -> Self {
        self.request.contents = contents;
        self
    }

    /// Sets the generation configuration.
    pub fn generation_config(mut self, generation_config: GenerationConfig) -> Self {
        self.request.generation_config = Some(generation_config);
        self
    }

    /// Sets the tools available to the model (e.g. function calling, Google Search).
    pub fn tools(mut self, tools: Vec<Tools>) -> Self {
        self.request.tools = Some(tools);
        self
    }

    /// Sets the safety filter settings.
    pub fn safety_settings(mut self, safety_settings: Vec<SafetySetting>) -> Self {
        self.request.safety_settings = Some(safety_settings);
        self
    }

    /// Sets a system instruction to guide the model's behavior.
    pub fn system_instruction(mut self, system_instruction: Content) -> Self {
        self.request.system_instruction = Some(system_instruction);
        self
    }

    /// Sets the tool configuration (e.g. function calling mode and allowed functions).
    pub fn tool_config(mut self, tool_config: ToolConfig) -> Self {
        self.request.tool_config = Some(tool_config);
        self
    }

    /// Sets the cached content to use as context (`cachedContents/{cachedContent}`).
    pub fn cached_content<T: Into<String>>(mut self, cached_content: T) -> Self {
        self.request.cached_content = Some(cached_content.into());
        self
    }

    /// Sets the service tier used to serve the request.
    pub fn service_tier(mut self, service_tier: ServiceTier) -> Self {
        self.request.service_tier = Some(service_tier);
        self
    }

    /// Sets whether the request and response may be stored by the API.
    pub fn store(mut self, store: bool) -> Self {
        self.request.store = Some(store);
        self
    }

    /// Sets user-defined metadata labels for the request.
    pub fn labels(mut self, labels: HashMap<String, String>) -> Self {
        self.request.labels = Some(labels);
        self
    }

    /// Sets a continuation token to resume generation from a previous candidate.
    ///
    /// `contents` must identically replicate the prompt and conversation history from the
    /// initial request. See [`GenerateContentRequest::continuation_token`].
    pub fn continuation_token<T: Into<String>>(mut self, continuation_token: T) -> Self {
        self.request.continuation_token = Some(continuation_token.into());
        self
    }

    /// Consumes the builder and returns the constructed [`GenerateContentRequest`].
    pub fn build(self) -> GenerateContentRequest {
        self.request
    }
}

/// A set of tool declarations the model may use during generation.
///
/// See <https://ai.google.dev/api/caching#Tool>.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Tools {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function_declarations: Option<Vec<FunctionDeclaration>>,

    #[serde(rename = "googleSearchRetrieval")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub google_search_retrieval: Option<GoogleSearchRetrieval>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub google_search: Option<GoogleSearch>,

    #[serde(rename = "codeExecution")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code_execution: Option<CodeExecution>,

    #[serde(rename = "urlContext")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url_context: Option<UrlContext>,

    /// File search tool configuration. See <https://ai.google.dev/api/caching#FileSearch>.
    #[serde(rename = "fileSearch")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_search: Option<Value>,

    /// Google Maps grounding tool configuration. See <https://ai.google.dev/api/caching#GoogleMaps>.
    #[serde(rename = "googleMaps")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub google_maps: Option<Value>,

    /// Computer use tool configuration. See <https://ai.google.dev/api/caching#ComputerUse>.
    #[serde(rename = "computerUse")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub computer_use: Option<Value>,

    /// MCP servers the model may call. See <https://ai.google.dev/api/caching#McpServer>.
    #[serde(rename = "mcpServers")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mcp_servers: Option<Vec<Value>>,
}

/// Enables the Google Search grounding tool.
///
/// Powered by Google Search to ground model responses in real-time information.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GoogleSearch {
    /// The set of search types to enable. If not set, web search is enabled by default.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search_types: Option<SearchTypes>,
    /// Filter search results to a specific time range.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_range_filter: Option<Interval>,
}

/// Different types of search that can be enabled on the [`GoogleSearch`] tool.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SearchTypes {
    /// Enables web search. Only text results are returned.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub web_search: Option<WebSearch>,
    /// Enables image search. Image bytes are returned.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_search: Option<ImageSearch>,
}

/// Standard web search configuration.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct WebSearch {}

/// Image search configuration.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct ImageSearch {}

/// Represents a time interval for filtering search results.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Interval {
    /// Inclusive start of the interval (RFC 3339 / ISO 8601 string).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_time: Option<String>,
    /// Exclusive end of the interval (RFC 3339 / ISO 8601 string).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_time: Option<String>,
}

impl Interval {
    /// Creates a new [`Interval`] with inclusive start and exclusive end timestamps.
    pub fn new(start_time: impl Into<String>, end_time: impl Into<String>) -> Self {
        Self {
            start_time: Some(start_time.into()),
            end_time: Some(end_time.into()),
        }
    }
}

/// Enables the code execution tool (no configuration required).
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CodeExecution {}

/// Enables the URL context tool (no configuration required).
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct UrlContext {}

/// Configuration for dynamic retrieval in Google Search grounding.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DynamicRetrievalConfig {
    /// The retrieval mode (e.g. `"MODE_DYNAMIC"`).
    pub mode: String,
    /// The threshold for triggering retrieval. Defaults to `0.7`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dynamic_threshold: Option<f32>,
}

impl Default for DynamicRetrievalConfig {
    fn default() -> Self {
        Self {
            mode: "MODE_DYNAMIC".to_string(),
            dynamic_threshold: Some(0.7),
        }
    }
}

/// Google Search retrieval tool with dynamic retrieval configuration.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GoogleSearchRetrieval {
    /// Configuration controlling when retrieval is triggered.
    pub dynamic_retrieval_config: DynamicRetrievalConfig,
}

/// Configuration shared by all tools declared in a request.
///
/// See <https://ai.google.dev/api/caching#ToolConfig>.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ToolConfig {
    /// Controls how the model calls declared functions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function_calling_config: Option<FunctionCallingConfig>,
    /// Configuration for retrieval tools (e.g. the user's location for grounding).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retrieval_config: Option<RetrievalConfig>,
    /// Whether server-side tool invocations are included in the response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_server_side_tool_invocations: Option<bool>,
}

/// Configuration for function calling behavior.
///
/// See <https://ai.google.dev/api/caching#FunctionCallingConfig>.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FunctionCallingConfig {
    /// The function calling mode. The API defaults to [`FunctionCallingMode::Auto`] when unset.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<FunctionCallingMode>,
    /// Restricts which declared functions the model may call.
    ///
    /// Should only be set when `mode` is [`FunctionCallingMode::Any`] or
    /// [`FunctionCallingMode::Validated`].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_function_names: Option<Vec<String>>,
}

/// How the model may call declared functions.
///
/// See <https://ai.google.dev/api/caching#Mode_1>.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FunctionCallingMode {
    /// Unspecified mode; do not use.
    ModeUnspecified,
    /// The model decides whether to call a function or respond with natural language.
    Auto,
    /// The model always calls a function.
    Any,
    /// The model never calls a function.
    None,
    /// The model may call a function or respond with natural language; function calls are
    /// validated against the declared schemas.
    Validated,
}

/// Configuration for retrieval tools.
///
/// See <https://ai.google.dev/api/caching#RetrievalConfig>.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RetrievalConfig {
    /// The location of the user.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lat_lng: Option<LatLng>,
    /// The language code of the user (e.g. `"en-US"`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language_code: Option<String>,
}

/// A latitude/longitude pair, in degrees.
///
/// See <https://ai.google.dev/api/caching#LatLng>.
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct LatLng {
    pub latitude: f64,
    pub longitude: f64,
}

/// Parameters that control how the model generates content.
///
/// Use [`GenerationConfig::builder`] for ergonomic construction.
///
/// See <https://ai.google.dev/api/generate-content#v1beta.GenerationConfig>.
#[derive(Clone, Debug, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GenerationConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_output_tokens: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_k: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_sequences: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub candidate_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_mime_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_schema: Option<Value>,
    /// Output schema of the generated response as standard JSON Schema (alternative to
    /// [`response_schema`](Self::response_schema)).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_json_schema: Option<Value>,
    /// Output response format configuration (`ResponseFormatConfig`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<Value>,
    /// Seed used in decoding for reproducibility when set.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<i32>,
    /// Penalty applied to the next token's logprobs if the token has already been seen.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presence_penalty: Option<f32>,
    /// Penalty applied to the next token's logprobs proportional to how often it has been seen.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency_penalty: Option<f32>,
    /// If true, export the logprobs results in the response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_logprobs: Option<bool>,
    /// Number of top logprobs to return at each decoding step (only valid when
    /// [`response_logprobs`](Self::response_logprobs) is `Some(true)`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logprobs: Option<i32>,
    /// Enables enhanced civic answers.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_enhanced_civic_answers: Option<bool>,
    /// Requested response modalities (supports [`Modality::Unspecified`], [`Modality::Text`],
    /// [`Modality::Image`], and [`Modality::Audio`]).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_modalities: Option<Vec<Modality>>,
    /// Media resolution for input media.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media_resolution: Option<MediaResolution>,
    /// Speech generation configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speech_config: Option<SpeechConfig>,
    /// Configuration for thinking features.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking_config: Option<ThinkingConfig>,
    /// Image generation configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_config: Option<ImageConfig>,
    /// Enables affective dialog so the model adapts to emotional tone.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_affective_dialog: Option<bool>,
    /// Audio transcription configuration (`AudioTranscriptionConfig`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio_transcription_config: Option<Value>,
    /// Translation configuration (`TranslationConfig`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub translation_config: Option<Value>,
}

impl GenerationConfig {
    /// Returns a new [`GenerationConfigBuilder`].
    pub fn builder() -> GenerationConfigBuilder {
        GenerationConfigBuilder::new()
    }
}

/// Builder for [`GenerationConfig`].
#[derive(Clone, Debug, Default)]
pub struct GenerationConfigBuilder {
    generation_config: GenerationConfig,
}

impl GenerationConfigBuilder {
    fn new() -> Self {
        Self {
            generation_config: Default::default(),
        }
    }

    pub fn max_output_tokens<T: Into<i32>>(mut self, max_output_tokens: T) -> Self {
        self.generation_config.max_output_tokens = Some(max_output_tokens.into());
        self
    }

    pub fn temperature<T: Into<f32>>(mut self, temperature: T) -> Self {
        self.generation_config.temperature = Some(temperature.into());
        self
    }

    pub fn top_p<T: Into<f32>>(mut self, top_p: T) -> Self {
        self.generation_config.top_p = Some(top_p.into());
        self
    }

    pub fn top_k<T: Into<i32>>(mut self, top_k: T) -> Self {
        self.generation_config.top_k = Some(top_k.into());
        self
    }

    pub fn stop_sequences<T: Into<Vec<String>>>(mut self, stop_sequences: T) -> Self {
        self.generation_config.stop_sequences = Some(stop_sequences.into());
        self
    }

    pub fn candidate_count<T: Into<u32>>(mut self, candidate_count: T) -> Self {
        self.generation_config.candidate_count = Some(candidate_count.into());
        self
    }

    pub fn response_mime_type<T: Into<String>>(mut self, response_mime_type: T) -> Self {
        self.generation_config.response_mime_type = Some(response_mime_type.into());
        self
    }

    pub fn response_schema<T: Into<Value>>(mut self, response_schema: T) -> Self {
        self.generation_config.response_schema = Some(response_schema.into());
        self
    }

    /// Sets the output JSON Schema (alternative to [`response_schema`](Self::response_schema)).
    pub fn response_json_schema<T: Into<Value>>(mut self, response_json_schema: T) -> Self {
        self.generation_config.response_json_schema = Some(response_json_schema.into());
        self
    }

    /// Sets the response format configuration (`ResponseFormatConfig`).
    pub fn response_format<T: Into<Value>>(mut self, response_format: T) -> Self {
        self.generation_config.response_format = Some(response_format.into());
        self
    }

    /// Sets the decoding seed.
    pub fn seed<T: Into<i32>>(mut self, seed: T) -> Self {
        self.generation_config.seed = Some(seed.into());
        self
    }

    /// Sets the presence penalty applied to already-seen tokens.
    pub fn presence_penalty(mut self, presence_penalty: f32) -> Self {
        self.generation_config.presence_penalty = Some(presence_penalty);
        self
    }

    /// Sets the frequency penalty applied proportionally to token occurrence count.
    pub fn frequency_penalty(mut self, frequency_penalty: f32) -> Self {
        self.generation_config.frequency_penalty = Some(frequency_penalty);
        self
    }

    /// Sets whether logprobs are included in the response.
    pub fn response_logprobs(mut self, response_logprobs: bool) -> Self {
        self.generation_config.response_logprobs = Some(response_logprobs);
        self
    }

    /// Sets the number of top logprobs to return at each decoding step.
    pub fn logprobs<T: Into<i32>>(mut self, logprobs: T) -> Self {
        self.generation_config.logprobs = Some(logprobs.into());
        self
    }

    /// Sets whether enhanced civic answers are enabled.
    pub fn enable_enhanced_civic_answers(mut self, enable_enhanced_civic_answers: bool) -> Self {
        self.generation_config.enable_enhanced_civic_answers = Some(enable_enhanced_civic_answers);
        self
    }

    /// Sets the requested response modalities.
    pub fn response_modalities<T: Into<Vec<Modality>>>(mut self, response_modalities: T) -> Self {
        self.generation_config.response_modalities = Some(response_modalities.into());
        self
    }

    /// Sets the media resolution for input media.
    pub fn media_resolution(mut self, media_resolution: MediaResolution) -> Self {
        self.generation_config.media_resolution = Some(media_resolution);
        self
    }

    /// Sets the speech generation configuration.
    pub fn speech_config(mut self, speech_config: SpeechConfig) -> Self {
        self.generation_config.speech_config = Some(speech_config);
        self
    }

    pub fn thinking_config(mut self, thinking_config: ThinkingConfig) -> Self {
        self.generation_config.thinking_config = Some(thinking_config);
        self
    }

    /// Sets the image generation configuration.
    pub fn image_config(mut self, image_config: ImageConfig) -> Self {
        self.generation_config.image_config = Some(image_config);
        self
    }

    /// Sets whether affective dialog is enabled.
    pub fn enable_affective_dialog(mut self, enable_affective_dialog: bool) -> Self {
        self.generation_config.enable_affective_dialog = Some(enable_affective_dialog);
        self
    }

    /// Sets the audio transcription configuration (`AudioTranscriptionConfig`).
    pub fn audio_transcription_config<T: Into<Value>>(
        mut self,
        audio_transcription_config: T,
    ) -> Self {
        self.generation_config.audio_transcription_config = Some(audio_transcription_config.into());
        self
    }

    /// Sets the translation configuration (`TranslationConfig`).
    pub fn translation_config<T: Into<Value>>(mut self, translation_config: T) -> Self {
        self.generation_config.translation_config = Some(translation_config.into());
        self
    }

    /// Consumes the builder and returns the constructed [`GenerationConfig`].
    pub fn build(self) -> GenerationConfig {
        self.generation_config
    }
}

/// Media resolution for input media in a generation request.
///
/// See <https://ai.google.dev/api/generate-content#MediaResolution>.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum MediaResolution {
    #[serde(rename = "MEDIA_RESOLUTION_UNSPECIFIED")]
    Unspecified,
    #[serde(rename = "MEDIA_RESOLUTION_LOW")]
    Low,
    #[serde(rename = "MEDIA_RESOLUTION_MEDIUM")]
    Medium,
    #[serde(rename = "MEDIA_RESOLUTION_HIGH")]
    High,
}

/// Speech generation configuration.
///
/// See <https://ai.google.dev/api/generate-content#SpeechConfig>.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SpeechConfig {
    /// Configuration for single-voice output (`VoiceConfig`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub voice_config: Option<Value>,
    /// Configuration for multi-speaker output (`MultiSpeakerVoiceConfig`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub multi_speaker_voice_config: Option<Value>,
    /// Language code (in BCP 47 format, e.g. `"en-US"`) for speech synthesis.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language_code: Option<String>,
}

/// Configuration for image generation outputs.
///
/// See <https://ai.google.dev/api/generate-content#ImageConfig>.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ImageConfig {
    /// Aspect ratio of generated images (e.g. `"1:1"`, `"16:9"`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aspect_ratio: Option<String>,
    /// Size of generated images (e.g. `"1K"`, `"2K"`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_size: Option<String>,
}

/// Configuration for the model's "thinking" (chain-of-thought) behavior.
///
/// Use [`ThinkingConfig::builder`] for ergonomic construction.
///
/// See <https://ai.google.dev/api/generate-content#ThinkingConfig>.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ThinkingConfig {
    /// Whether to include thoughts in the response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_thoughts: Option<bool>,
    /// The number of thought tokens that the model should generate.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking_budget: Option<i32>,
    /// The level of thinking effort the model should use.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking_level: Option<ThinkingLevel>,
}

impl ThinkingConfig {
    /// Returns a new [`ThinkingConfigBuilder`].
    pub fn builder() -> ThinkingConfigBuilder {
        ThinkingConfigBuilder::default()
    }
}

/// Builder for [`ThinkingConfig`].
#[derive(Clone, Debug, Default)]
pub struct ThinkingConfigBuilder {
    thinking_config: ThinkingConfig,
}

impl ThinkingConfigBuilder {
    /// Sets whether thoughts are included in the response.
    pub fn include_thoughts(mut self, include_thoughts: bool) -> Self {
        self.thinking_config.include_thoughts = Some(include_thoughts);
        self
    }

    /// Sets the number of thought tokens the model should generate.
    pub fn thinking_budget<T: Into<i32>>(mut self, thinking_budget: T) -> Self {
        self.thinking_config.thinking_budget = Some(thinking_budget.into());
        self
    }

    /// Sets the [`ThinkingLevel`] for the model.
    pub fn thinking_level(mut self, thinking_level: ThinkingLevel) -> Self {
        self.thinking_config.thinking_level = Some(thinking_level);
        self
    }

    /// Consumes the builder and returns the constructed [`ThinkingConfig`].
    pub fn build(self) -> ThinkingConfig {
        self.thinking_config
    }
}

/// The level of thinking effort the model should use.
///
/// See <https://ai.google.dev/api/generate-content#ThinkingLevel>.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ThinkingLevel {
    ThinkingLevelUnspecified,
    Minimal,
    Low,
    Medium,
    High,
}

/// A safety filter configuration that controls blocking thresholds for harmful content.
///
/// See <https://ai.google.dev/api/generate-content#safetysetting>.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SafetySetting {
    pub category: HarmCategory,
    pub threshold: HarmBlockThreshold,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method: Option<HarmBlockMethod>,
}

/// Categories of potentially harmful content.
///
/// See <https://ai.google.dev/api/generate-content#harmcategory>.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum HarmCategory {
    #[serde(rename = "HARM_CATEGORY_UNSPECIFIED")]
    Unspecified,
    #[serde(rename = "HARM_CATEGORY_HATE_SPEECH")]
    HateSpeech,
    #[serde(rename = "HARM_CATEGORY_DANGEROUS_CONTENT")]
    DangerousContent,
    #[serde(rename = "HARM_CATEGORY_HARASSMENT")]
    Harassment,
    #[serde(rename = "HARM_CATEGORY_SEXUALLY_EXPLICIT")]
    SexuallyExplicit,
    /// Deprecated by the API.
    #[serde(rename = "HARM_CATEGORY_CIVIC_INTEGRITY")]
    CivicIntegrity,
    #[serde(rename = "HARM_CATEGORY_JAILBREAK")]
    Jailbreak,
}

/// The threshold at which harmful content is blocked.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum HarmBlockThreshold {
    #[serde(rename = "HARM_BLOCK_THRESHOLD_UNSPECIFIED")]
    Unspecified,
    #[serde(rename = "BLOCK_LOW_AND_ABOVE")]
    BlockLowAndAbove,
    #[serde(rename = "BLOCK_MEDIUM_AND_ABOVE")]
    BlockMediumAndAbove,
    #[serde(rename = "BLOCK_ONLY_HIGH")]
    BlockOnlyHigh,
    #[serde(rename = "BLOCK_NONE")]
    BlockNone,
}

/// The method used to evaluate harm (severity-based or probability-based).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum HarmBlockMethod {
    #[serde(rename = "HARM_BLOCK_METHOD_UNSPECIFIED")]
    Unspecified, // HARM_BLOCK_METHOD_UNSPECIFIED
    #[serde(rename = "SEVERITY")]
    Severity, // SEVERITY
    #[serde(rename = "PROBABILITY")]
    Probability, // PROBABILITY
}

/// A single candidate response generated by the model.
///
/// See <https://ai.google.dev/api/generate-content#candidate>.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Candidate {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<Content>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub citation_metadata: Option<CitationMetadata>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub safety_ratings: Option<Vec<SafetyRating>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finish_reason: Option<String>,
    pub index: u32,
    /// Details on why the model stopped generating.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finish_message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avg_logprobs: Option<f64>,
    /// See <https://ai.google.dev/api/generate-content#LogprobsResult>.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logprobs_result: Option<Value>,
    /// See <https://ai.google.dev/api/generate-content#GroundingMetadata>.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grounding_metadata: Option<Value>,
    /// See <https://ai.google.dev/api/generate-content#UrlContextMetadata>.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url_context_metadata: Option<Value>,
    /// A base64-encoded token returned when `finish_reason` is `CONTINUATION`.
    ///
    /// Pass it to [`GenerateContentRequest::continuation_token`] to resume generation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub continuation_token: Option<String>,
}

impl Candidate {
    /// Returns the concatenated text from this candidate's content, if any.
    pub fn get_text(&self) -> Option<String> {
        match &self.content {
            Some(content) => content.get_text(),
            None => None,
        }
    }
}

/// A citation to a source used by the model in its response.
///
/// See <https://ai.google.dev/api/caching#CitationSource>.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CitationSource {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_index: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_index: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uri: Option<String>,
    /// License for the GitHub project that is attributed as a source for segment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,
}

/// Type alias for [`CitationSource`].
pub type Citation = CitationSource;

/// Metadata containing citations for a candidate's content.
///
/// See <https://ai.google.dev/api/caching#CitationMetadata>.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CitationMetadata {
    #[serde(default)]
    pub citation_sources: Vec<CitationSource>,
}

/// A safety rating for a piece of content across a specific harm category.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SafetyRating {
    pub category: String,
    pub probability: String,
    /// Was this content blocked because of this rating?
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocked: Option<bool>,
    pub probability_score: Option<f32>,
    pub severity: Option<String>,
    pub severity_score: Option<f32>,
}

/// Feedback explaining why a prompt was blocked.
///
/// See <https://ai.google.dev/api/generate-content#PromptFeedback>.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PromptFeedback {
    pub block_reason: Option<BlockReason>,
    pub block_reason_message: Option<String>,
    pub safety_ratings: Option<Vec<SafetyRating>>,
}

/// Reason a prompt was blocked.
///
/// See <https://ai.google.dev/api/generate-content#BlockReason>.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum BlockReason {
    #[serde(rename = "BLOCK_REASON_UNSPECIFIED")]
    Unspecified,
    #[serde(rename = "SAFETY")]
    Safety,
    #[serde(rename = "OTHER")]
    Other,
    #[serde(rename = "BLOCKLIST")]
    Blocklist,
    #[serde(rename = "PROHIBITED_CONTENT")]
    ProhibitedContent,
    #[serde(rename = "IMAGE_SAFETY")]
    ImageSafety,
}

/// Token usage statistics for a generate content request/response.
///
/// See <https://ai.google.dev/api/generate-content#UsageMetadata>.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_token_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cached_content_token_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub candidates_token_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_use_prompt_token_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thoughts_token_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_token_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_tokens_details: Option<Vec<ModalityTokenCount>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_tokens_details: Option<Vec<ModalityTokenCount>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub candidates_tokens_details: Option<Vec<ModalityTokenCount>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_use_prompt_tokens_details: Option<Vec<ModalityTokenCount>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_tier: Option<ServiceTier>,
}

/// A breakdown of tokens by [`Modality`].
///
/// See <https://ai.google.dev/api/generate-content#ModalityTokenCount>.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ModalityTokenCount {
    pub modality: Modality,
    pub token_count: u32,
}

/// Content modality reported in [`ModalityTokenCount`] or requested in
/// [`GenerationConfig::response_modalities`].
///
/// See <https://ai.google.dev/api/generate-content#Modality>.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Modality {
    #[serde(rename = "MODALITY_UNSPECIFIED")]
    Unspecified,
    #[serde(rename = "TEXT")]
    Text,
    #[serde(rename = "IMAGE")]
    Image,
    #[serde(rename = "VIDEO")]
    Video,
    #[serde(rename = "AUDIO")]
    Audio,
    #[serde(rename = "DOCUMENT")]
    Document,
}

/// The pricing/performance tier used to serve a request.
///
/// See <https://ai.google.dev/api/generate-content#ServiceTier>.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ServiceTier {
    #[serde(rename = "unspecified")]
    Unspecified,
    #[serde(rename = "flex")]
    Flex,
    #[serde(rename = "standard")]
    Standard,
    #[serde(rename = "priority")]
    Priority,
}

/// A declaration of a function the model may call.
///
/// See <https://ai.google.dev/api/caching#FunctionDeclaration>.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FunctionDeclaration {
    pub name: String,
    pub description: String,
    /// Whether the function blocks the conversation while it runs. The API defaults to
    /// [`Behavior::Blocking`]; currently only supported by `BidiGenerateContent`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub behavior: Option<Behavior>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters_json_schema: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_json_schema: Option<Value>,
}

/// How a declared function is executed relative to the conversation.
///
/// See <https://ai.google.dev/api/caching#Behavior>.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Behavior {
    Unspecified,
    /// The system waits for the function response before continuing the conversation.
    Blocking,
    /// The system does not wait for the function response and keeps talking to the user.
    NonBlocking,
}

/// See <https://ai.google.dev/api/caching#FunctionResponse>.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FunctionResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub name: String,
    pub response: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parts: Option<Vec<FunctionResponsePart>>, // TODO: Add missing properties from docs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub will_continue: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scheduling: Option<Scheduling>,
}

/// See <https://ai.google.dev/api/caching#FunctionResponsePart>.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum FunctionResponsePart {
    InlineData(FunctionResponseBlob),
    FileData(FunctionResponseFileData),
}

/// See <https://ai.google.dev/api/caching#FunctionResponseBlob>.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FunctionResponseBlob {
    /// Optional display name of the blob.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    pub mime_type: String,
    pub data: String,
}

impl FunctionResponseBlob {
    /// Creates a new [`FunctionResponseBlob`] with the given MIME type and base64 data.
    pub fn new(mime_type: impl Into<String>, data: impl Into<String>) -> Self {
        Self {
            display_name: None,
            mime_type: mime_type.into(),
            data: data.into(),
        }
    }

    /// Sets the optional display name for this blob.
    pub fn with_display_name(mut self, display_name: impl Into<String>) -> Self {
        self.display_name = Some(display_name.into());
        self
    }
}

/// URI-based data for a function response.
///
/// See <https://ai.google.dev/api/caching#FunctionResponseFileData>.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FunctionResponseFileData {
    /// Optional display name of the file to distinguish file data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    /// The IANA standard MIME type of the source data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    /// URI of the file.
    pub file_uri: String,
}

impl FunctionResponseFileData {
    /// Creates a new [`FunctionResponseFileData`] with the given file URI.
    pub fn new(file_uri: impl Into<String>) -> Self {
        Self {
            display_name: None,
            mime_type: None,
            file_uri: file_uri.into(),
        }
    }

    /// Sets the display name for this file data.
    pub fn with_display_name(mut self, display_name: impl Into<String>) -> Self {
        self.display_name = Some(display_name.into());
        self
    }

    /// Sets the MIME type for this file data.
    pub fn with_mime_type(mut self, mime_type: impl Into<String>) -> Self {
        self.mime_type = Some(mime_type.into());
        self
    }
}

/// See <https://ai.google.dev/api/caching#Scheduling>.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Scheduling {
    SchedulingUnspecified,
    Silent,
    WhenIdle,
    Interrupt,
}

/// A single property within a function's parameter schema.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FunctionParametersProperty {
    pub r#type: String,
    pub description: String,
}

/// The raw response from the `generateContent` endpoint, which may be a success or an error.
///
/// Use [`into_result`](GenerateContentResponse::into_result) to convert into a standard
/// `Result<GenerateContentResponseResult>`.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
#[expect(
    clippy::large_enum_variant,
    reason = "short-lived wire envelope; boxing the Ok variant would break callers that match on it"
)]
pub enum GenerateContentResponse {
    Ok(GenerateContentResponseResult),
    Error(GenerateContentResponseError),
}

impl From<GenerateContentResponse> for Result<GenerateContentResponseResult> {
    fn from(val: GenerateContentResponse) -> Self {
        match val {
            GenerateContentResponse::Ok(result) => Ok(result),
            GenerateContentResponse::Error(error) => Err(error.error.into()),
        }
    }
}

/// A successful response from the `generateContent` endpoint.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerateContentResponseResult {
    #[serde(default)]
    pub candidates: Vec<Candidate>,
    pub usage_metadata: Option<UsageMetadata>,
    pub prompt_feedback: Option<PromptFeedback>,
    /// The model version used to generate the response.
    pub model_version: Option<String>,
    /// Identifier for this response.
    pub response_id: Option<String>,
    /// The status of the model used to generate the response.
    pub model_status: Option<ModelStatus>,
}

/// The status of the model that served a request.
///
/// See <https://ai.google.dev/api/generate-content#ModelStatus>.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ModelStatus {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_stage: Option<ModelStage>,
    /// When the model will be retired, as an RFC 3339 timestamp.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retirement_time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// The release stage of a model.
///
/// See <https://ai.google.dev/api/generate-content#ModelStage>.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ModelStage {
    ModelStageUnspecified,
    /// Deprecated by the API.
    UnstableExperimental,
    Experimental,
    Preview,
    Stable,
    Legacy,
    /// Deprecated by the API.
    Deprecated,
    Retired,
}

/// An error response from the `generateContent` endpoint.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GenerateContentResponseError {
    pub error: VertexApiError,
}

impl GenerateContentResponse {
    /// Converts this response into a `Result`, mapping the error variant to [`crate::error::Error`].
    pub fn into_result(self) -> Result<GenerateContentResponseResult> {
        match self {
            GenerateContentResponse::Ok(result) => Ok(result),
            GenerateContentResponse::Error(error) => Err(error.error.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::types::{
        Candidate, CodeExecution, Content, Modality, Part, PartData, Role, ServiceTier, Tools,
        UrlContext, UsageMetadata,
    };

    use super::{
        Behavior, CitationMetadata, CitationSource, FunctionCallingConfig, FunctionCallingMode,
        FunctionResponse, FunctionResponseBlob, FunctionResponseFileData, FunctionResponsePart,
        GenerateContentRequest, GenerateContentResponse, GenerateContentResponseResult,
        GenerationConfig, GoogleSearch, HarmCategory, ImageConfig, ImageSearch, Interval, LatLng,
        MediaResolution, ModelStage, RetrievalConfig, SafetyRating, Scheduling, SearchTypes,
        SpeechConfig, ThinkingConfig, ThinkingLevel, ToolConfig, WebSearch,
    };

    #[test]
    fn thinking_level_round_trips_api_strings() -> serde_json::Result<()> {
        let cases = [
            (
                ThinkingLevel::ThinkingLevelUnspecified,
                "THINKING_LEVEL_UNSPECIFIED",
            ),
            (ThinkingLevel::Minimal, "MINIMAL"),
            (ThinkingLevel::Low, "LOW"),
            (ThinkingLevel::Medium, "MEDIUM"),
            (ThinkingLevel::High, "HIGH"),
        ];

        for (level, expected) in cases {
            assert_eq!(serde_json::to_value(level)?, json!(expected));
            assert_eq!(
                serde_json::from_value::<ThinkingLevel>(json!(expected))?,
                level
            );
        }
        Ok(())
    }

    #[test]
    fn thinking_config_omits_unset_include_thoughts_and_deserializes_without_it()
    -> serde_json::Result<()> {
        assert_eq!(serde_json::to_value(ThinkingConfig::default())?, json!({}));

        let without_include_thoughts = ThinkingConfig::builder()
            .thinking_budget(256)
            .thinking_level(ThinkingLevel::Minimal)
            .build();
        assert_eq!(
            serde_json::to_value(&without_include_thoughts)?,
            json!({
                "thinkingBudget": 256,
                "thinkingLevel": "MINIMAL"
            })
        );

        let parsed: ThinkingConfig = serde_json::from_value(json!({ "thinkingLevel": "MEDIUM" }))?;
        assert_eq!(
            parsed,
            ThinkingConfig {
                include_thoughts: None,
                thinking_budget: None,
                thinking_level: Some(ThinkingLevel::Medium),
            }
        );

        let empty_parsed: ThinkingConfig = serde_json::from_value(json!({}))?;
        assert_eq!(empty_parsed, ThinkingConfig::default());

        let full = ThinkingConfig::builder()
            .include_thoughts(true)
            .thinking_budget(1024)
            .thinking_level(ThinkingLevel::High)
            .build();
        let full_json = serde_json::to_value(&full)?;
        assert_eq!(
            full_json,
            json!({
                "includeThoughts": true,
                "thinkingBudget": 1024,
                "thinkingLevel": "HIGH"
            })
        );
        assert_eq!(serde_json::from_value::<ThinkingConfig>(full_json)?, full);
        Ok(())
    }

    #[test]
    fn media_resolution_round_trips_api_strings() -> serde_json::Result<()> {
        let cases = [
            (MediaResolution::Unspecified, "MEDIA_RESOLUTION_UNSPECIFIED"),
            (MediaResolution::Low, "MEDIA_RESOLUTION_LOW"),
            (MediaResolution::Medium, "MEDIA_RESOLUTION_MEDIUM"),
            (MediaResolution::High, "MEDIA_RESOLUTION_HIGH"),
        ];

        for (resolution, expected) in cases {
            assert_eq!(serde_json::to_value(resolution)?, json!(expected));
            assert_eq!(
                serde_json::from_value::<MediaResolution>(json!(expected))?,
                resolution
            );
        }
        Ok(())
    }

    #[test]
    fn generation_config_serializes_seed_response_modalities_and_media_resolution()
    -> serde_json::Result<()> {
        let config = GenerationConfig::builder()
            .seed(42)
            .response_modalities(vec![Modality::Text, Modality::Image])
            .media_resolution(MediaResolution::Medium)
            .build();

        let value = serde_json::to_value(&config)?;
        assert_eq!(
            value,
            json!({
                "seed": 42,
                "responseModalities": ["TEXT", "IMAGE"],
                "mediaResolution": "MEDIA_RESOLUTION_MEDIUM"
            })
        );
        assert_eq!(serde_json::from_value::<GenerationConfig>(value)?, config);
        Ok(())
    }

    #[test]
    fn generation_config_omits_unset_fields_and_round_trips_all_new_fields()
    -> serde_json::Result<()> {
        assert_eq!(
            serde_json::to_value(GenerationConfig::builder().build())?,
            json!({})
        );
        assert_eq!(serde_json::to_value(SpeechConfig::default())?, json!({}));
        assert_eq!(serde_json::to_value(ImageConfig::default())?, json!({}));

        let config = GenerationConfig::builder()
            .max_output_tokens(512)
            .temperature(0.5_f32)
            .top_p(0.875_f32)
            .top_k(40)
            .stop_sequences(vec!["END".to_string()])
            .candidate_count(1_u32)
            .response_mime_type("application/json")
            .response_schema(json!({ "type": "OBJECT" }))
            .response_json_schema(json!({
                "type": "object",
                "properties": { "answer": { "type": "string" } }
            }))
            .response_format(json!({ "type": "json_object" }))
            .seed(12345)
            .presence_penalty(0.25)
            .frequency_penalty(-0.5)
            .response_logprobs(true)
            .logprobs(5)
            .enable_enhanced_civic_answers(false)
            .response_modalities([Modality::Text, Modality::Audio])
            .media_resolution(MediaResolution::High)
            .speech_config(SpeechConfig {
                voice_config: Some(json!({
                    "prebuiltVoiceConfig": { "voiceName": "Kore" }
                })),
                multi_speaker_voice_config: Some(json!({ "speakerVoiceConfigs": [] })),
                language_code: Some("en-US".to_string()),
            })
            .thinking_config(
                ThinkingConfig::builder()
                    .include_thoughts(false)
                    .thinking_level(ThinkingLevel::Low)
                    .build(),
            )
            .image_config(ImageConfig {
                aspect_ratio: Some("16:9".to_string()),
                image_size: Some("2K".to_string()),
            })
            .enable_affective_dialog(true)
            .audio_transcription_config(json!({}))
            .translation_config(json!({ "targetLanguage": "pt-BR" }))
            .build();

        let value = serde_json::to_value(&config)?;
        assert_eq!(
            value,
            json!({
                "maxOutputTokens": 512,
                "temperature": 0.5,
                "topP": 0.875,
                "topK": 40,
                "stopSequences": ["END"],
                "candidateCount": 1,
                "responseMimeType": "application/json",
                "responseSchema": { "type": "OBJECT" },
                "responseJsonSchema": {
                    "type": "object",
                    "properties": { "answer": { "type": "string" } }
                },
                "responseFormat": { "type": "json_object" },
                "seed": 12345,
                "presencePenalty": 0.25,
                "frequencyPenalty": -0.5,
                "responseLogprobs": true,
                "logprobs": 5,
                "enableEnhancedCivicAnswers": false,
                "responseModalities": ["TEXT", "AUDIO"],
                "mediaResolution": "MEDIA_RESOLUTION_HIGH",
                "speechConfig": {
                    "voiceConfig": {
                        "prebuiltVoiceConfig": { "voiceName": "Kore" }
                    },
                    "multiSpeakerVoiceConfig": { "speakerVoiceConfigs": [] },
                    "languageCode": "en-US"
                },
                "thinkingConfig": {
                    "includeThoughts": false,
                    "thinkingLevel": "LOW"
                },
                "imageConfig": {
                    "aspectRatio": "16:9",
                    "imageSize": "2K"
                },
                "enableAffectiveDialog": true,
                "audioTranscriptionConfig": {},
                "translationConfig": { "targetLanguage": "pt-BR" }
            })
        );
        assert_eq!(serde_json::from_value::<GenerationConfig>(value)?, config);
        Ok(())
    }

    #[test]
    fn parses_response_with_model_status_and_candidate_metadata() -> serde_json::Result<()> {
        let input = json!({
            "candidates": [{
                "content": { "role": "model", "parts": [{ "text": "Partial answer" }] },
                "finishReason": "CONTINUATION",
                "finishMessage": "Generation paused.",
                "index": 0,
                "tokenCount": 3,
                "avgLogprobs": -0.25,
                "groundingMetadata": { "webSearchQueries": ["swallow airspeed"] },
                "continuationToken": "Y29udGludWU="
            }],
            "modelVersion": "gemini-3-pro-preview",
            "responseId": "2uUdaYPkG73WvdIP2aPs2Ak",
            "modelStatus": {
                "modelStage": "PREVIEW",
                "retirementTime": "2027-01-01T00:00:00Z",
                "message": "Preview model."
            }
        });

        let response: GenerateContentResponseResult = serde_json::from_value(input)?;

        assert_eq!(
            response.model_version.as_deref(),
            Some("gemini-3-pro-preview")
        );
        assert_eq!(
            response.response_id.as_deref(),
            Some("2uUdaYPkG73WvdIP2aPs2Ak")
        );
        let status = response.model_status.as_ref();
        assert_eq!(
            status.and_then(|s| s.model_stage),
            Some(ModelStage::Preview)
        );
        assert_eq!(
            status.and_then(|s| s.retirement_time.as_deref()),
            Some("2027-01-01T00:00:00Z")
        );
        let candidate = &response.candidates[0];
        assert_eq!(
            candidate.finish_message.as_deref(),
            Some("Generation paused.")
        );
        assert_eq!(candidate.token_count, Some(3));
        assert_eq!(candidate.avg_logprobs, Some(-0.25));
        assert_eq!(
            candidate.grounding_metadata,
            Some(json!({ "webSearchQueries": ["swallow airspeed"] }))
        );
        assert_eq!(
            candidate.continuation_token.as_deref(),
            Some("Y29udGludWU=")
        );
        Ok(())
    }

    #[test]
    fn model_stage_round_trips_api_strings() -> serde_json::Result<()> {
        let cases = [
            (ModelStage::ModelStageUnspecified, "MODEL_STAGE_UNSPECIFIED"),
            (ModelStage::UnstableExperimental, "UNSTABLE_EXPERIMENTAL"),
            (ModelStage::Experimental, "EXPERIMENTAL"),
            (ModelStage::Preview, "PREVIEW"),
            (ModelStage::Stable, "STABLE"),
            (ModelStage::Legacy, "LEGACY"),
            (ModelStage::Deprecated, "DEPRECATED"),
            (ModelStage::Retired, "RETIRED"),
        ];

        for (stage, expected) in cases {
            assert_eq!(serde_json::to_value(stage)?, json!(expected));
            assert_eq!(
                serde_json::from_value::<ModelStage>(json!(expected))?,
                stage
            );
        }
        Ok(())
    }

    #[test]
    fn behavior_round_trips_api_strings() -> serde_json::Result<()> {
        let cases = [
            (Behavior::Unspecified, "UNSPECIFIED"),
            (Behavior::Blocking, "BLOCKING"),
            (Behavior::NonBlocking, "NON_BLOCKING"),
        ];

        for (behavior, expected) in cases {
            assert_eq!(serde_json::to_value(behavior)?, json!(expected));
            assert_eq!(
                serde_json::from_value::<Behavior>(json!(expected))?,
                behavior
            );
        }
        Ok(())
    }

    #[test]
    fn new_harm_categories_round_trip_api_strings() -> serde_json::Result<()> {
        let cases = [
            (
                HarmCategory::CivicIntegrity,
                "HARM_CATEGORY_CIVIC_INTEGRITY",
            ),
            (HarmCategory::Jailbreak, "HARM_CATEGORY_JAILBREAK"),
        ];

        for (category, expected) in cases {
            assert_eq!(serde_json::to_value(&category)?, json!(expected));
            assert_eq!(
                serde_json::from_value::<HarmCategory>(json!(expected))?,
                category
            );
        }
        Ok(())
    }

    #[test]
    fn request_serializes_new_optional_fields() -> serde_json::Result<()> {
        let request = GenerateContentRequest::builder()
            .cached_content("cachedContents/abc")
            .service_tier(ServiceTier::Flex)
            .store(false)
            .labels([("team".to_string(), "search".to_string())].into())
            .continuation_token("Y29udGludWU=")
            .build();

        let value = serde_json::to_value(&request)?;

        assert_eq!(
            value,
            json!({
                "contents": [],
                "cachedContent": "cachedContents/abc",
                "serviceTier": "flex",
                "store": false,
                "labels": { "team": "search" },
                "continuationToken": "Y29udGludWU="
            })
        );
        Ok(())
    }

    #[test]
    fn tools_serialize_new_tool_fields_and_omit_unset_ones() -> serde_json::Result<()> {
        let empty = Tools::default();
        let tools = Tools {
            code_execution: Some(CodeExecution {}),
            url_context: Some(UrlContext {}),
            google_maps: Some(json!({ "enableWidget": true })),
            ..Default::default()
        };

        assert_eq!(serde_json::to_value(&empty)?, json!({}));
        assert_eq!(
            serde_json::to_value(&tools)?,
            json!({
                "codeExecution": {},
                "urlContext": {},
                "googleMaps": { "enableWidget": true }
            })
        );
        Ok(())
    }

    fn part(response: FunctionResponse) -> Part {
        Part {
            thought: None,
            thought_signature: None,
            part_metadata: None,
            media_resolution: None,
            video_metadata: None,
            speech_metadata: None,
            data: PartData::FunctionResponse(response),
        }
    }

    #[test]
    fn request_without_tool_config_omits_field() -> serde_json::Result<()> {
        let request = GenerateContentRequest::builder().build();

        let value = serde_json::to_value(&request)?;

        assert_eq!(value, json!({ "contents": [] }));
        Ok(())
    }

    #[test]
    fn request_serializes_function_calling_config() -> serde_json::Result<()> {
        let request = GenerateContentRequest::builder()
            .tool_config(ToolConfig {
                function_calling_config: Some(FunctionCallingConfig {
                    mode: Some(FunctionCallingMode::Any),
                    allowed_function_names: Some(vec![
                        "get_weather".to_string(),
                        "get_time".to_string(),
                    ]),
                }),
                ..Default::default()
            })
            .build();

        let value = serde_json::to_value(&request)?;

        assert_eq!(
            value,
            json!({
                "contents": [],
                "toolConfig": {
                    "functionCallingConfig": {
                        "mode": "ANY",
                        "allowedFunctionNames": ["get_weather", "get_time"]
                    }
                }
            })
        );
        Ok(())
    }

    #[test]
    fn function_calling_mode_round_trips_api_strings() -> serde_json::Result<()> {
        let cases = [
            (FunctionCallingMode::ModeUnspecified, "MODE_UNSPECIFIED"),
            (FunctionCallingMode::Auto, "AUTO"),
            (FunctionCallingMode::Any, "ANY"),
            (FunctionCallingMode::None, "NONE"),
            (FunctionCallingMode::Validated, "VALIDATED"),
        ];

        for (mode, expected) in cases {
            assert_eq!(serde_json::to_value(mode)?, json!(expected));
            assert_eq!(
                serde_json::from_value::<FunctionCallingMode>(json!(expected))?,
                mode
            );
        }
        Ok(())
    }

    #[test]
    fn tool_config_omits_unset_optional_fields() -> serde_json::Result<()> {
        let empty = ToolConfig::default();
        let nested = ToolConfig {
            function_calling_config: Some(FunctionCallingConfig::default()),
            retrieval_config: Some(RetrievalConfig::default()),
            include_server_side_tool_invocations: None,
        };

        assert_eq!(serde_json::to_value(&empty)?, json!({}));
        assert_eq!(
            serde_json::to_value(&nested)?,
            json!({ "functionCallingConfig": {}, "retrievalConfig": {} })
        );
        Ok(())
    }

    #[test]
    fn tool_config_serializes_retrieval_config() -> serde_json::Result<()> {
        let config = ToolConfig {
            retrieval_config: Some(RetrievalConfig {
                lat_lng: Some(LatLng {
                    latitude: 51.5,
                    longitude: -0.125,
                }),
                language_code: Some("en-GB".to_string()),
            }),
            include_server_side_tool_invocations: Some(true),
            ..Default::default()
        };

        let value = serde_json::to_value(&config)?;

        assert_eq!(
            value,
            json!({
                "retrievalConfig": {
                    "latLng": { "latitude": 51.5, "longitude": -0.125 },
                    "languageCode": "en-GB"
                },
                "includeServerSideToolInvocations": true
            })
        );
        Ok(())
    }

    #[test]
    fn function_response_omits_unset_optional_fields() -> serde_json::Result<()> {
        let part = part(FunctionResponse {
            id: None,
            name: "get_weather".to_string(),
            response: json!({ "temperature": 21 }),
            parts: None,
            will_continue: None,
            scheduling: None,
        });

        let value = serde_json::to_value(&part)?;

        assert_eq!(
            value,
            json!({
                "functionResponse": { "name": "get_weather", "response": { "temperature": 21 } }
            })
        );
        Ok(())
    }

    #[test]
    fn parses_blocked_prompt_response() {
        let input =
            r#"{"promptFeedback":{"blockReason":"SAFETY"},"usageMetadata":{"promptTokenCount":5}}"#;
        let response: GenerateContentResponse = serde_json::from_str(input).unwrap();
        let GenerateContentResponse::Ok(response) = response else {
            panic!("expected successful response with prompt feedback");
        };

        assert!(response.candidates.is_empty());
        assert_eq!(
            response.prompt_feedback.unwrap().block_reason,
            Some(super::BlockReason::Safety)
        );
    }

    #[test]
    fn parses_blocked_prompt_response_with_message() {
        let input = r#"{"promptFeedback":{"blockReason":"PROHIBITED_CONTENT","blockReasonMessage":"The input violates safety policies."}}"#;
        let response: GenerateContentResponse = serde_json::from_str(input).unwrap();
        let GenerateContentResponse::Ok(response) = response else {
            panic!("expected successful response with prompt feedback");
        };

        assert!(response.candidates.is_empty());
        let feedback = response.prompt_feedback.unwrap();
        assert_eq!(
            feedback.block_reason,
            Some(super::BlockReason::ProhibitedContent)
        );
        assert_eq!(
            feedback.block_reason_message.as_deref(),
            Some("The input violates safety policies.")
        );
    }

    #[test]
    fn function_response_includes_set_optional_fields() -> serde_json::Result<()> {
        let part = part(FunctionResponse {
            id: Some("call-1".to_string()),
            name: "get_weather".to_string(),
            response: json!({ "temperature": 21 }),
            parts: Some(vec![FunctionResponsePart::InlineData(
                FunctionResponseBlob {
                    display_name: None,
                    mime_type: "image/png".to_string(),
                    data: "aGVsbG8=".to_string(),
                },
            )]),
            will_continue: Some(false),
            scheduling: Some(Scheduling::Silent),
        });

        let value = serde_json::to_value(&part)?;

        assert_eq!(
            value,
            json!({
                "functionResponse": {
                    "id": "call-1",
                    "name": "get_weather",
                    "response": { "temperature": 21 },
                    "parts": [{ "inlineData": { "mimeType": "image/png", "data": "aGVsbG8=" } }],
                    "willContinue": false,
                    "scheduling": "SILENT"
                }
            })
        );
        Ok(())
    }

    #[test]
    fn citation_metadata_parsed_from_citation_sources_serializes_back_to_citation_sources()
    -> serde_json::Result<()> {
        let input = json!({
            "citationSources": [
                {
                    "startIndex": 0,
                    "endIndex": 50,
                    "uri": "https://github.com/example/repo",
                    "license": "Apache-2.0"
                }
            ]
        });

        let metadata: CitationMetadata = serde_json::from_value(input.clone())?;

        assert_eq!(metadata.citation_sources.len(), 1);
        let citation = &metadata.citation_sources[0];
        assert_eq!(citation.start_index, Some(0));
        assert_eq!(citation.end_index, Some(50));
        assert_eq!(
            citation.uri.as_deref(),
            Some("https://github.com/example/repo")
        );
        assert_eq!(citation.license.as_deref(), Some("Apache-2.0"));

        let serialized = serde_json::to_value(&metadata)?;
        assert_eq!(serialized, input);
        assert!(
            serialized
                .as_object()
                .unwrap()
                .contains_key("citationSources")
        );
        Ok(())
    }

    #[test]
    fn citation_source_parses_with_and_without_license() -> serde_json::Result<()> {
        let with_license = json!({
            "startIndex": 10,
            "endIndex": 30,
            "uri": "https://example.com",
            "license": "BSD-3-Clause"
        });
        let parsed_with: CitationSource = serde_json::from_value(with_license.clone())?;
        assert_eq!(parsed_with.license.as_deref(), Some("BSD-3-Clause"));
        assert_eq!(serde_json::to_value(&parsed_with)?, with_license);

        let without_license = json!({
            "startIndex": 10,
            "endIndex": 30,
            "uri": "https://example.com"
        });
        let parsed_without: CitationSource = serde_json::from_value(without_license.clone())?;
        assert_eq!(parsed_without.license, None);
        assert_eq!(serde_json::to_value(&parsed_without)?, without_license);
        Ok(())
    }

    #[test]
    fn safety_rating_parses_with_and_without_blocked() -> serde_json::Result<()> {
        let with_blocked = json!({
            "category": "HARM_CATEGORY_HATE_SPEECH",
            "probability": "HIGH",
            "blocked": true,
            "probabilityScore": 0.95
        });
        let rating_blocked: SafetyRating = serde_json::from_value(with_blocked)?;
        assert_eq!(rating_blocked.blocked, Some(true));
        assert_eq!(rating_blocked.category, "HARM_CATEGORY_HATE_SPEECH");
        assert_eq!(rating_blocked.probability, "HIGH");
        let serialized_blocked = serde_json::to_value(&rating_blocked)?;
        assert_eq!(serialized_blocked["blocked"], true);

        let without_blocked = json!({
            "category": "HARM_CATEGORY_DANGEROUS_CONTENT",
            "probability": "NEGLIGIBLE"
        });
        let rating_unblocked: SafetyRating = serde_json::from_value(without_blocked)?;
        assert_eq!(rating_unblocked.blocked, None);
        let serialized_unblocked = serde_json::to_value(&rating_unblocked)?;
        assert!(
            !serialized_unblocked
                .as_object()
                .unwrap()
                .contains_key("blocked")
        );
        Ok(())
    }

    #[test]
    fn google_search_parses_and_serializes_search_types_and_time_range_filter()
    -> serde_json::Result<()> {
        let input = json!({
            "searchTypes": {
                "webSearch": {},
                "imageSearch": {}
            },
            "timeRangeFilter": {
                "startTime": "2026-01-01T00:00:00Z",
                "endTime": "2026-10-30T00:00:00Z"
            }
        });

        let search: GoogleSearch = serde_json::from_value(input.clone())?;
        let search_types = search.search_types.as_ref().expect("searchTypes");
        assert_eq!(search_types.web_search, Some(WebSearch {}));
        assert_eq!(search_types.image_search, Some(ImageSearch {}));

        let filter = search.time_range_filter.as_ref().expect("timeRangeFilter");
        assert_eq!(filter.start_time.as_deref(), Some("2026-01-01T00:00:00Z"));
        assert_eq!(filter.end_time.as_deref(), Some("2026-10-30T00:00:00Z"));

        let serialized = serde_json::to_value(&search)?;
        assert_eq!(serialized, input);

        let constructed = GoogleSearch {
            search_types: Some(SearchTypes {
                web_search: Some(WebSearch {}),
                image_search: Some(ImageSearch {}),
            }),
            time_range_filter: Some(Interval::new(
                "2026-01-01T00:00:00Z",
                "2026-10-30T00:00:00Z",
            )),
        };
        assert_eq!(serde_json::to_value(&constructed)?, input);
        Ok(())
    }

    #[test]
    fn google_search_empty_serializes_to_empty_object() -> serde_json::Result<()> {
        let empty = GoogleSearch::default();
        let value = serde_json::to_value(&empty)?;
        assert_eq!(value, json!({}));

        let parsed: GoogleSearch = serde_json::from_value(json!({}))?;
        assert_eq!(parsed, GoogleSearch::default());
        Ok(())
    }

    #[test]
    fn function_response_blob_parses_with_and_without_display_name() -> serde_json::Result<()> {
        let with_name = json!({
            "displayName": "photo.png",
            "mimeType": "image/png",
            "data": "aGVsbG8="
        });
        let blob: FunctionResponseBlob = serde_json::from_value(with_name.clone())?;
        assert_eq!(blob.display_name.as_deref(), Some("photo.png"));
        assert_eq!(blob.mime_type, "image/png");
        assert_eq!(blob.data, "aGVsbG8=");
        assert_eq!(serde_json::to_value(&blob)?, with_name);

        let without_name = json!({
            "mimeType": "image/png",
            "data": "aGVsbG8="
        });
        let blob: FunctionResponseBlob = serde_json::from_value(without_name.clone())?;
        assert_eq!(blob.display_name, None);
        assert_eq!(serde_json::to_value(&blob)?, without_name);
        Ok(())
    }

    #[test]
    fn function_response_part_parses_and_serializes_file_data() -> serde_json::Result<()> {
        let input = json!({
            "fileData": {
                "displayName": "data.csv",
                "mimeType": "text/csv",
                "fileUri": "https://generativelanguage.googleapis.com/v1beta/files/abc123"
            }
        });

        let part: FunctionResponsePart = serde_json::from_value(input.clone())?;
        assert_eq!(
            part,
            FunctionResponsePart::FileData(FunctionResponseFileData {
                display_name: Some("data.csv".to_string()),
                mime_type: Some("text/csv".to_string()),
                file_uri: "https://generativelanguage.googleapis.com/v1beta/files/abc123"
                    .to_string(),
            })
        );
        assert_eq!(serde_json::to_value(&part)?, input);

        let minimal = json!({
            "fileData": {
                "fileUri": "https://example.com/file"
            }
        });
        let part_min: FunctionResponsePart = serde_json::from_value(minimal.clone())?;
        assert_eq!(
            part_min,
            FunctionResponsePart::FileData(FunctionResponseFileData::new(
                "https://example.com/file"
            ))
        );
        assert_eq!(serde_json::to_value(&part_min)?, minimal);
        Ok(())
    }

    #[test]
    pub fn parses_usage_metadata() {
        let input = r#"
        {
          "promptTokenCount": 11,
          "candidatesTokenCount": 202,
          "totalTokenCount": 1041,
          "promptTokensDetails": [
            {
              "modality": "TEXT",
              "tokenCount": 11
            }
          ],
          "thoughtsTokenCount": 828
        }"#;
        let usage: UsageMetadata = serde_json::from_str(input).unwrap();
        assert_eq!(usage.prompt_token_count, Some(11));
        assert_eq!(usage.candidates_token_count, Some(202));
        assert_eq!(usage.total_token_count, Some(1041));
        assert_eq!(usage.thoughts_token_count, Some(828));
        let prompt_details = usage.prompt_tokens_details.expect("promptTokensDetails");
        assert_eq!(prompt_details.len(), 1);
        assert_eq!(prompt_details[0].modality, Modality::Text);
        assert_eq!(prompt_details[0].token_count, 11);
    }

    #[test]
    pub fn parses_usage_metadata_full() {
        let input = r#"
        {
          "promptTokenCount": 100,
          "cachedContentTokenCount": 40,
          "candidatesTokenCount": 50,
          "toolUsePromptTokenCount": 7,
          "thoughtsTokenCount": 200,
          "totalTokenCount": 357,
          "promptTokensDetails": [
            { "modality": "TEXT", "tokenCount": 80 },
            { "modality": "IMAGE", "tokenCount": 20 }
          ],
          "cacheTokensDetails": [
            { "modality": "TEXT", "tokenCount": 40 }
          ],
          "candidatesTokensDetails": [
            { "modality": "TEXT", "tokenCount": 50 }
          ],
          "toolUsePromptTokensDetails": [
            { "modality": "TEXT", "tokenCount": 7 }
          ],
          "serviceTier": "standard"
        }"#;
        let usage: UsageMetadata = serde_json::from_str(input).unwrap();
        assert_eq!(usage.prompt_token_count, Some(100));
        assert_eq!(usage.cached_content_token_count, Some(40));
        assert_eq!(usage.candidates_token_count, Some(50));
        assert_eq!(usage.tool_use_prompt_token_count, Some(7));
        assert_eq!(usage.thoughts_token_count, Some(200));
        assert_eq!(usage.total_token_count, Some(357));
        assert_eq!(usage.service_tier, Some(ServiceTier::Standard));

        let prompt_details = usage.prompt_tokens_details.unwrap();
        assert_eq!(prompt_details.len(), 2);
        assert_eq!(prompt_details[0].modality, Modality::Text);
        assert_eq!(prompt_details[0].token_count, 80);
        assert_eq!(prompt_details[1].modality, Modality::Image);
        assert_eq!(prompt_details[1].token_count, 20);

        assert_eq!(usage.cache_tokens_details.unwrap()[0].token_count, 40);
        assert_eq!(usage.candidates_tokens_details.unwrap()[0].token_count, 50);
        assert_eq!(
            usage.tool_use_prompt_tokens_details.unwrap()[0].token_count,
            7
        );
    }

    #[test]
    pub fn parses_minimal_usage_metadata() {
        let input = r#"{ "totalTokenCount": 12 }"#;
        let usage: UsageMetadata = serde_json::from_str(input).unwrap();
        assert_eq!(usage.total_token_count, Some(12));
        assert!(usage.prompt_token_count.is_none());
        assert!(usage.cached_content_token_count.is_none());
        assert!(usage.thoughts_token_count.is_none());
        assert!(usage.tool_use_prompt_token_count.is_none());
        assert!(usage.prompt_tokens_details.is_none());
        assert!(usage.service_tier.is_none());
    }

    #[test]
    pub fn skips_none_usage_metadata_fields_on_serialize() {
        let usage = UsageMetadata {
            prompt_token_count: Some(5),
            total_token_count: Some(5),
            ..Default::default()
        };
        let json = serde_json::to_value(&usage).unwrap();
        let obj = json.as_object().unwrap();
        assert!(obj.contains_key("promptTokenCount"));
        assert!(obj.contains_key("totalTokenCount"));
        assert!(!obj.contains_key("cachedContentTokenCount"));
        assert!(!obj.contains_key("thoughtsTokenCount"));
        assert!(!obj.contains_key("toolUsePromptTokenCount"));
        assert!(!obj.contains_key("promptTokensDetails"));
        assert!(!obj.contains_key("serviceTier"));
    }

    #[test]
    fn request_system_instruction_omits_unset_role() -> serde_json::Result<()> {
        let request = GenerateContentRequest::builder()
            .contents(vec![
                Content::builder()
                    .role(Role::User)
                    .add_text_part("Hello!")
                    .build(),
            ])
            .system_instruction(Content::system_prompt("You are a helpful assistant."))
            .build();

        let value = serde_json::to_value(&request)?;

        assert_eq!(
            value,
            json!({
                "contents": [{ "role": "user", "parts": [{ "text": "Hello!" }] }],
                "systemInstruction": {
                    "parts": [{ "text": "You are a helpful assistant." }]
                }
            })
        );
        Ok(())
    }

    #[test]
    pub fn parses_all_modalities() {
        let cases = [
            (r#""MODALITY_UNSPECIFIED""#, Modality::Unspecified),
            (r#""TEXT""#, Modality::Text),
            (r#""IMAGE""#, Modality::Image),
            (r#""VIDEO""#, Modality::Video),
            (r#""AUDIO""#, Modality::Audio),
            (r#""DOCUMENT""#, Modality::Document),
        ];
        for (input, expected) in cases {
            let parsed: Modality = serde_json::from_str(input).unwrap();
            assert_eq!(parsed, expected);
            assert_eq!(serde_json::to_string(&parsed).unwrap(), input);
        }
    }

    #[test]
    pub fn parses_all_service_tiers() {
        let cases = [
            (r#""unspecified""#, ServiceTier::Unspecified),
            (r#""flex""#, ServiceTier::Flex),
            (r#""standard""#, ServiceTier::Standard),
            (r#""priority""#, ServiceTier::Priority),
        ];
        for (input, expected) in cases {
            let parsed: ServiceTier = serde_json::from_str(input).unwrap();
            assert_eq!(parsed, expected);
            assert_eq!(serde_json::to_string(&parsed).unwrap(), input);
        }
    }

    #[test]
    pub fn parses_candidate() {
        let input = r#"
          {
            "content": {
              "parts": [
                {
                  "text": "**What do you mean? An African or a European swallow?**\n\nIf you are looking for the actual physics rather than the *Monty Python and the Holy Grail* reference, here is the breakdown:\n\n**1. The European Swallow**\nBased on an analysis published by Jonathan Corum (using data on the Strouhal number of cruising flight), the estimated airspeed velocity of an unladen European Swallow is roughly **11 meters per second**, or **24 miles per hour**.\n\n**2. The African Swallow**\nData on the African swallow is scarcer, mostly because—as the guard in the movie points out—African swallows are non-migratory. However, since they are similar in size to their European counterparts, their cruising speed would likely be comparable.\n\nBut of course, the real question is: *Could it carry a coconut?* (A five-ounce bird could not carry a one-pound coconut. It is a simple question of weight ratios.)",
                  "thoughtSignature": "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"
          }
        ],
        "role": "model"
      },
      "finishReason": "STOP",
      "index": 0
    }"#;
        let _ = serde_json::from_str::<Candidate>(input).unwrap();
    }

    #[test]
    pub fn parses_google_ai_response() {
        let input = r#"
        {
          "candidates": [
            {
              "content": {
                "parts": [
                  {
                    "text": "Hello, world!"
                  }
                ],
                "role": "model"
              },
              "finishReason": "STOP",
              "index": 0
            }
          ],
  "usageMetadata": {
    "promptTokenCount": 11,
    "candidatesTokenCount": 202,
    "totalTokenCount": 1041,
    "promptTokensDetails": [
      {
        "modality": "TEXT",
        "tokenCount": 11
      }
    ],
    "thoughtsTokenCount": 828
  },
  "modelVersion": "gemini-3-pro-preview",
  "responseId": "2uUdaYPkG73WvdIP2aPs2Ak"
}
"#;
        let _ = serde_json::from_str::<GenerateContentResponseResult>(input).unwrap();
    }
}
