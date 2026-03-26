//! OCR output model.

use serde::{Deserialize, Serialize};

/// Output from OCR processing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcrOutput {
    /// Extracted markdown content.
    pub markdown: String,
    /// Number of pages processed.
    pub page_count: usize,
    /// Document identifier.
    pub document_id: String,
}

impl OcrOutput {
    /// Create a new OCR output.
    #[must_use]
    pub fn new(markdown: String, page_count: usize, document_id: String) -> Self {
        Self {
            markdown,
            page_count,
            document_id,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ocr_output_new() {
        let output = OcrOutput::new("Test content".to_string(), 3, "doc-123".to_string());
        assert_eq!(output.markdown, "Test content");
        assert_eq!(output.page_count, 3);
        assert_eq!(output.document_id, "doc-123");
    }
}
