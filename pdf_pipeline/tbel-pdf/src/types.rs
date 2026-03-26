//! PDF error types.

use serde::{Deserialize, Serialize};
use std::io;
use thiserror::Error;

/// PDF processing error types.
#[derive(Debug, Error, Serialize, Deserialize)]
pub enum PdfError {
    /// IO error.
    #[error("IO error: {0}")]
    Io(String),

    /// Parse error.
    #[error("Parse error: {0}")]
    Parse(String),

    /// Extraction error.
    #[error("Extraction error: {0}")]
    Extraction(String),
}

impl From<io::Error> for PdfError {
    fn from(err: io::Error) -> Self {
        Self::Io(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pdf_error_io() {
        let err = PdfError::Io("file not found".to_string());
        assert!(err.to_string().contains("IO error"));
    }

    #[test]
    fn test_pdf_error_parse() {
        let err = PdfError::Parse("invalid format".to_string());
        assert!(err.to_string().contains("Parse error"));
    }

    #[test]
    fn test_pdf_error_extraction() {
        let err = PdfError::Extraction("no tables found".to_string());
        assert!(err.to_string().contains("Extraction error"));
    }
}
