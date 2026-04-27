# Architecture

## 1. High-Level Overview

This repository is a Rust library and CLI for extracting structured financial tables from Belarusian PDF reports via OCR. The system takes PDF documents (by URL, path, or bytes), sends them through the Mistral OCR API, preprocesses the resulting markdown, extracts and validates financial tables, cleans and normalizes data into a three-column schema (code, period, period), and produces structured JSON and XLSX output. The library also compiles to wasm32 for browser and edge deployment via `wasm_bindgen`.

**Observed identity and purpose:**
- Single Cargo workspace with one crate `tbel-pdf` (`pdf_pipeline/Cargo.toml`, `pdf_pipeline/tbel-pdf/Cargo.toml`)
- Dual compilation targets: native (CLI + library) and wasm32 (library-only) (`pdf_pipeline/tbel-pdf/Cargo.toml:15-20`)
- Target domain: Belarusian financial statements — BalanceSheet, IncomeStatement, StatementCashFlow, StatementEquityChanges (`pdf_pipeline/tbel-pdf/src/models/report_type.rs`)
- External OCR dependency: Mistral OCR API, abstracted behind the `OcrProvider` trait (`pdf_pipeline/tbel-pdf/src/ocr.rs`)

**Evidence anchors:**
- `pdf_pipeline/Cargo.toml` — workspace manifest, single member `tbel-pdf`
- `pdf_pipeline/tbel-pdf/Cargo.toml` — crate manifest with `cli` feature, `cdylib` + `rlib` crate types
- `pdf_pipeline/tbel-pdf/src/bin/tbel-pdf.rs` — CLI binary entrypoint
- `pdf_pipeline/tbel-pdf/src/lib.rs` — library public API with compile-time wasm32 guard
- `pdf_pipeline/tbel-pdf/src/wasm_bridge.rs` — `wasm_bindgen` exports for JS interop
- `pdf_pipeline/rust-toolchain.toml` — pinned toolchain 1.94.0

## 2. System Architecture (Logical)

```
┌──────────────────────────────────────────────────────────────────┐
│                      CLI Layer (feature-gated)                   │
│  clap arg parsing, subcommand dispatch, XLSX export, exit codes  │
│  src/bin/tbel-pdf.rs, src/commands/                              │
└──────────────────────────┬───────────────────────────────────────┘
                            │
                            ▼
┌──────────────────────────────────────────────────────────────────┐
│                       Contract Layer                             │
│  SuccessContract, FailureContract, ExitCode, ErrorCode           │
│  src/contract/ (native only)                                     │
└──────────────────────────┬───────────────────────────────────────┘
                            │
         ┌──────────────────┴──────────────────┐
         ▼                                     ▼
┌─────────────────────┐       ┌────────────────────────────────┐
│  wasm_bridge        │       │  ProcessingFacade              │
│  (wasm32 only)      │       │  (shared entry: native + wasm) │
│  src/wasm_bridge.rs │       │  src/processing.rs             │
└─────────┬───────────┘       └───────────┬────────────────────┘
          │                               │
          └───────────┬───────────────────┘
                      ▼
┌──────────────────────────────────────────────────────────────────┐
│                     Processing Pipeline                          │
│  markdown → table_extraction → validation                        │
│  src/markdown.rs, src/table_extraction.rs, src/cleaner.rs,      │
│  src/normalization.rs, src/date.rs                               │
└──────────────────────────┬───────────────────────────────────────┘
                            │
          ┌─────────────────┴─────────────────┐
          ▼                                   ▼
┌──────────────────────┐        ┌──────────────────────────────┐
│   Domain Models      │        │       Adapters               │
│   (pure, no I/O)     │        │  OcrProvider trait,          │
│   ReportTable,       │◄───────│  MistralOcrProvider,         │
│   PdfInput,          │        │  PdfReader, Scraper          │
│   OcrOutput,         │        │  src/ocr.rs, pdf.rs,         │
│   ReportType         │        │  scraper.rs, date.rs         │
└──────────────────────┘        └──────────────────────────────┘
                            │
                            ▼
               ┌──────────────────────────────┐
               │   Report Cleaning            │
               │   (library module)           │
               │   clean_report_tables()      │
               │   src/report_cleaning.rs     │
               └──────────────────────────────┘
```

**Components:**

