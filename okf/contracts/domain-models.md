---
type: Schema
title: Domain models
description: Pure data types shared across native, wasm, and test targets.
resource: pdf_pipeline/tbel-pdf/src/models/
source_paths:
  - pdf_pipeline/tbel-pdf/src/models/mod.rs
  - pdf_pipeline/tbel-pdf/src/models/report_type.rs
  - pdf_pipeline/tbel-pdf/src/models/report_table.rs
  - pdf_pipeline/tbel-pdf/src/models/pdf_input.rs
  - pdf_pipeline/tbel-pdf/src/models/ocr_output.rs
  - pdf_pipeline/tbel-pdf/src/models/cleaned_report.rs
  - pdf_pipeline/tbel-pdf/src/models/code_value.rs
  - pdf_pipeline/tbel-pdf/src/error.rs
confidence: observed
---

# Domain models

`src/models/` is the pure data vocabulary shared across targets and tests. It
must stay domain-data-only: no HTTP, filesystem, Tokio, scraper, PDF-reader,
CLI, or wasm-bridge logic [1]. See
[dependency invariants](/architecture/dependency-invariants.md).

## Re-exported types

From `src/models/mod.rs` [1]: `CleanedReport`, `DataColumn`, `CodeValue`,
`OcrOutput`, `PdfInput`, `ReportTable`, `TableCell`, `ReportType`.

## Key types

| Type | Purpose | Source |
|------|---------|--------|
| `ReportType` | Enum of supported report kinds | [2] |
| `PdfInput` | Input source: `Path`, `Bytes`, `Url` | [4] |
| `OcrOutput` | `{ markdown, page_count, document_id }` | [5] |
| `ReportTable` | `{ headers, rows: Vec<Vec<TableCell>>, table_index }` | [3] |
| `TableCell` | `{ content, row_index, col_index }` | [3] |
| `CleanedReport` | `{ report_type, columns: Vec<DataColumn>, source_path }` | [6] |
| `DataColumn` | `{ header, values }` | [6] |
| `CodeValue` | `{ code, name: Option, row_index }` | [7] |

`ReportType` is detailed in [report types](/domain/report-types.md).

## Pipeline errors

`PipelineError` (`src/error.rs`) is the crate-wide error enum [8]:

* `IoError { operation, path, message }`
* `NoFinancialTablesFound`
* `UnsupportedLayout(String)`
* `InvalidHeader(String)`
* `DimensionValidationFailed(String)`
* `ProviderError(String)`
* `ParseError(String)`
* `ExportError(String)`

These map to the [CLI JSON contract](/contracts/cli-json-contract.md) failure
codes and exit codes.

## Relationships

* Consumed by [ProcessingFacade](/pipeline/processing-facade.md) and
  [table extraction](/pipeline/table-extraction.md).
* `ReportType` drives [report types](/domain/report-types.md).

# Citations

[1] `pdf_pipeline/tbel-pdf/src/models/mod.rs` — module surface and re-exports.
[2] `pdf_pipeline/tbel-pdf/src/models/report_type.rs` — `ReportType` enum and descriptors.
[3] `pdf_pipeline/tbel-pdf/src/models/report_table.rs` — `ReportTable`, `TableCell`.
[4] `pdf_pipeline/tbel-pdf/src/models/pdf_input.rs` — `PdfInput` variants.
[5] `pdf_pipeline/tbel-pdf/src/models/ocr_output.rs` — `OcrOutput`.
[6] `pdf_pipeline/tbel-pdf/src/models/cleaned_report.rs` — `CleanedReport`, `DataColumn`.
[7] `pdf_pipeline/tbel-pdf/src/models/code_value.rs` — `CodeValue`.
[8] `pdf_pipeline/tbel-pdf/src/error.rs` — `PipelineError`.
