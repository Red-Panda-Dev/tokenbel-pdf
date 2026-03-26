//! OCR provider adapter for PDF text extraction.
//!
//! This module defines the trait boundary for OCR services, allowing
//! different providers (Mistral, Google, etc.) to be plugged in.
//! Test doubles enable offline unit testing without network calls.

use async_trait::async_trait;
use base64::Engine;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

use crate::models::{OcrOutput, PdfInput};

/// Additional image data from OCR processing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageData {
    /// Page number where image was found.
    pub page: u32,
    /// Base64-encoded image data.
    pub base64: String,
}

/// Error types for OCR providers.
#[derive(Debug, thiserror::Error)]
pub enum ProviderError {
    #[error("Network error: {0}")]
    Network(String),

    #[error("Provider API error: {0}")]
    Api(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Parse error: {0}")]
    Parse(String),
}

/// Trait for OCR text extraction providers.
///
/// Implementations handle the specifics of calling external OCR services
/// (Mistral, Google Cloud Vision, AWS Textract, etc.). The trait enables
/// dependency injection for testing with deterministic doubles.
#[async_trait]
pub trait OcrProvider: Send + Sync {
    /// Perform OCR on the given PDF input.
    ///
    /// # Errors
    /// Returns `ProviderError` if OCR processing fails.
    async fn acquire_ocr(&self, input: PdfInput) -> Result<OcrOutput, ProviderError>;
}

/// Deterministic test double for OCR providers.
///
/// Returns pre-configured responses based on document ID,
/// enabling offline unit testing without network calls.
#[derive(Debug, Clone)]
pub struct MockOcrProvider {
    responses: HashMap<String, OcrOutput>,
}

impl MockOcrProvider {
    /// Create a new mock provider with an empty response map.
    #[must_use]
    pub fn new() -> Self {
        Self {
            responses: HashMap::new(),
        }
    }

    /// Add a response for a specific document ID.
    #[must_use]
    pub fn with_response(mut self, document_id: &str, output: OcrOutput) -> Self {
        self.responses.insert(document_id.to_string(), output);
        self
    }

    /// Add a simple text response for a document ID.
    #[must_use]
    pub fn with_text(self, document_id: &str, markdown: &str) -> Self {
        let output = OcrOutput::new(markdown.to_string(), 1, document_id.to_string());
        self.with_response(document_id, output)
    }
}

impl Default for MockOcrProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl OcrProvider for MockOcrProvider {
    async fn acquire_ocr(&self, input: PdfInput) -> Result<OcrOutput, ProviderError> {
        let doc_id = input.document_id();
        self.responses.get(&doc_id).cloned().ok_or_else(|| {
            ProviderError::InvalidInput(format!("No mock response configured for: {}", doc_id))
        })
    }
}

/// Stub OCR provider that returns empty results.
///
/// Useful as a default implementation when OCR is not yet configured.
pub struct StubOcrProvider;

#[async_trait]
impl OcrProvider for StubOcrProvider {
    async fn acquire_ocr(&self, input: PdfInput) -> Result<OcrOutput, ProviderError> {
        let doc_id = input.document_id();
        Ok(OcrOutput::new(String::new(), 0, doc_id))
    }
}

/// Mistral OCR provider.
#[derive(Debug, Clone)]
pub struct MistralOcrProvider {
    client: reqwest::Client,
    api_key: String,
    model: String,
}

impl MistralOcrProvider {
    /// Create a Mistral OCR provider with default OCR model.
    #[must_use]
    pub fn new(api_key: impl Into<String>) -> Self {
        Self::with_model(api_key, "mistral-ocr-latest")
    }

    /// Create a Mistral OCR provider with an explicit model.
    #[must_use]
    pub fn with_model(api_key: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key: api_key.into(),
            model: model.into(),
        }
    }

    fn input_pdf_bytes(&self, input: &PdfInput) -> Result<Vec<u8>, ProviderError> {
        match input {
            PdfInput::Path { path, .. } => std::fs::read(path).map_err(|err| {
                ProviderError::InvalidInput(format!(
                    "Failed to read PDF from '{}': {}",
                    path.display(),
                    err
                ))
            }),
            PdfInput::Bytes { bytes, .. } => Ok(bytes.clone()),
            PdfInput::Url { .. } => Err(ProviderError::InvalidInput(
                "PdfInput::Url does not provide local bytes".to_string(),
            )),
        }
    }

    fn doc_name(input: &PdfInput) -> Option<String> {
        match input {
            PdfInput::Path { path, .. } => path
                .file_name()
                .and_then(|name| name.to_str())
                .map(ToOwned::to_owned),
            PdfInput::Bytes { filename, .. } => filename.clone(),
            PdfInput::Url {
                document_url,
                filename,
                ..
            } => filename.clone().or_else(|| {
                document_url
                    .rsplit('/')
                    .next()
                    .filter(|name| !name.is_empty())
                    .map(ToOwned::to_owned)
            }),
        }
    }

