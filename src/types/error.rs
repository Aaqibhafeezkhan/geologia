// Copyright 2026 Andre Cipriani Bandarra
// SPDX-License-Identifier: Apache-2.0

use std::fmt::Formatter;
use std::time::Duration;

use serde::{Deserialize, Serialize};

/// A structured error returned by the Vertex AI / Gemini API.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VertexApiError {
    /// The HTTP status code.
    pub code: i32,
    /// A human-readable error message.
    pub message: String,
    /// The gRPC status string (e.g. `"INVALID_ARGUMENT"`).
    pub status: String,
    /// Optional additional error details.
    pub details: Option<Vec<serde_json::Value>>,
}

impl core::fmt::Display for VertexApiError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        writeln!(f, "Vertex API Error {} - {}", self.code, self.message)?;
        Ok(())
    }
}

impl std::error::Error for VertexApiError {}

impl VertexApiError {
    /// Returns the recommended retry delay if provided in the error details.
    ///
    /// When Gemini rejects a request due to quota limits (`429 RESOURCE_EXHAUSTED`),
    /// its error details usually include a `google.rpc.RetryInfo` entry specifying
    /// how long the client should wait before retrying (e.g. `"37s"` or `"1.5s"`).
    ///
    /// Returns `None` if there are no error details, no `RetryInfo` entry is found,
    /// or the `retryDelay` value is missing or cannot be parsed.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::Duration;
    /// use geologia::prelude::VertexApiError;
    /// use serde_json::json;
    ///
    /// let err = VertexApiError {
    ///     code: 429,
    ///     message: "Resource exhausted".into(),
    ///     status: "RESOURCE_EXHAUSTED".into(),
    ///     details: Some(vec![json!({
    ///         "@type": "type.googleapis.com/google.rpc.RetryInfo",
    ///         "retryDelay": "37s"
    ///     })]),
    /// };
    ///
    /// assert_eq!(err.retry_delay(), Some(Duration::from_secs(37)));
    /// ```
    pub fn retry_delay(&self) -> Option<Duration> {
        self.details.as_deref()?.iter().find_map(|detail| {
            let is_retry_info = detail
                .get("@type")
                .and_then(|t| t.as_str())
                .is_some_and(is_retry_info_type);
            if !is_retry_info {
                return None;
            }
            detail
                .get("retryDelay")
                .or_else(|| detail.get("retry_delay"))
                .and_then(|d| d.as_str())
                .and_then(parse_duration)
        })
    }
}

fn is_retry_info_type(type_url: &str) -> bool {
    type_url == "google.rpc.RetryInfo" || type_url.ends_with("/google.rpc.RetryInfo")
}

fn parse_duration(s: &str) -> Option<Duration> {
    let s = s.strip_suffix('s')?;
    if s.is_empty() {
        return None;
    }

    let (secs_str, nanos_str) = match s.split_once('.') {
        Some((secs, nanos)) => (secs, Some(nanos)),
        None => (s, None),
    };

    if secs_str.is_empty() || !secs_str.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    let secs: u64 = secs_str.parse().ok()?;

    let nanos = match nanos_str {
        Some(nanos_str) => {
            if nanos_str.is_empty()
                || nanos_str.len() > 9
                || !nanos_str.chars().all(|c| c.is_ascii_digit())
            {
                return None;
            }
            let parsed: u32 = nanos_str.parse().ok()?;
            let factor = 10u32.checked_pow(9 - nanos_str.len() as u32)?;
            parsed.checked_mul(factor)?
        }
        None => 0,
    };

    Some(Duration::new(secs, nanos))
}

