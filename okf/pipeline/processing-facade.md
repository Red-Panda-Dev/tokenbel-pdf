---
type: Service
title: ProcessingFacade orchestration
description: Shared, target-neutral orchestration for OCR markdown to extracted financial tables.
resource: pdf_pipeline/tbel-pdf/src/processing.rs
source_paths:
  - pdf_pipeline/tbel-pdf/src/processing.rs
  - pdf_pipeline/tbel-pdf/src/lib.rs
confidence: observed
---

# ProcessingFacade orchestration

`ProcessingFacade` (`src/processing.rs`) is the central orchestration boundary
used by both the native CLI and the wasm bridge [1]. It owns the pipeline that
turns OCR markdown into a validated `ProcessingResult`.

## Processing flow

Documented and observed in source [1]:

1. **Input** - accept PDF bytes/path/URL via `PdfInput`, or pre-extracted markdown.
2. **OCR** - use an `OcrProvider` to obtain `OcrOutput` (full path only).
3. **Preprocess** - clean LaTeX and normalize markdown structure.
4. **Extract** - parse markdown tables into `ReportTable` candidates.
5. **Validate** - filter for valid financial tables; error if none survive.
6. **Output** - return `ProcessingResult`.

## Entry points

* `process(input: PdfInput, ocr: &dyn OcrProvider)` - full path: acquires OCR
  through the [OCR provider boundary](/pipeline/ocr-provider-boundary.md), then
  runs the shared pipeline [1].
* `process_markdown(markdown, page_count, document_id)` - reuses the parsing
  path when OCR markdown is already available (e.g. wasm callers receiving
  markdown from JavaScript) [1].

`ProcessingFacadeBuilder` / `ProcessingOptions` configure `max_tables` and an
optional explicit `report_type`; otherwise report type is inferred from the
document id filename, falling back to `BalanceSheet` [1]. See
[report types](/domain/report-types.md).

## Result type

`ProcessingResult` carries `document_id`, `report_type`, extracted `tables`
(`Vec<ReportTable>`), `page_count`, and `preprocessed_markdown` [1]. It is
consumed by [report cleaning](/pipeline/table-extraction.md) and the
[CLI contract](/contracts/cli-json-contract.md).

## Relationships

* Depends on [OCR provider boundary](/pipeline/ocr-provider-boundary.md).
* Feeds [table extraction and validation](/pipeline/table-extraction.md).
* Errors surface as `PipelineError` (see [domain models](/contracts/domain-models.md)).

# Citations

[1] `pdf_pipeline/tbel-pdf/src/processing.rs` — `ProcessingFacade`, `process`, `process_markdown`, `ProcessingResult`.
