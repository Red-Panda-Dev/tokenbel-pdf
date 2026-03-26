//! Domain models for PDF financial report processing.

mod report_table;
mod pdf_input;
mod ocr_output;
mod code_value;
mod cleaned_report;
mod report_type;

pub use report_table::{ReportTable, TableCell};
pub use pdf_input::PdfInput;
pub use ocr_output::OcrOutput;
pub use code_value::CodeValue;
pub use cleaned_report::{CleanedReport, DataColumn};
pub use report_type::ReportType;