/// The top-level error envelope returned by the Gemini API, wrapping a [`VertexApiError`]
/// under an `error` key (e.g. `{ "error": { "code": 400, "status": "INVALID_ARGUMENT", .. } }`).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VertexApiErrorResponse {
    /// The wrapped error payload.
    pub error: VertexApiError,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_gemini_error_envelope() {
        let body = r#"{
            "error": {
                "code": 400,
                "message": "Invalid value at 'contents[0].parts[0]'",
                "status": "INVALID_ARGUMENT",
                "details": [{"@type": "type.googleapis.com/google.rpc.BadRequest"}]
            }
        }"#;

        let parsed: VertexApiErrorResponse =
            serde_json::from_str(body).expect("should parse error envelope");
        assert_eq!(parsed.error.code, 400);
        assert_eq!(parsed.error.status, "INVALID_ARGUMENT");
        assert_eq!(
            parsed.error.message,
            "Invalid value at 'contents[0].parts[0]'"
        );
        assert_eq!(parsed.error.details.unwrap().len(), 1);
    }

    #[test]
    fn retry_delay_whole_seconds() {
        let err = VertexApiError {
            code: 429,
            message: "Quota exceeded".into(),
            status: "RESOURCE_EXHAUSTED".into(),
            details: Some(vec![serde_json::json!({
                "@type": "type.googleapis.com/google.rpc.RetryInfo",
                "retryDelay": "37s"
            })]),
        };
        assert_eq!(err.retry_delay(), Some(Duration::from_secs(37)));
    }

    #[test]
    fn retry_delay_fractional_seconds() {
        let err = VertexApiError {
            code: 429,
            message: "Quota exceeded".into(),
            status: "RESOURCE_EXHAUSTED".into(),
            details: Some(vec![serde_json::json!({
                "@type": "type.googleapis.com/google.rpc.RetryInfo",
                "retryDelay": "1.5s"
            })]),
        };
        assert_eq!(err.retry_delay(), Some(Duration::from_millis(1500)));

        let err_sub_millis = VertexApiError {
            code: 429,
            message: "Quota exceeded".into(),
            status: "RESOURCE_EXHAUSTED".into(),
            details: Some(vec![serde_json::json!({
                "@type": "type.googleapis.com/google.rpc.RetryInfo",
                "retryDelay": "0.005s"
            })]),
        };
        assert_eq!(err_sub_millis.retry_delay(), Some(Duration::from_millis(5)));
    }

    #[test]
    fn retry_delay_without_details() {
        let err = VertexApiError {
            code: 429,
            message: "Quota exceeded".into(),
            status: "RESOURCE_EXHAUSTED".into(),
            details: None,
        };
        assert_eq!(err.retry_delay(), None);
    }

    #[test]
    fn retry_delay_without_retry_info() {
        let err = VertexApiError {
            code: 400,
            message: "Bad request".into(),
            status: "INVALID_ARGUMENT".into(),
            details: Some(vec![serde_json::json!({
                "@type": "type.googleapis.com/google.rpc.BadRequest"
            })]),
        };
        assert_eq!(err.retry_delay(), None);

        let err_empty = VertexApiError {
            code: 429,
            message: "Quota exceeded".into(),
            status: "RESOURCE_EXHAUSTED".into(),
            details: Some(vec![]),
        };
        assert_eq!(err_empty.retry_delay(), None);
    }

    #[test]
    fn retry_delay_with_unparsable_or_missing_retry_delay() {
        let cases = [
            "invalid",
            "37",            // missing 's' suffix
            "-1s",           // negative
            "-1.5s",         // negative fractional
            "1.5.0s",        // multiple decimals
            ".5s",           // missing leading digits
            "1.s",           // missing fractional digits
            "",              // empty
            "s",             // only suffix
            " 37s",          // leading space
            "37s ",          // trailing space
            "NaNs",          // non-digit
            "1e3s",          // scientific notation
            "1.1234567890s", // more than 9 fractional digits (sub-nanosecond)
        ];

        for case in cases {
            let err = VertexApiError {
                code: 429,
                message: "Quota exceeded".into(),
                status: "RESOURCE_EXHAUSTED".into(),
                details: Some(vec![serde_json::json!({
                    "@type": "type.googleapis.com/google.rpc.RetryInfo",
                    "retryDelay": case
                })]),
            };
            assert_eq!(
                err.retry_delay(),
                None,
                "expected None for delay string: {case}"
            );
        }

        // Missing retryDelay field
        let err_missing = VertexApiError {
            code: 429,
            message: "Quota exceeded".into(),
            status: "RESOURCE_EXHAUSTED".into(),
            details: Some(vec![serde_json::json!({
                "@type": "type.googleapis.com/google.rpc.RetryInfo"
            })]),
        };
        assert_eq!(err_missing.retry_delay(), None);

        // retryDelay is a number, not a string
        let err_non_string = VertexApiError {
            code: 429,
            message: "Quota exceeded".into(),
            status: "RESOURCE_EXHAUSTED".into(),
            details: Some(vec![serde_json::json!({
                "@type": "type.googleapis.com/google.rpc.RetryInfo",
                "retryDelay": 37
            })]),
        };
        assert_eq!(err_non_string.retry_delay(), None);
    }

    #[test]
    fn retry_delay_with_multiple_details_entries() {
        let err = VertexApiError {
            code: 429,
            message: "Quota exceeded".into(),
            status: "RESOURCE_EXHAUSTED".into(),
            details: Some(vec![
                serde_json::json!({
                    "@type": "type.googleapis.com/google.rpc.ErrorInfo",
                    "reason": "RATE_LIMIT_EXCEEDED"
                }),
                serde_json::json!({
                    "@type": "type.googleapis.com/google.rpc.RetryInfo",
                    "retryDelay": "42s"
                }),
            ]),
        };
        assert_eq!(err.retry_delay(), Some(Duration::from_secs(42)));
    }

    #[test]
    fn retry_delay_with_multiple_retry_info_entries_skips_malformed() {
        let err = VertexApiError {
            code: 429,
            message: "Quota exceeded".into(),
            status: "RESOURCE_EXHAUSTED".into(),
            details: Some(vec![
                serde_json::json!({
                    "@type": "type.googleapis.com/google.rpc.RetryInfo",
                    "retryDelay": "malformed"
                }),
                serde_json::json!({
                    "@type": "type.googleapis.com/google.rpc.RetryInfo",
                    "retryDelay": "20s"
                }),
            ]),
        };
        assert_eq!(err.retry_delay(), Some(Duration::from_secs(20)));
    }

    #[test]
    fn retry_delay_snake_case_and_short_type() {
        let err = VertexApiError {
            code: 429,
            message: "Quota exceeded".into(),
            status: "RESOURCE_EXHAUSTED".into(),
            details: Some(vec![serde_json::json!({
                "@type": "google.rpc.RetryInfo",
                "retry_delay": "10s"
            })]),
        };
        assert_eq!(err.retry_delay(), Some(Duration::from_secs(10)));
    }
}