    fn parse_markdown(response: &serde_json::Value) -> Result<(String, usize), ProviderError> {
        let pages = response
            .get("pages")
            .and_then(|value| value.as_array())
            .ok_or_else(|| {
                ProviderError::Parse("Mistral OCR response has no 'pages' array".to_string())
            })?;

        let markdown = pages
            .iter()
            .map(|page| {
                let page_markdown = page
                    .get("markdown")
                    .and_then(|value| value.as_str())
                    .unwrap_or_default();

                let table_markdown = page
                    .get("tables")
                    .and_then(|value| value.as_array())
                    .map(|tables| {
                        tables
                            .iter()
                            .filter_map(|table| {
                                table
                                    .get("content")
                                    .and_then(|value| value.as_str())
                                    .map(ToOwned::to_owned)
                            })
                            .collect::<Vec<_>>()
                            .join("\n\n")
                    })
                    .unwrap_or_default();

                if table_markdown.is_empty() {
                    page_markdown.to_string()
                } else if page_markdown.is_empty() {
                    table_markdown
                } else {
                    format!("{}\n\n{}", page_markdown, table_markdown)
                }
            })
            .collect::<Vec<_>>()
            .join("\n\n");

        if markdown.trim().is_empty() {
            return Err(ProviderError::Parse(
                "Mistral OCR response pages contain no markdown".to_string(),
            ));
        }

        Ok((markdown, pages.len()))
    }

    fn data_url_for_pdf(path: &Path, bytes: &[u8]) -> String {
        let b64 = base64::engine::general_purpose::STANDARD.encode(bytes);
        if path.extension().and_then(|ext| ext.to_str()) == Some("png") {
            return format!("data:image/png;base64,{}", b64);
        }
        format!("data:application/pdf;base64,{}", b64)
    }
}

#[async_trait]
impl OcrProvider for MistralOcrProvider {
    async fn acquire_ocr(&self, input: PdfInput) -> Result<OcrOutput, ProviderError> {
        let doc_id = input.document_id();
        let document_url = match &input {
            PdfInput::Url { document_url, .. } => document_url.clone(),
            PdfInput::Path { .. } | PdfInput::Bytes { .. } => {
                let bytes = self.input_pdf_bytes(&input)?;
                let path = match &input {
                    PdfInput::Path { path, .. } => path.clone(),
                    PdfInput::Bytes { filename, .. } => {
                        Path::new(filename.as_deref().unwrap_or("input.pdf")).to_path_buf()
                    }
                    PdfInput::Url { .. } => unreachable!("handled above"),
                };

                std::env::var("TBEL_OCR_DOCUMENT_URL")
                    .ok()
                    .filter(|url| !url.trim().is_empty())
                    .unwrap_or_else(|| Self::data_url_for_pdf(&path, &bytes))
            }
        };

        let mut request = serde_json::json!({
            "model": self.model,
            "document": {
                "type": "document_url",
                "document_url": document_url,
            },
            "include_image_base64": false,
            "table_format": "markdown",
        });

        if let Some(document_name) = Self::doc_name(&input) {
            request["document"]["document_name"] = serde_json::Value::String(document_name);
        }

        let response = self
            .client
            .post("https://api.mistral.ai/v1/ocr")
            .bearer_auth(&self.api_key)
            .json(&request)
            .send()
            .await
            .map_err(|err| ProviderError::Network(err.to_string()))?;

        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|err| ProviderError::Network(err.to_string()))?;

        if !status.is_success() {
            return Err(ProviderError::Api(format!(
                "Mistral OCR API returned {}: {}",
                status, body
            )));
        }

        let parsed: serde_json::Value =
            serde_json::from_str(&body).map_err(|err| ProviderError::Parse(err.to_string()))?;
        let (markdown, pages) = Self::parse_markdown(&parsed)?;
        Ok(OcrOutput::new(markdown, pages, doc_id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_mock_ocr_provider_returns_configured_response() {
        let mock =
            MockOcrProvider::new().with_text("doc1", "# Test Document\n\nSome content here.");

        let input = PdfInput::Path {
            path: PathBuf::from("/test.pdf"),
            document_id: Some("doc1".to_string()),
        };
        let result = mock.acquire_ocr(input).await;

        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.markdown.contains("Test Document"));
        assert_eq!(output.page_count, 1);
    }

    #[tokio::test]
    async fn test_mock_ocr_provider_returns_error_for_unknown_document() {
        let mock = MockOcrProvider::new();

        let input = PdfInput::Path {
            path: PathBuf::from("/test.pdf"),
            document_id: Some("unknown_doc".to_string()),
        };
        let result = mock.acquire_ocr(input).await;

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("No mock response configured"));
    }

