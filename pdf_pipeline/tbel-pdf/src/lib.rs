//! TBel PDF Processing Library.
//!
//! PDF processing pipeline for Belarusian financial reports with OCR,
//! table extraction, and data normalization.
//!
//! ## Target Support
//!
//! - **Native (default)**: Full library + CLI support
//! - **wasm32 (milestone 1)**: Library-only compilation support
//!
//! For wasm32 targets, use `PdfInput::Bytes` or `PdfInput::Url` as input.
//! The `cli` feature is not supported on wasm32.
//!

// Compile-time guard: prevent CLI feature on wasm32 targets
#[cfg(all(feature = "cli", target_arch = "wasm32"))]
compile_error!(
    "The 'cli' feature is not supported on wasm32. Use the library target only for wasm builds."
);

pub mod cleaner;
pub mod date;
pub mod error;
pub mod markdown;
pub mod models;
pub mod normalization;
pub mod ocr;
pub mod pdf;
pub mod processing;
pub mod report_cleaning;
pub mod scraper;
pub mod table_extraction;
pub mod types;

#[cfg(target_arch = "wasm32")]
pub mod wasm_bridge;

#[cfg(not(target_arch = "wasm32"))]
pub mod contract;

#[cfg(not(target_arch = "wasm32"))]
pub mod commands;

pub use cleaner::{ExtractedData, FinancialRecord, Page, PdfDocument};
#[cfg(not(target_arch = "wasm32"))]
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
pub use processing::{
    ProcessingFacade, ProcessingFacadeBuilder, ProcessingOptions, ProcessingResult,
};
pub use report_cleaning::{
    clean_report_tables, clean_report_tables_with_normalizer, normalize_date_header,
    parse_belarusian_integer, CleanedTable,
};
pub use scraper::{extract_company_name, extract_financial_data, parse_document};
pub use table_extraction::{
    extract_table_candidates, extract_table_candidates_from_markdown, is_valid_financial_table,
};
pub use types::PdfError;