1. **CLI Layer** — Argument parsing via clap, subcommand dispatch to `pipeline`, report type inference from filename, XLSX file generation, stage artifact emission, exit code mapping. Gated behind the `cli` feature and blocked on wasm32 by `compile_error!`. (`src/bin/`, `src/commands/`)

2. **wasm Bridge** — JavaScript interop via `wasm_bindgen`. Exposes `process_markdown`, `process_pdf`, `validate_markdown`, and `get_supported_report_types`. Includes its own XLSX export path for wasm consumers. (`src/wasm_bridge.rs`, wasm32 only)

3. **Contract Layer** — Typed JSON output schemas (`SuccessContract`, `FailureContract`) and exit codes for CLI consumers. Native only. (`src/contract/`)

4. **ProcessingFacade** — The shared orchestration entry point used by both the CLI and wasm bridge. Accepts `PdfInput` + `OcrProvider`, runs: OCR → markdown preprocessing → table extraction → validation. Returns `ProcessingResult`. (`src/processing.rs`)

5. **Report Cleaning** — Library-level business logic that transforms raw `ProcessingResult` tables into cleaned three-column output (`CleanedTable`). Handles header alignment, blank column removal, code-column detection, Belarusian number parsing, and date header normalization. Shared between CLI XLSX export and integration tests. (`src/report_cleaning.rs`)

6. **Processing Pipeline** — Stateless transformation stages: markdown cleaning/LaTeX removal, table candidate extraction from HTML/markdown, financial table validation, data cleaning (Belarusian number formats), normalization, date normalization. (`src/markdown.rs`, `src/table_extraction.rs`, `src/cleaner.rs`, `src/normalization.rs`, `src/date.rs`)

7. **Domain Models** — Pure data types: `ReportTable`, `TableCell`, `PdfInput` (Path | Bytes | Url), `OcrOutput`, `ReportType`, `CleanedReport`, `CodeValue`. Zero I/O imports. (`src/models/`)

8. **Adapters** — Trait-based external service boundaries: `OcrProvider` (Mistral, Mock, Stub), `DateNormalizer`, `PdfReader`, HTML scraper. Real implementations live in top-level modules; `src/adapters/mod.rs` only re-exports for backwards compatibility. (`src/ocr.rs`, `src/pdf.rs`, `src/scraper.rs`, `src/date.rs`)

**Dependency direction:**
- CLI → Contract → ProcessingFacade + ReportCleaning → Pipeline + Adapters → Models
- wasm_bridge → ProcessingFacade + ReportCleaning → Pipeline + Adapters → Models
- Models have zero dependency on adapters, I/O, or pipeline logic
- CLI and wasm_bridge are mutually exclusive at compile time (feature gates + target arch)

**What is intentionally NOT depended upon:**
- Models do not import `reqwest`, `tokio`, `chrono::Local`, `scraper`, `lopdf`, or any filesystem API
- Library code (non-`cli` feature) does not depend on `clap`, `tracing-subscriber`, `rust_xlsxwriter`, or `tokio`
- `report_cleaning.rs` does not import `clap`, `rust_xlsxwriter`, or any CLI-only dependency
- wasm32 builds exclude `chrono`, `scraper`, `lopdf` (cfg-gated in `Cargo.toml`)

## 3. Code Map (Physical)

