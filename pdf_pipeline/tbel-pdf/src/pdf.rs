//! PDF input adapter using lopdf for reading PDF files.

#[cfg(not(target_arch = "wasm32"))]
use std::path::Path;

#[cfg(not(target_arch = "wasm32"))]
use crate::error::PipelineError;
use crate::models::PdfInput;

/// PDF reader for loading PDF files into the pipeline.
pub struct PdfReader;

impl PdfReader {
    /// Create a PdfInput from a file path.
    ///
    /// Reads the file from disk and wraps it in a PdfInput::Bytes variant.
    ///
    /// # Errors
    ///
    /// Returns an IoError if the file cannot be read.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<PdfInput, PipelineError> {
        let path = path.as_ref();
        let bytes = std::fs::read(path).map_err(|e| PipelineError::IoError {
            operation: "read".to_string(),
            path: path.display().to_string(),
            message: e.to_string(),
        })?;
        let document_id = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();
        Ok(PdfInput::Bytes {
            bytes,
            document_id: Some(document_id),
            filename: path.file_name().map(|s| s.to_string_lossy().to_string()),
        })
    }

    /// Create a PdfInput from raw bytes.
    ///
    /// # Arguments
    ///
    /// * `bytes` - Raw PDF file bytes
    /// * `document_id` - Optional document identifier for logging/tracing
    ///
    /// # Returns
    ///
    /// A PdfInput::Bytes variant containing the raw bytes.
    #[must_use]
    pub fn from_bytes(bytes: Vec<u8>, document_id: String) -> PdfInput {
        PdfInput::Bytes {
            bytes,
            document_id: Some(document_id),
            filename: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_bytes() {
        let input = PdfReader::from_bytes(vec![1, 2, 3], "test".to_string());
        assert_eq!(input.document_id(), "test");
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_from_path_unknown() {
        // Test with a non-existent path - should return error
        let result = PdfReader::from_path("/nonexistent/path.pdf");
        assert!(result.is_err());
    }
}
