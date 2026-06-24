---
type: Architecture
title: System architecture
description: Logical layers and data flow of the TokenBel PDF processing pipeline.
source_paths:
  - ARCHITECTURE.md
  - pdf_pipeline/tbel-pdf/src/lib.rs
  - pdf_pipeline/tbel-pdf/src/processing.rs
  - pdf_pipeline/tbel-pdf/src/bin/tbel-pdf.rs
  - pdf_pipeline/tbel-pdf/src/wasm_bridge.rs
confidence: observed
---

# System architecture

The repository is a single Cargo workspace under `pdf_pipeline/` with one crate,
`tbel-pdf`, that builds as a Rust library (`rlib`), a feature-gated native CLI,
and a wasm-compatible library artifact (`cdylib`) [1][2].

## Logical layers

```text
Native CLI                wasm bridge
src/bin/, src/commands/   src/wasm_bridge.rs
        |                         |
        v                         v
CLI contract/export       JS-facing serialization/export
src/contract/             wasm-bindgen types
        \_________________________/
                    |
                    v
Shared orchestration
src/processing.rs: ProcessingFacade
                    |
                    v
Pipeline transformations
src/markdown.rs, src/table_extraction.rs, src/report_cleaning.rs,
src/date.rs, src/normalization.rs, src/cleaner.rs
                    |
                    v
External boundaries + pure data
src/ocr.rs, src/pdf.rs, src/scraper.rs, src/models/
```

(Adapted from [1].)

## Primary data flow

Native CLI path [1]:

```text
CLI invocation
  -> src/bin/tbel-pdf.rs
  -> src/commands/mod.rs
  -> src/commands/pipeline.rs
  -> MistralOcrProvider through OcrProvider in src/ocr.rs
  -> ProcessingFacade::process_markdown in src/processing.rs
  -> markdown preprocessing in src/markdown.rs
  -> table extraction/validation in src/table_extraction.rs
  -> business cleaning/date normalization in src/report_cleaning.rs and src/date.rs
  -> XLSX output and SuccessContract JSON from src/commands/pipeline.rs
```

Library/wasm path [1]: a Rust or JS caller reaches the public API in `src/lib.rs`
or the wasm exports in `src/wasm_bridge.rs`, then `ProcessingFacade::process`
(full OCR path) or `ProcessingFacade::process_markdown` (pre-extracted markdown).
The split lets wasm/integration callers reuse the parsing path when OCR markdown
is already available.

## Orchestration boundary

[ProcessingFacade](/pipeline/processing-facade.md) is the central orchestration
boundary shared by both entrypoints; CLI/wasm code must route shared extraction
behavior through it rather than duplicating stages [1]. It depends on the
[OCR provider boundary](/pipeline/ocr-provider-boundary.md) and produces a
`ProcessingResult` consumed by [report cleaning](/pipeline/table-extraction.md)
and the [CLI contract](/contracts/cli-json-contract.md).

## Notes

* Not a server: there is no persistent database, queue, or long-running request
  lifecycle described in the repository. The CLI path is request-like but writes
  XLSX plus a JSON contract to disk [1].
* Live OCR requires `MISTRAL_API_KEY` [1][3].

# Citations

[1] `ARCHITECTURE.md` — System architecture, code map, and data flow.
[2] `pdf_pipeline/tbel-pdf/src/lib.rs` — Public module surface and target/feature gates.
[3] `pdf_pipeline/tbel-pdf/src/commands/pipeline.rs` — CLI argument handling and env lookup.
