# Knowledge Bundle Update Log

## 2026-06-25

* **Initialization**: Created OKF knowledge bundle for the repository root (`tokenbel-pdf`).
* **Creation**: Added [System architecture](architecture/system-architecture.md), [Dependency direction and invariants](architecture/dependency-invariants.md).
* **Creation**: Added [ProcessingFacade orchestration](pipeline/processing-facade.md), [OCR provider boundary](pipeline/ocr-provider-boundary.md), [Table extraction and validation](pipeline/table-extraction.md), [Date normalization](pipeline/date-normalization.md).
* **Creation**: Added [CLI JSON contract](contracts/cli-json-contract.md), [Domain models](contracts/domain-models.md).
* **Creation**: Added [Report types](domain/report-types.md), [Normalization rules](domain/normalization-rules.md).
* **Creation**: Added [Target and feature boundaries](build/target-feature-boundaries.md).
* **Note**: Concepts were grounded in observed source under `pdf_pipeline/tbel-pdf/src/` plus `ARCHITECTURE.md`, `README.md`, and `pdf_pipeline/docs/cli-contract.md`. Where documentation and code could diverge, executable code was treated as source of truth per `ARCHITECTURE.md` §6.

## 2026-08-25

* **Update**: Refreshed [Table extraction and validation](pipeline/table-extraction.md) — documented page-boundary continuation merging (`flush_table_rows` / `is_continuation_fragment` in `pdf_pipeline/tbel-pdf/src/table_extraction.rs`), which merges orphan OCR table blocks that start with code-like rows into the previous table instead of letting `is_valid_financial_table` drop them. Motivated by a real 2-page balance sheet (`examples/Buh-balans-za-2-kv.-2026-g.pdf`) whose rows 290/300 arrived at the top of page 2 as a header-less block.
* **Note**: Added real OCR fixture `pdf_pipeline/tests/fixtures/ocr/buh_balans_2kv_2026.md` with regression test `test_page_boundary_balance_sheet_keeps_totals_rows_290_and_300` (`pdf_pipeline/tbel-pdf/tests/pipeline.rs`).
* **Note**: `pdf_pipeline/tbel-pdf/src/commands/pipeline.rs` now maps local file inputs to `PdfInput::Path` (base64 data URL upload) instead of sending a bare path as `document_url`; CLI failures now surface on stderr.