```
tokenbel-pdf/
├── AGENTS.md                    # Root agent guide, routes into pdf_pipeline/AGENTS.md
├── ARCHITECTURE.md              # This file
├── README.md                    # Russian-language repository overview
└── pdf_pipeline/                # Cargo workspace root — all Rust code lives here
    ├── Cargo.toml               # Workspace manifest (single member: tbel-pdf)
    ├── Cargo.lock               # Locked dependencies
    ├── rust-toolchain.toml      # Pinned: 1.94.0, components: rustfmt, clippy
    ├── ci-check.sh              # CI matrix: native + wasm32 + optional coverage
    ├── coverage.sh              # Library coverage gate (cargo-llvm-cov, 70% threshold)
    ├── README.md                # Operational reference (CLI usage, env vars, report types)
    ├── AGENTS.md                # Authoritative Rust-specific guidance
    ├── docs/
    │   └── cli-contract.md      # CLI JSON contract specification
    ├── tests/                   # Workspace-level test fixtures
    │   ├── *.pdf                # Sample financial reports (3 files)
    │   ├── fixtures/
    │   │   ├── manifest.json    # Test fixture definitions
    │   │   ├── ocr/             # Committed real OCR markdown (3 files)
    │   │   └── source_of_truth/ # Reference data for regression checks
    │   ├── golden/              # Regression golden files (10 JSON+XLSX pairs)
    │   └── output/              # Test output artifacts (gitignored)
    └── tbel-pdf/                # The unified crate: library + CLI + wasm bridge
        ├── Cargo.toml           # Features: default=[], cli; crate-type cdylib+rlib
        ├── src/
        │   ├── lib.rs           # Public API re-exports, compile_error! guard
        │   ├── processing.rs    # ProcessingFacade — shared orchestration
        │   ├── report_cleaning.rs # Business-level table cleaning (CleanedTable, helpers)
        │   ├── wasm_bridge.rs   # wasm_bindgen exports (wasm32 only)
        │   ├── bin/
        │   │   └── tbel-pdf.rs  # CLI binary entrypoint (feature-gated)
        │   ├── commands/        # CLI subcommand dispatch (cli feature-gated)
        │   │   ├── mod.rs       # clap App/Commands definitions
        │   │   └── pipeline.rs  # Full pipeline: OCR → clean → XLSX export
        │   ├── contract/        # ExitCode, ErrorCode, SuccessContract, FailureContract
        │   ├── models/          # Pure domain types — no I/O
        │   │   ├── mod.rs       # Re-exports all model types
        │   │   ├── report_table.rs   # ReportTable, TableCell
        │   │   ├── report_type.rs    # ReportType enum (4 variants)
        │   │   ├── pdf_input.rs      # PdfInput (Path | Bytes | Url)
        │   │   ├── ocr_output.rs     # OcrOutput
        │   │   ├── cleaned_report.rs # CleanedReport, DataColumn
        │   │   └── code_value.rs     # CodeValue
        │   ├── adapters/        # Re-exports from top-level modules (backwards compat)
        │   │   └── mod.rs       # pub use crate::{ocr, pdf, date, ...}
        │   ├── ocr.rs           # OcrProvider trait, MistralOcrProvider, Mock/Stub
        │   ├── pdf.rs           # PdfReader
        │   ├── scraper.rs       # HTML parsing, company name extraction
        │   ├── date.rs          # DateNormalizer trait, RuleBasedDateNormalizer
        │   ├── markdown.rs      # Markdown preprocessing, LaTeX cleaning, table merging
        │   ├── table_extraction.rs  # Table candidate extraction, financial table validation
        │   ├── cleaner.rs       # DataFrameCleaner, Belarusian number format parsing
        │   ├── normalization.rs # Table structure normalization
        │   ├── error.rs         # PipelineError, Result
        │   └── types.rs         # PdfError
        ├── prompts/             # Mistral prompt templates
        └── tests/
            ├── pipeline.rs      # Integration tests (real OCR fixture tests)
            └── worker_smoke.mjs # Node.js wasm smoke test runner
```

**Where is X?**
- OCR provider implementations → `pdf_pipeline/tbel-pdf/src/ocr.rs`
- wasm JS interop → `pdf_pipeline/tbel-pdf/src/wasm_bridge.rs`
- Shared processing orchestration → `pdf_pipeline/tbel-pdf/src/processing.rs`
- Table cleaning and normalization → `pdf_pipeline/tbel-pdf/src/report_cleaning.rs`
- Table extraction logic → `pdf_pipeline/tbel-pdf/src/table_extraction.rs`
- Date normalization → `pdf_pipeline/tbel-pdf/src/date.rs`
- CLI argument handling and pipeline command → `pdf_pipeline/tbel-pdf/src/commands/`
- JSON output schemas → `pdf_pipeline/tbel-pdf/src/contract/mod.rs`
- Domain types → `pdf_pipeline/tbel-pdf/src/models/`
- Test fixtures → `pdf_pipeline/tests/fixtures/`
- Golden regression files → `pdf_pipeline/tests/golden/`
- CI verification script → `pdf_pipeline/ci-check.sh`

## 4. Life of a Request / Primary Data Flow

### Native CLI path

