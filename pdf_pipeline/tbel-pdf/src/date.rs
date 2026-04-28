//! Date normalization adapter for financial report headers.
//!
//! Uses Mistral chat completion with a dedicated prompt template to convert
//! Russian financial date headers into `MM.YYYY`.

use async_trait::async_trait;
use reqwest::Client;
use std::collections::HashMap;
use std::sync::Mutex;
use thiserror::Error;

/// Error types for date normalization.
#[derive(Debug, Clone, Error)]
pub enum DateError {
    #[error("Parse error: {0}")]
    Parse(String),
}

const DEFAULT_MISTRAL_MODEL: &str = "mistral-large-latest";
const DATE_PROMPT_TEMPLATE: &str = include_str!("../prompts/financial_date_extraction.txt");
const MAX_RETRIES: u8 = 3;

#[derive(Debug, Clone)]
pub struct DateNormalizerConfig {
    /// Optional Mistral API key. If None, date normalization falls back to rule-based logic.
    pub api_key: Option<String>,
    /// Model identifier for date normalization (e.g., "mistral-small-latest").
    pub model: String,
}

impl Default for DateNormalizerConfig {
    fn default() -> Self {
        Self {
            api_key: None,
            model: DEFAULT_MISTRAL_MODEL.to_string(),
        }
    }
}

/// Trait for date header normalization.
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
pub trait DateNormalizer: Send + Sync {
    /// Normalize a date header to MM.YYYY format.
    ///
    /// # Errors
    /// Returns `DateError` only when execution-level failures happen.
    /// Parsing failures from the model return the original header.
    async fn normalize_header(&self, header: &str) -> Result<String, DateError>;
}

/// Mistral-backed date normalizer.
///
/// Kept under the historical type name to avoid API churn for callers.
#[derive(Debug)]
pub struct RuleBasedDateNormalizer {
    client: Client,
    api_key: Option<String>,
    model: String,
    cache: Mutex<HashMap<String, String>>,
}

impl RuleBasedDateNormalizer {
    /// Create a date normalizer using environment-driven config.
    ///
    /// Reads:
    /// - `MISTRAL_API_KEY`
    /// - `MISTRAL_DATE_MODEL` (defaults to `mistral-large-latest`)
    #[must_use]
    #[cfg(not(target_arch = "wasm32"))]
    pub fn new() -> Self {
        let api_key = std::env::var("MISTRAL_API_KEY")
            .ok()
            .filter(|value| !value.trim().is_empty());
        let model = std::env::var("MISTRAL_DATE_MODEL")
            .ok()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| DEFAULT_MISTRAL_MODEL.to_string());

