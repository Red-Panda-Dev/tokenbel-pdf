---
type: Process
title: Table extraction and validation
description: Parsing markdown tables into ReportTable candidates and filtering financial ones.
resource: pdf_pipeline/tbel-pdf/src/table_extraction.rs
source_paths:
  - pdf_pipeline/tbel-pdf/src/table_extraction.rs
  - pdf_pipeline/tbel-pdf/src/report_cleaning.rs
  - pdf_pipeline/tbel-pdf/src/lib.rs
  - pdf_pipeline/README.md
confidence: observed
---

# Table extraction and validation

Two related stages sit between OCR markdown and the cleaned business output:
candidate extraction/validation (`src/table_extraction.rs`) and business-level
cleaning (`src/report_cleaning.rs`) [1][2][3].

## Candidate extraction

`extract_table_candidates_from_markdown` parses preprocessed markdown into
`ReportTable` candidates; `is_valid_financial_table` filters for tables that look
like financial statements (headers, code-like rows, dimensions) [1][4]. Both are
re-exported from `src/lib.rs` [3].

If no candidates survive, `ProcessingFacade` returns
`PipelineError::NoFinancialTablesFound` (see
[ProcessingFacade](/pipeline/processing-facade.md)).

## Business cleaning

`src/report_cleaning.rs` turns a `ProcessingResult` into `CleanedTable` values
(headers + string rows) ready for XLSX/JSON export [2][4]. Observed helpers [2]:

* `clean_report_tables` / `clean_report_tables_with_normalizer` - entry points.
* `align_code_column` / `is_code_column` / `code_digits` - detect and align the
  numeric line-code column (2-3 ASCII digits).
* `remove_blank_columns` - drop OCR-introduced empty columns.
* `normalize_date_header` - month-name fallback mapping to `MM.YYYY`.
* `parse_belarusian_integer` - normalize numeric values.
* `prepare_tables_debug` / `PreparedTableDebug` - intermediate debug shape.

Date header normalization can delegate to the
[date normalization](/pipeline/date-normalization.md) adapter; the rule-based
`normalize_date_header` provides an offline fallback [2].

## Relationships

* Consumes output of [ProcessingFacade orchestration](/pipeline/processing-facade.md).
* Uses [date normalization](/pipeline/date-normalization.md) for headers.
* Output feeds the [CLI contract](/contracts/cli-json-contract.md) XLSX/JSON
  export and conforms to [normalization rules](/domain/normalization-rules.md).

# Citations

[1] `pdf_pipeline/tbel-pdf/src/table_extraction.rs` — candidate extraction and validation.
[2] `pdf_pipeline/tbel-pdf/src/report_cleaning.rs` — `CleanedTable` and cleaning helpers.
[3] `pdf_pipeline/tbel-pdf/src/lib.rs` — public re-exports.
[4] `pdf_pipeline/README.md` — pipeline stage descriptions and stage artifacts.
