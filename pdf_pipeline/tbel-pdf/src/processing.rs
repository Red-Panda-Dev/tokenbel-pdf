//! Processing facade for PDF financial report extraction.
//!
//! Provides a target-neutral processing pipeline that works on both native
//! and wasm32 targets. This is the shared Rust processing logic used by both
//! the native CLI and the wasm JavaScript bridge.
//!
//! ## Processing Flow
//!
//! 1. **Input**: Accept PDF bytes or URL via `PdfInput`
//! 2. **OCR**: Use an `OcrProvider` to extract markdown from PDF
//! 3. **Preprocess**: Clean LaTeX, normalize markdown structure
//! 4. **Extract**: Parse markdown tables into `ReportTable` structs
//! 5. **Validate**: Filter for valid financial tables
//! 6. **Output**: Return structured `ProcessingResult`

use crate::error::PipelineError;
use crate::markdown::{clean_latex_from_markdown, preprocess_markdown};
use crate::models::{OcrOutput, PdfInput, ReportTable, ReportType};
use crate::ocr::OcrProvider;
use crate::table_extraction::{extract_table_candidates_from_markdown, is_valid_financial_table};

/// Result of successful PDF processing.
#[derive(Debug, Clone)]
pub struct ProcessingResult {
    /// Document identifier.
    pub document_id: String,
    /// Detected or specified report type.
    pub report_type: ReportType,
    /// Extracted financial tables.
    pub tables: Vec<ReportTable>,
    /// Number of pages processed.
    pub page_count: usize,
}

/// Processing options for fine-grained control.
#[derive(Debug, Clone, Default)]
pub struct ProcessingOptions {
    /// Report type (auto-detected if None).
    pub report_type: Option<ReportType>,
    /// Maximum tables to return (default: all valid).
    pub max_tables: Option<usize>,
}

/// Processing facade for PDF financial report extraction.
///
/// Provides a unified processing interface that works on both native and wasm32.
/// This struct is cheap to clone and can be shared across requests.
#[derive(Debug, Clone)]
pub struct ProcessingFacade {
    options: ProcessingOptions,
}

impl ProcessingFacade {
    /// Create a new processing facade with default options.
    #[must_use]
    pub fn new() -> Self {
        Self {
            options: ProcessingOptions::default(),
        }
    }

    /// Create a processing facade with custom options.
    #[must_use]
    pub fn with_options(options: ProcessingOptions) -> Self {
        Self { options }
    }

    /// Process PDF input and extract financial tables.
    ///
    /// # Arguments
    ///
    /// * `input` - PDF input source (bytes or URL)
    /// * `ocr` - OCR provider for PDF text extraction
    ///
    /// # Returns
    ///
    /// Processing result containing extracted tables and metadata
    ///
    /// # Errors
    ///
    /// Returns `PipelineError` if processing fails
    pub async fn process(
        &self,
        input: PdfInput,
        ocr: &dyn OcrProvider,
    ) -> Result<ProcessingResult, PipelineError> {
        let document_id = input.document_id();

        // Step 1: OCR extraction
        let ocr_output = ocr
            .acquire_ocr(input)
            .await
            .map_err(|e| PipelineError::ProviderError(e.to_string()))?;

        self.process_ocr_output(ocr_output, document_id)
    }

    /// Process pre-extracted OCR markdown output.
    ///
    /// Useful when OCR has already been performed externally (e.g., in wasm
    /// where we receive markdown directly from JavaScript).
    ///
    /// # Arguments
    ///
    /// * `markdown` - Markdown content from OCR
    /// * `page_count` - Number of pages processed
    /// * `document_id` - Document identifier
    ///
    /// # Returns
    ///
    /// Processing result containing extracted tables and metadata
    pub fn process_markdown(
        &self,
        markdown: &str,
        page_count: usize,
        document_id: String,
    ) -> Result<ProcessingResult, PipelineError> {
        let doc_id = document_id.clone();
        let ocr_output = OcrOutput::new(markdown.to_string(), page_count, document_id);
        self.process_ocr_output(ocr_output, doc_id)
    }