        Self::with_config(DateNormalizerConfig { api_key, model })
    }

    #[must_use]
    pub fn with_config(config: DateNormalizerConfig) -> Self {
        let api_key = config.api_key.and_then(|value| {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        });
        let model = if config.model.trim().is_empty() {
            DEFAULT_MISTRAL_MODEL.to_string()
        } else {
            config.model
        };

        Self::with_optional_key(api_key, model)
    }

    /// Create a date normalizer with explicit API key and model.
    #[must_use]
    pub fn with_model(api_key: impl Into<String>, model: impl Into<String>) -> Self {
        Self::with_config(DateNormalizerConfig {
            api_key: Some(api_key.into()),
            model: model.into(),
        })
    }

    #[cfg(all(test, not(target_arch = "wasm32")))]
    fn without_api_key() -> Self {
        Self::with_optional_key(None, DEFAULT_MISTRAL_MODEL.to_string())
    }

    fn with_optional_key(api_key: Option<String>, model: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            model,
            cache: Mutex::new(HashMap::new()),
        }
    }

    fn build_prompt(header: &str) -> String {
        DATE_PROMPT_TEMPLATE.replace("{header_text}", header)
    }

    pub fn is_valid_mm_yyyy(value: &str) -> bool {
        let mut parts = value.split('.');
        let Some(month_raw) = parts.next() else {
            return false;
        };
        let Some(year_raw) = parts.next() else {
            return false;
        };
        if parts.next().is_some() {
            return false;
        }
        if month_raw.len() != 2 || year_raw.len() != 4 {
            return false;
        }
        let Ok(month) = month_raw.parse::<u8>() else {
            return false;
        };
        year_raw.chars().all(|ch| ch.is_ascii_digit()) && (1..=12).contains(&month)
    }

    fn parse_model_output(model_output: &str) -> Option<String> {
        let first_line = model_output.lines().next().unwrap_or_default().trim();

        if first_line.eq_ignore_ascii_case("ERROR") {
            return None;
        }

        if Self::is_valid_mm_yyyy(first_line) {
            Some(first_line.to_string())
        } else {
            None
        }
    }

    fn read_content_from_message(message: &serde_json::Value) -> Option<String> {
        if let Some(content) = message.get("content").and_then(|value| value.as_str()) {
            return Some(content.to_string());
        }

        let content_parts = message.get("content")?.as_array()?;
        let joined = content_parts
            .iter()
            .filter_map(|part| part.get("text").and_then(|value| value.as_str()))
            .collect::<Vec<_>>()
            .join("\n");
        if joined.is_empty() {
            None
        } else {
            Some(joined)
        }
    }

    fn get_cached(&self, header: &str) -> Option<String> {
        self.cache
            .lock()
            .ok()
            .and_then(|cache| cache.get(header).cloned())
    }

    fn cache_value(&self, header: &str, normalized: &str) {
        if let Ok(mut cache) = self.cache.lock() {
            cache.insert(header.to_string(), normalized.to_string());
        }
    }

    async fn request_mistral(&self, prompt: String) -> Result<String, DateError> {
        let Some(api_key) = self.api_key.as_ref() else {
            return Err(DateError::Parse(
                "MISTRAL_API_KEY is not configured".to_string(),
            ));
        };

        let request = serde_json::json!({
            "model": self.model,
            "messages": [{"role": "user", "content": prompt}],
            "temperature": 0,
        });

        let response = self
            .client
            .post("https://api.mistral.ai/v1/chat/completions")
            .bearer_auth(api_key)
            .json(&request)
            .send()
            .await
            .map_err(|err| DateError::Parse(format!("Failed to call Mistral API: {err}")))?;

        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|err| DateError::Parse(format!("Failed to read Mistral response: {err}")))?;

        if !status.is_success() {
            return Err(DateError::Parse(format!(
                "Mistral API returned {}: {}",
                status, body
            )));
        }

        let parsed: serde_json::Value = serde_json::from_str(&body)
            .map_err(|err| DateError::Parse(format!("Failed to parse Mistral JSON: {err}")))?;
        let message = parsed
            .get("choices")
            .and_then(|value| value.as_array())
            .and_then(|choices| choices.first())
            .and_then(|choice| choice.get("message"))
            .ok_or_else(|| {
                DateError::Parse("Mistral response has no choices[0].message".to_string())
            })?;

        Self::read_content_from_message(message).ok_or_else(|| {
            DateError::Parse("Mistral response has empty message content".to_string())
        })
    }
}

impl Default for RuleBasedDateNormalizer {
    fn default() -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        {
            Self::new()
        }

        #[cfg(target_arch = "wasm32")]
        {
            Self::with_config(DateNormalizerConfig::default())
        }
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl DateNormalizer for RuleBasedDateNormalizer {
    async fn normalize_header(&self, header: &str) -> Result<String, DateError> {
        let header = header.trim();
        if header.is_empty() {
            return Ok(header.to_string());
        }

        if let Some(value) = self.get_cached(header) {
            return Ok(value);
        }

        if self.api_key.is_none() {
            self.cache_value(header, header);
            return Ok(header.to_string());
        }

        let prompt = Self::build_prompt(header);
        let mut last_error: Option<String> = None;

        for attempt in 1..=MAX_RETRIES {
            match self.request_mistral(prompt.clone()).await {
                Ok(model_output) => {
                    if let Some(normalized) = Self::parse_model_output(&model_output) {
                        tracing::debug!(
                            header = %header,
                            attempt,
                            normalized = %normalized,
                            "Date header normalized successfully"
                        );
                        self.cache_value(header, &normalized);
                        return Ok(normalized);
                    }
                    last_error = Some(format!(
                        "Model returned invalid output (not MM.YYYY): {:?}",
                        model_output.lines().next().unwrap_or_default()
                    ));
                    tracing::warn!(
                        header = %header,
                        attempt,
                        output = %model_output.lines().next().unwrap_or_default(),
                        "Date header normalization attempt produced invalid output"
                    );
                }
                Err(err) => {
                    last_error = Some(err.to_string());
                    tracing::warn!(
                        header = %header,
                        attempt,
                        error = %err,
                        "Date header normalization attempt failed"
                    );
                }
            }
        }

        tracing::warn!(
            header = %header,
            attempts = MAX_RETRIES,
            last_error = ?last_error,
            "All date header normalization attempts failed, returning None to signal fallback"
        );

        Ok(header.to_string())
    }
}

/// Stub date normalizer for testing.
///
/// Returns known mappings from a predefined map, falls back to raw header.
#[derive(Debug, Clone)]
pub struct StubDateNormalizer {
    mappings: HashMap<String, String>,
}

impl StubDateNormalizer {
    /// Create a new stub with empty mappings.
    #[must_use]
    pub fn new() -> Self {
        Self {
            mappings: HashMap::new(),
        }
    }

