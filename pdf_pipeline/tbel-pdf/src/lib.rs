//! TBel PDF Processing Library.
//!
//! PDF processing pipeline for Belarusian financial reports with OCR,
//! table extraction, and data normalization.

pub mod cleaner;
pub mod contract;
pub mod date;
pub mod error;
pub mod markdown;
pub mod models;
pub mod normalization;
pub mod ocr;
pub mod pdf;
pub mod scraper;
pub mod table_extraction;
pub mod types;

pub use cleaner::{ExtractedData, FinancialRecord, Page, PdfDocument};
pub use contract::{ExitCode, FailureContract, SuccessContract};
pub use date::{DateError, DateNormalizer, RuleBasedDateNormalizer, StubDateNormalizer};
pub use error::{PipelineError, Result};
pub use markdown::{clean_latex_from_markdown, preprocess_markdown};
pub use models::{
    CleanedReport, CodeValue, DataColumn, OcrOutput, PdfInput, ReportTable, ReportType, TableCell,
};
pub use ocr::{
    ImageData, MistralOcrProvider, MockOcrProvider, OcrProvider, ProviderError, StubOcrProvider,
};
pub use pdf::PdfReader;
pub use scraper::{extract_company_name, extract_financial_data, parse_document};
pub use table_extraction::{
    extract_table_candidates, extract_table_candidates_from_markdown, is_valid_financial_table,
};
pub use types::PdfError;