```
CLI args (--input-url, --report-type, --emit-stage-artifacts)
    │
    ▼
src/bin/tbel-pdf.rs
  - Parse clap args via App::parse()
  - Initialize tracing subscriber
  - Dispatch to Commands::Pipeline
    │
    ▼
src/commands/pipeline.rs — execute()
  1. Resolve ReportType (from --report-type flag or URL filename heuristic)
  2. Read MISTRAL_API_KEY env var
  3. Construct MistralOcrProvider
  4. Call ocr_provider.acquire_ocr(input) → OcrOutput
    │
    ▼
ProcessingFacade::process_markdown(ocr_output.markdown)
  5. clean_latex_from_markdown — remove LaTeX artifacts
  6. preprocess_markdown — normalize structure, merge tables
  7. extract_table_candidates_from_markdown — parse markdown tables
  8. filter is_valid_financial_table — keep only valid financial tables
  → ProcessingResult { document_id, report_type, tables, page_count }
    │
    ▼
clean_report_tables(&result) → Vec<CleanedTable>
  9. Per-table: find header row, remove blank columns, align code column
  10. Extract original date headers, normalize to MM.YYYY format
  11. Filter rows to 2-3 digit numeric codes only
  12. Parse Belarusian number formats (space thousands, comma decimals, parenthesized negatives)
  → CleanedTable { headers: [code, MM.YYYY, MM.YYYY], rows }
    │
    ▼
tables_to_xlsx() → <stem>_output.xlsx
  13. Merge all cleaned tables into one continuous sheet with single header row
  14. Optional: write stage artifacts (ocr_output.md, tables.json, meta.json)
  15. Emit SuccessContract JSON to stdout / --emit-contract file
```

**Status:** Fully `Observed` in `src/commands/pipeline.rs` and `src/processing.rs`.

### wasm bridge path

```
JS caller → wasm_bridge exports (process_markdown | process_pdf)
    │
    ▼
wasm_bridge.rs: decode_request → parse_report_type
    │
    ├─ process_markdown path:
    │    ProcessingFacade::process_markdown(markdown, page_count, doc_id)
    │      → clean_latex → preprocess → extract → validate
    │
    └─ process_pdf path:
         ProcessingFacade::process(PdfInput, &dyn OcrProvider)
           → ocr.acquire_ocr(input)  [Mistral API call]
           → process_ocr_output (same pipeline as above)
    │
    ▼
ProcessingResult → success_from_result()
  → Optional XLSX export (rust_xlsxwriter with wasm feature)
  → JsSuccess { document_id, report_type, tables, xlsx? }
    │
    ▼
serde_wasm_bindgen → JsValue → Promise resolved to JS caller
```

**Status:** Fully `Observed` in `src/wasm_bridge.rs` and `src/processing.rs`.

### Processing pipeline stages (shared by both paths)

```
OcrOutput (markdown)
    │
    ▼ 1. clean_latex_from_markdown — remove LaTeX artifacts
    ▼ 2. preprocess_markdown — normalize structure, merge tables
    ▼ 3. extract_table_candidates_from_markdown — parse markdown tables
    ▼ 4. is_valid_financial_table — filter (min 3 cols × 10 rows)
    ▼ 5. ReportType detection (filename heuristic or explicit)
    │
    ▼
ProcessingResult { document_id, report_type, tables, page_count }
```

## 5. Architectural Invariants & Constraints

- **Rule:** Domain models in `src/models/` must not perform I/O
  - **Rationale:** Enables pure unit testing and clear separation of data contracts from infrastructure.
  - **Enforcement / Signals (Observed):** Models only import `serde`, `chrono::NaiveDate`, and std; no `reqwest`, `tokio`, `scraper`, `lopdf`, or filesystem imports. All model submodules are declared `mod` (private) with `pub use` re-exports in `models/mod.rs`.

- **Rule:** CLI feature is gated and blocked on wasm32
  - **Rationale:** The crate compiles as both a native binary and a wasm32 library; CLI dependencies are unnecessary and unresolvable on wasm32.
  - **Enforcement / Signals (Observed):** `compile_error!` guard in `src/lib.rs:16-19`; `cli` feature in `Cargo.toml:20` with optional native-only dependencies; `[[bin]]` entry requires `cli` feature.

- **Rule:** ProcessingFacade is the single shared orchestration entry point
  - **Rationale:** Prevents duplicated pipeline logic between CLI and wasm bridge.
  - **Enforcement / Signals (Observed):** Both `src/wasm_bridge.rs` and `src/commands/pipeline.rs` use `ProcessingFacade`/`ProcessingFacadeBuilder` for table extraction.

