//! CLI contract types for machine-readable output.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Exit codes for the CLI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitCode {
    /// Successful execution.
    Success = 0,
    /// Usage error (invalid arguments).
    UsageError = 1,
    /// Pipeline error (processing failure).
    PipelineError = 2,
    /// Provider error (OCR/external service failure).
    ProviderError = 3,
}

/// Error codes for machine-readable failure reporting.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    /// No financial tables found in document.
    NoFinancialTablesFound,
    /// Unsupported table layout.
    UnsupportedLayout,
    /// Invalid header row.
    InvalidHeader,
    /// Table dimension validation failed.
    DimensionValidationFailed,
    /// OCR/external provider error.
    ProviderError,
    /// Data parsing error.
    ParseError,
}

/// Success contract emitted by CLI on successful processing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuccessContract {
    /// Output JSON file path.
    pub output_json: PathBuf,
    /// Output XLSX file path (if generated).
    pub output_xlsx: Option<PathBuf>,
    /// Document ID.
    pub document_id: String,
    /// Report type.
    pub report_type: String,
    /// Number of rows extracted.
    pub row_count: usize,
    /// Number of columns extracted.
    pub column_count: usize,
}

/// Failure contract emitted by CLI on processing failure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailureContract {
    /// Error code for machine-readable error handling.
    pub error_code: ErrorCode,
    /// Human-readable error message.
    pub error_message: String,
    /// Document ID (if available).
    pub document_id: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exit_code_values() {
        assert_eq!(ExitCode::Success as i32, 0);
        assert_eq!(ExitCode::UsageError as i32, 1);
        assert_eq!(ExitCode::PipelineError as i32, 2);
        assert_eq!(ExitCode::ProviderError as i32, 3);
    }

    #[test]
    fn test_success_contract_serialization() {
        let contract = SuccessContract {
            output_json: PathBuf::from("/tmp/output.json"),
            output_xlsx: Some(PathBuf::from("/tmp/output.xlsx")),
            document_id: "doc-123".to_string(),
            report_type: "balance_sheet".to_string(),
            row_count: 15,
            column_count: 4,
        };
        let json = serde_json::to_string(&contract).unwrap();
        assert!(json.contains("doc-123"));
        assert!(json.contains("balance_sheet"));
    }

    #[test]
    fn test_failure_contract_serialization() {
        let contract = FailureContract {
            error_code: ErrorCode::NoFinancialTablesFound,
            error_message: "No tables detected".to_string(),
            document_id: Some("doc-456".to_string()),
        };
        let json = serde_json::to_string(&contract).unwrap();
        assert!(json.contains("no_financial_tables_found"));
        assert!(json.contains("No tables detected"));
    }

    #[test]
    fn test_error_code_variants() {
        let codes = vec![
            ErrorCode::NoFinancialTablesFound,
            ErrorCode::UnsupportedLayout,
            ErrorCode::InvalidHeader,
            ErrorCode::DimensionValidationFailed,
            ErrorCode::ProviderError,
            ErrorCode::ParseError,
        ];
        for code in codes {
            let json = serde_json::to_string(&code).unwrap();
            assert!(!json.is_empty());
        }
    }
}