    fn process_ocr_output(
        &self,
        ocr_output: OcrOutput,
        document_id: String,
    ) -> Result<ProcessingResult, PipelineError> {
        // Step 2: Preprocess markdown
        let cleaned = clean_latex_from_markdown(&ocr_output.markdown);
        let preprocessed = preprocess_markdown(&cleaned);

        // Step 3: Extract table candidates
        let candidates = extract_table_candidates_from_markdown(&preprocessed);

        if candidates.is_empty() {
            return Err(PipelineError::NoFinancialTablesFound);
        }

        // Step 4: Filter valid financial tables
        let valid_tables: Vec<ReportTable> = candidates
            .into_iter()
            .filter(is_valid_financial_table)
            .collect();

        if valid_tables.is_empty() {
            return Err(PipelineError::NoFinancialTablesFound);
        }

        // Step 5: Limit results if specified
        let tables = match self.options.max_tables {
            Some(max) => valid_tables.into_iter().take(max).collect(),
            None => valid_tables,
        };

        // Step 6: Determine report type
        let report_type = self
            .options
            .report_type
            .or_else(|| ReportType::try_from_filename(&document_id))
            .unwrap_or(ReportType::BalanceSheet);

        Ok(ProcessingResult {
            document_id,
            report_type,
            tables,
            page_count: ocr_output.page_count,
        })
    }
}

impl Default for ProcessingFacade {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for constructing ProcessingFacade with fluent API.
#[derive(Debug, Default)]
pub struct ProcessingFacadeBuilder {
    options: ProcessingOptions,
}

impl ProcessingFacadeBuilder {
    #[must_use]
    pub fn report_type(mut self, report_type: ReportType) -> Self {
        self.options.report_type = Some(report_type);
        self
    }

    #[must_use]
    pub fn max_tables(mut self, max: usize) -> Self {
        self.options.max_tables = Some(max);
        self
    }

    #[must_use]
    pub fn build(self) -> ProcessingFacade {
        ProcessingFacade::with_options(self.options)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(not(target_arch = "wasm32"))]
    use crate::ocr::StubOcrProvider;

    fn make_test_markdown() -> &'static str {
        r#"| Код строки | Наименование показателей | 2024 | 2023 |
| --- | --- | --- | --- |
| 010 | Основные средства | 1 000 | 900 |
| 020 | Нематериальные активы | 500 | 400 |
| 030 | Вложения в долгосрочные активы | 300 | 200 |
| 040 | Долгосрочная дебиторская задолженность | 200 | 150 |
| 050 | ИТОГО по разделу I | 2 000 | 1 650 |
| 060 | Запасы | 800 | 700 |
| 070 | Налог на добавленную стоимость | 100 | 80 |
| 080 | Денежные средства | 600 | 500 |
| 090 | ИТОГО по разделу II | 1 500 | 1 280 |
| 100 | БАЛАНС | 3 500 | 2 930 |"#
    }

    #[test]
    fn test_process_facade_process_markdown() {
        let facade = ProcessingFacade::new();
        let markdown = make_test_markdown();

        let result = facade.process_markdown(markdown, 1, "test-doc".to_string());

        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.document_id, "test-doc");
        assert_eq!(result.page_count, 1);
        assert!(!result.tables.is_empty());
    }

    #[test]
    fn test_process_facade_with_options() {
        let facade = ProcessingFacadeBuilder::default()
            .report_type(ReportType::BalanceSheet)
            .max_tables(1)
            .build();

        let markdown = make_test_markdown();
        let result = facade.process_markdown(markdown, 1, "test".to_string());

        assert!(result.is_ok());
        assert_eq!(result.unwrap().tables.len(), 1);
    }

    #[test]
    fn test_process_facade_no_tables() {
        let facade = ProcessingFacade::new();
        let markdown = "| Col1 | Col2 |\n| A | B |";

        let result = facade.process_markdown(markdown, 1, "empty".to_string());

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            PipelineError::NoFinancialTablesFound
        ));
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[tokio::test]
    async fn test_process_with_ocr_provider() {
        let facade = ProcessingFacade::new();
        let ocr = StubOcrProvider;

        let input = PdfInput::from_bytes(vec![], "async-test", None);
        let result = facade.process(input, &ocr).await;

        // Stub provider returns empty, so we expect no tables
        assert!(result.is_err());
    }

    #[test]
    fn test_process_result_table_count() {
        let facade = ProcessingFacade::new();
        let markdown = make_test_markdown();

        let result = facade
            .process_markdown(markdown, 1, "count-test".to_string())
            .unwrap();

        // Should find exactly one table
        assert_eq!(result.tables.len(), 1);
        assert_eq!(result.tables[0].row_count(), 10);
    }
}