- **Rule:** All external HTTP calls go through adapter traits
  - **Rationale:** Enables mocking, provider swapping, and offline testing without changing business logic.
  - **Enforcement / Signals (Observed):** `OcrProvider` trait in `src/ocr.rs` with `async fn acquire_ocr`; `MockOcrProvider` and `StubOcrProvider` test doubles defined alongside production `MistralOcrProvider`.

- **Rule:** Report cleaning is a library module with no CLI-only dependencies
  - **Rationale:** `report_cleaning.rs` must be usable from integration tests without enabling the `cli` feature.
  - **Enforcement / Signals (Observed):** `src/report_cleaning.rs` imports only `crate::processing::ProcessingResult` and `crate::models::*`; no `clap`, `rust_xlsxwriter`, or filesystem imports. Re-exported from `lib.rs`.

- **Rule:** OCR and date normalizer must have offline test doubles
  - **Rationale:** Enables CI and local testing without `MISTRAL_API_KEY` or network access.
  - **Enforcement / Signals (Observed):** `MockOcrProvider`, `StubOcrProvider` in `src/ocr.rs`; `StubDateNormalizer` in `src/date.rs`.

- **Rule:** Exit codes are standardized and stable
  - **Rationale:** Enables reliable scripting and CI integration.
  - **Enforcement / Signals (Observed):** `ExitCode` enum in `src/contract/mod.rs` with values 0–3; documented in `docs/cli-contract.md`.

- **Rule:** Rust toolchain is pinned to exact version 1.94.0
  - **Rationale:** Reproducible builds across environments.
  - **Enforcement / Signals (Observed):** `pdf_pipeline/rust-toolchain.toml` with `channel = "1.94.0"` and `rust-version = "1.94"` in crate manifest.

- **Rule:** Crate must not be split into multiple crates
  - **Rationale:** Unified crate design simplifies feature gating, cross-target compilation, and dependency management.
  - **Enforcement / Signals (Inferred):** Stated in `AGENTS.md` change rules; workspace has single member `tbel-pdf`.

- **Rule:** JSON contract schema changes must update contract tests and documentation
  - **Rationale:** External callers depend on stable output format.
  - **Enforcement / Signals (Observed):** `SuccessContract` and `FailureContract` have serialization round-trip tests in `src/contract/mod.rs`; `docs/cli-contract.md` defines the stable schema.

- **Rule:** wasm32 builds exclude native-only dependencies
  - **Rationale:** `chrono`, `scraper`, `lopdf`, and `tokio` cannot compile or are unnecessary on wasm32.
  - **Enforcement / Signals (Observed):** `Cargo.toml` uses `cfg(not(target_arch = "wasm32"))` and `cfg(target_arch = "wasm32")` for dependency and module gating.

- **Rule:** `src/adapters/mod.rs` only re-exports; canonical implementations are top-level modules
  - **Rationale:** Historical backwards compatibility; the real adapter code lives in `src/ocr.rs`, `src/pdf.rs`, etc. Orphaned submodule files exist in `src/adapters/` but are not compiled.
  - **Enforcement / Signals (Observed):** `src/adapters/mod.rs` contains only `pub use crate::...` statements with no `mod` declarations, so sibling files are not compiled.

## 6. Documentation Strategy

**Hierarchy:**

1. **`ARCHITECTURE.md`** (this file) — Global map, logical components, invariants, and high-level data flow. The authoritative source for architectural decisions and boundaries.

2. **`pdf_pipeline/AGENTS.md`** — Authoritative Rust-specific guidance: module layout, local boundaries, safe change rules, validation commands, and nearby doc references. Supersedes root `AGENTS.md` for Rust work.

3. **`pdf_pipeline/README.md`** — Operational reference: CLI usage, environment variables, supported report types, error codes, and development workflow.

4. **`pdf_pipeline/docs/cli-contract.md`** — Stable JSON contract specification for machine-readable CLI output.

5. **Module-level docs** — Each major module has rustdoc comments explaining purpose and key exports (observable in `src/lib.rs`, `src/processing.rs`, `src/ocr.rs`, `src/wasm_bridge.rs`).

**What belongs where:**
- Global architecture, invariants, and cross-boundary rules → `ARCHITECTURE.md`
- Rust build/lint/test commands, module boundaries, safe change procedures → `pdf_pipeline/AGENTS.md`
- CLI usage, configuration, and operational details → `pdf_pipeline/README.md`
- JSON schema specification → `pdf_pipeline/docs/cli-contract.md`
- Module internals and API docs → inline rustdoc comments