    #[tokio::test]
    async fn test_stub_ocr_provider_returns_empty() {
        let stub = StubOcrProvider;

        let input = PdfInput::Path {
            path: PathBuf::from("/test.pdf"),
            document_id: Some("any_doc".to_string()),
        };
        let result = stub.acquire_ocr(input).await;

        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.markdown.is_empty());
        assert_eq!(output.page_count, 0);
    }

    #[test]
    fn test_pdf_input_document_id() {
        let input = PdfInput::Path {
            path: PathBuf::from("/test.pdf"),
            document_id: Some("doc-123".to_string()),
        };
        assert_eq!(input.document_id(), "doc-123");
    }

    #[test]
    fn test_ocr_output_from_markdown() {
        let output = OcrOutput::new(
            "Page 1 content\n\nPage 2 content".to_string(),
            2,
            "test-doc".to_string(),
        );

        assert_eq!(output.page_count, 2);
        assert!(output.markdown.contains("Page 1"));
        assert!(output.markdown.contains("Page 2"));
    }

    #[test]
    fn test_mistral_provider_default_model() {
        let provider = MistralOcrProvider::new("test-key");
        assert_eq!(provider.model, "mistral-ocr-latest");
    }

    #[test]
    fn test_mistral_provider_with_model() {
        let provider = MistralOcrProvider::with_model("test-key", "custom-model");
        assert_eq!(provider.model, "custom-model");
    }

    #[test]
    fn test_parse_markdown_with_pages() {
        let response = serde_json::json!({
            "pages": [
                {"markdown": "# Page 1"},
                {"markdown": "# Page 2"}
            ]
        });
        let result = MistralOcrProvider::parse_markdown(&response);
        assert!(result.is_ok());
        let (markdown, pages) = result.unwrap();
        assert_eq!(pages, 2);
        assert!(markdown.contains("Page 1"));
        assert!(markdown.contains("Page 2"));
    }

    #[test]
    fn test_parse_markdown_with_tables() {
        let response = serde_json::json!({
            "pages": [
                {
                    "markdown": "",
                    "tables": [
                        {"content": "| Col1 | Col2 |\n| A | B |"}
                    ]
                }
            ]
        });
        let result = MistralOcrProvider::parse_markdown(&response);
        assert!(result.is_ok());
        let (markdown, _) = result.unwrap();
        assert!(markdown.contains("Col1"));
    }

    #[test]
    fn test_parse_markdown_empty_pages() {
        let response = serde_json::json!({"pages": []});
        let result = MistralOcrProvider::parse_markdown(&response);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_markdown_missing_pages() {
        let response = serde_json::json!({});
        let result = MistralOcrProvider::parse_markdown(&response);
        assert!(result.is_err());
    }

    #[test]
    fn test_doc_name_extraction_url() {
        let input = PdfInput::Url {
            document_url: "https://example.com/report_2024.pdf".to_string(),
            document_id: None,
            filename: None,
        };
        let name = MistralOcrProvider::doc_name(&input);
        assert_eq!(name, Some("report_2024.pdf".to_string()));
    }

    #[test]
    fn test_doc_name_extraction_path() {
        let input = PdfInput::Path {
            path: PathBuf::from("/data/balance_sheet.pdf"),
            document_id: None,
        };
        let name = MistralOcrProvider::doc_name(&input);
        assert_eq!(name, Some("balance_sheet.pdf".to_string()));
    }

    #[test]
    fn test_doc_name_extraction_bytes_with_filename() {
        let input = PdfInput::Bytes {
            bytes: vec![],
            document_id: None,
            filename: Some("custom.pdf".to_string()),
        };
        let name = MistralOcrProvider::doc_name(&input);
        assert_eq!(name, Some("custom.pdf".to_string()));
    }

    #[test]
    fn test_data_url_for_pdf() {
        let path = PathBuf::from("test.pdf");
        let bytes = b"test content";
        let url = MistralOcrProvider::data_url_for_pdf(&path, bytes);
        assert!(url.starts_with("data:application/pdf;base64,"));
    }

    #[test]
    fn test_data_url_for_png() {
        let path = PathBuf::from("test.png");
        let bytes = b"test content";
        let url = MistralOcrProvider::data_url_for_pdf(&path, bytes);
        assert!(url.starts_with("data:image/png;base64,"));
    }

    #[test]
    fn test_provider_error_display() {
        let err = ProviderError::Network("connection failed".to_string());
        assert!(err.to_string().contains("Network error"));

        let err = ProviderError::Api("rate limited".to_string());
        assert!(err.to_string().contains("Provider API error"));

        let err = ProviderError::InvalidInput("bad file".to_string());
        assert!(err.to_string().contains("Invalid input"));

        let err = ProviderError::Parse("json error".to_string());
        assert!(err.to_string().contains("Parse error"));
    }
}
