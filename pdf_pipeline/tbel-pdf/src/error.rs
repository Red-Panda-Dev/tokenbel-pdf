//! Pipeline error types.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Result type for pipeline operations.
pub type Result<T> = std::result::Result<T, PipelineError>;

/// Pipeline error types.
#[derive(Debug, Error, Serialize, Deserialize)]
pub enum PipelineError {
    /// IO error.
    #[error("IO error during {operation} on '{path}': {message}")]
    IoError {
        operation: String,
        path: String,
        message: String,
    },

    /// No financial tables found.
    #[error("No financial tables found in document")]
    NoFinancialTablesFound,

    /// Unsupported table layout.
    #[error("Unsupported table layout: {0}")]
    UnsupportedLayout(String),

    /// Invalid header row.
    #[error("Invalid header row: {0}")]
    InvalidHeader(String),

    /// Table dimension validation failed.
    #[error("Table dimension validation failed: {0}")]
    DimensionValidationFailed(String),

    /// Provider error.
    #[error("Provider error: {0}")]
    ProviderError(String),

    /// Parse error.
    #[error("Parse error: {0}")]
    ParseError(String),

    /// Export error.
    #[error("Export error: {0}")]
    ExportError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_error_display() {
        let err = PipelineError::NoFinancialTablesFound;
        assert!(err.to_string().contains("No financial tables"));
    }

    #[test]
    fn test_pipeline_error_io_error() {
        let err = PipelineError::IoError {
            operation: "read".to_string(),
            path: "/test.pdf".to_string(),
            message: "permission denied".to_string(),
        };
        assert!(err.to_string().contains("read"));
        assert!(err.to_string().contains("/test.pdf"));
    }

    #[test]
    fn test_pipeline_error_unsupported_layout() {
        let err = PipelineError::UnsupportedLayout("merged cells".to_string());
        assert!(err.to_string().contains("merged cells"));
    }
}