    /// Add a known mapping.
    #[must_use]
    pub fn with_mapping(mut self, input: &str, output: &str) -> Self {
        self.mappings.insert(input.to_string(), output.to_string());
        self
    }
}

impl Default for StubDateNormalizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl DateNormalizer for StubDateNormalizer {
    async fn normalize_header(&self, header: &str) -> Result<String, DateError> {
        Ok(self
            .mappings
            .get(header)
            .cloned()
            .unwrap_or_else(|| header.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn test_parse_model_output_valid() {
        assert_eq!(
            RuleBasedDateNormalizer::parse_model_output("12.2025"),
            Some("12.2025".to_string())
        );
        assert_eq!(
            RuleBasedDateNormalizer::parse_model_output("09.2024"),
            Some("09.2024".to_string())
        );
        assert_eq!(
            RuleBasedDateNormalizer::parse_model_output("01.2023"),
            Some("01.2023".to_string())
        );
    }

    #[test]
    fn test_parse_model_output_error() {
        assert_eq!(RuleBasedDateNormalizer::parse_model_output("ERROR"), None);
        assert_eq!(RuleBasedDateNormalizer::parse_model_output("error"), None);
        assert_eq!(RuleBasedDateNormalizer::parse_model_output("Error"), None);
    }

    #[test]
    fn test_parse_model_output_invalid() {
        assert_eq!(RuleBasedDateNormalizer::parse_model_output("2025"), None);
        assert_eq!(RuleBasedDateNormalizer::parse_model_output("12/2025"), None);
        assert_eq!(RuleBasedDateNormalizer::parse_model_output("12.25"), None);
        assert_eq!(RuleBasedDateNormalizer::parse_model_output("13.2025"), None);
        assert_eq!(RuleBasedDateNormalizer::parse_model_output(""), None);
    }

    #[test]
    fn test_parse_model_output_multiline() {
        assert_eq!(
            RuleBasedDateNormalizer::parse_model_output("12.2025\nexplanation"),
            Some("12.2025".to_string())
        );
        assert_eq!(
            RuleBasedDateNormalizer::parse_model_output("ERROR\nreason"),
            None
        );
    }

    #[test]
    fn test_parse_model_output_whitespace() {
        assert_eq!(
            RuleBasedDateNormalizer::parse_model_output("  12.2025  "),
            Some("12.2025".to_string())
        );
    }

    #[test]
    fn test_mm_yyyy_validation() {
        assert!(RuleBasedDateNormalizer::is_valid_mm_yyyy("01.2024"));
        assert!(RuleBasedDateNormalizer::is_valid_mm_yyyy("12.2030"));
        assert!(!RuleBasedDateNormalizer::is_valid_mm_yyyy("1.2024"));
        assert!(!RuleBasedDateNormalizer::is_valid_mm_yyyy("13.2024"));
        assert!(!RuleBasedDateNormalizer::is_valid_mm_yyyy("11/2024"));
    }

    #[test]
    fn test_response_to_header_value() {
        assert_eq!(RuleBasedDateNormalizer::parse_model_output("ERROR"), None);
        assert_eq!(
            RuleBasedDateNormalizer::parse_model_output("09.2024"),
            Some("09.2024".to_string())
        );
        assert_eq!(
            RuleBasedDateNormalizer::parse_model_output("September 2024"),
            None
        );
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[tokio::test]
    async fn test_without_api_key_returns_original_header() {
        let normalizer = RuleBasedDateNormalizer::without_api_key();
        let result = normalizer.normalize_header("1 октября 2024").await.unwrap();
        assert_eq!(result, "1 октября 2024");
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[tokio::test]
    async fn test_stub_normalizer_with_mapping() {
        let stub = StubDateNormalizer::new().with_mapping("test date", "12.2024");

        let result = stub.normalize_header("test date").await.unwrap();
        assert_eq!(result, "12.2024");
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[tokio::test]
    async fn test_stub_normalizer_fallback() {
        let stub = StubDateNormalizer::new();

        let result = stub.normalize_header("unknown").await.unwrap();
        assert_eq!(result, "unknown");
    }

    #[test]
    fn test_build_prompt_replaces_placeholder() {
        let prompt = RuleBasedDateNormalizer::build_prompt("test header");
        assert!(prompt.contains("test header"));
    }

    #[test]
    fn test_read_content_from_message_string() {
        let message = serde_json::json!({"content": "test output"});
        let result = RuleBasedDateNormalizer::read_content_from_message(&message);
        assert_eq!(result, Some("test output".to_string()));
    }

    #[test]
    fn test_read_content_from_message_array() {
        let message = serde_json::json!({
            "content": [
                {"type": "text", "text": "line 1"},
                {"type": "text", "text": "line 2"}
            ]
        });
        let result = RuleBasedDateNormalizer::read_content_from_message(&message);
        assert_eq!(result, Some("line 1\nline 2".to_string()));
    }

    #[test]
    fn test_read_content_from_message_empty() {
        let message = serde_json::json!({"content": []});
        let result = RuleBasedDateNormalizer::read_content_from_message(&message);
        assert!(result.is_none());
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[tokio::test]
    async fn test_empty_header_returns_empty() {
        let normalizer = RuleBasedDateNormalizer::without_api_key();
        let result = normalizer.normalize_header("").await.unwrap();
        assert_eq!(result, "");
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[tokio::test]
    async fn test_whitespace_header_trimmed() {
        let normalizer = RuleBasedDateNormalizer::without_api_key();
        let result = normalizer.normalize_header("  test  ").await.unwrap();
        assert_eq!(result, "test");
    }

    #[test]
    fn test_mm_yyyy_validation_edge_cases() {
        assert!(RuleBasedDateNormalizer::is_valid_mm_yyyy("01.2024"));
        assert!(RuleBasedDateNormalizer::is_valid_mm_yyyy("12.2024"));
        assert!(!RuleBasedDateNormalizer::is_valid_mm_yyyy("00.2024"));
        assert!(!RuleBasedDateNormalizer::is_valid_mm_yyyy("01.24"));
        assert!(!RuleBasedDateNormalizer::is_valid_mm_yyyy(""));
        assert!(!RuleBasedDateNormalizer::is_valid_mm_yyyy("01.2024.05"));
    }

    struct SequenceDateNormalizer {
        responses: Vec<Result<String, DateError>>,
        call_count: AtomicUsize,
    }

    impl SequenceDateNormalizer {
        fn new(responses: Vec<Result<String, DateError>>) -> Self {
            Self {
                responses,
                call_count: AtomicUsize::new(0),
            }
        }

        fn call_count(&self) -> usize {
            self.call_count.load(Ordering::SeqCst)
        }
    }

    #[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
    #[cfg_attr(not(target_arch = "wasm32"), async_trait)]
    impl DateNormalizer for SequenceDateNormalizer {
        async fn normalize_header(&self, _header: &str) -> Result<String, DateError> {
            let idx = self.call_count.fetch_add(1, Ordering::SeqCst);
            self.responses
                .get(idx)
                .cloned()
                .unwrap_or(Ok("fallback".to_string()))
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[tokio::test]
    async fn test_sequence_first_attempt_succeeds() {
        let normalizer = SequenceDateNormalizer::new(vec![Ok("12.2025".to_string())]);
        let result = normalizer.normalize_header("За 2025 г.").await.unwrap();
        assert_eq!(result, "12.2025");
        assert_eq!(normalizer.call_count(), 1);
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[tokio::test]
    async fn test_sequence_fails_then_succeeds() {
        let normalizer = SequenceDateNormalizer::new(vec![
            Err(DateError::Parse("fail".to_string())),
            Ok("09.2024".to_string()),
        ]);
        let result1 = normalizer.normalize_header("test").await;
        assert!(result1.is_err());
        assert_eq!(normalizer.call_count(), 1);

        let result2 = normalizer.normalize_header("test2").await;
        assert_eq!(result2.unwrap(), "09.2024");
        assert_eq!(normalizer.call_count(), 2);
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[tokio::test]
    async fn test_stub_year_only_headers() {
        let stub = StubDateNormalizer::new()
            .with_mapping("За 2025 г.", "12.2025")
            .with_mapping("За 2024 г.", "12.2024");

        assert_eq!(
            stub.normalize_header("За 2025 г.").await.unwrap(),
            "12.2025"
        );
        assert_eq!(
            stub.normalize_header("За 2024 г.").await.unwrap(),
            "12.2024"
        );
    }
}
