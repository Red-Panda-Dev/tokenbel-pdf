# Architecture

## 1. High-Level Overview

This repository is a Rust CLI/library for extracting financial tables from Belarusian PDF reports using OCR. The primary purpose is to automate processing of standardized financial statements (Баланс, Отчёт о прибылях и убытках, etc.) into structured XLSX/JSON output.

**Observed identity and purpose:**
- Single Cargo workspace with one crate `tbel-pdf` (`pdf_pipeline/Cargo.toml`, `pdf_pipeline/tbel-pdf/Cargo.toml`)
- CLI binary gated behind `cli` feature flag (`pdf_pipeline/tbel-pdf/Cargo.toml:15-17`)
- Target domain: Belarusian financial reports with OCR, table extraction, and data normalization (`pdf_pipeline/README.md:1-14`)
- Supported report types: BalanceSheet, IncomeStatement, StatementCashFlow, StatementEquityChanges (`pdf_pipeline/tbel-pdf/src/models/report_type.rs`)

**Evidence anchors:**
- `pdf_pipeline/Cargo.toml` — workspace manifest
- `pdf_pipeline/tbel-pdf/Cargo.toml` — crate manifest with CLI feature
- `pdf_pipeline/tbel-pdf/src/bin/tbel-pdf.rs` — CLI entrypoint
- `pdf_pipeline/tbel-pdf/src/lib.rs` — library public API
- `pdf_pipeline/README.md` — operational documentation
- `pdf_pipeline/rust-toolchain.toml` — pinned toolchain 1.94.0

## 2. System Architecture (Logical)

```
┌─────────────────────────────────────────────────────────────────┐
│                         CLI Layer                                │
│  (feature-gated, clap, exit codes, JSON contract)               │
│  pdf_pipeline/tbel-pdf/src/{bin, commands}/                     │
└─────────────────────────┬───────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────────┐
│                       Contract Layer                             │
│  (SuccessContract, FailureContract, ErrorCode, ExitCode)        │
│  pdf_pipeline/tbel-pdf/src/contract/                            │
└─────────────────────────┬───────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────────┐
│                     Processing Pipeline                          │
│  (table_extraction, markdown, cleaner, normalization, date)     │
│  pdf_pipeline/tbel-pdf/src/{table_extraction,markdown,etc}.rs   │
└─────────────────────────┬───────────────────────────────────────┘
                          │
          ┌───────────────┴───────────────┐
          ▼                               ▼
┌─────────────────────┐       ┌─────────────────────────┐
│    Domain Models    │       │      Adapters           │
│  (pure, no I/O)     │       │  (OcrProvider trait,    │
│  ReportTable,       │       │   PdfReader, HTTP)      │
│  PdfInput,          │       │  ocr.rs, pdf.rs,        │
│  OcrOutput,         │       │  scraper.rs, date.rs    │
│  CleanedReport      │       │                         │
└─────────────────────┘       └─────────────────────────┘
          ▲                               │
          │                               │
          └───────────────────────────────┘
                    (models used by adapters)
```

**Components:**

1. **CLI Layer** — Argument parsing via clap, subcommand dispatch, exit code mapping. Feature-gated so the library can be used without CLI dependencies. (`src/bin/`, `src/commands/`)

2. **Contract Layer** — Typed JSON output schemas (`SuccessContract`, `FailureContract`) and exit codes. Provides stable machine-readable interface for callers. (`src/contract/`)

3. **Processing Pipeline** — Core transformation stages: markdown preprocessing, table candidate extraction, data cleaning (Belarusian number formats), and normalization. (`src/table_extraction.rs`, `src/markdown.rs`, `src/cleaner.rs`, `src/normalization.rs`, `src/date.rs`)

4. **Domain Models** — Pure data types representing inputs, outputs, and intermediate structures. No I/O, no external dependencies beyond serde/chrono. (`src/models/`)

5. **Adapters** — Trait-based boundaries for external services: `OcrProvider` (Mistral, Mock, Stub), `PdfReader`, HTML scraping. (`src/ocr.rs`, `src/pdf.rs`, `src/scraper.rs`)

**Dependency direction:**
- CLI → Contract → Processing Pipeline → Models + Adapters
- Models have no dependencies on adapters or I/O
- Adapters depend on models, not vice versa

**What is intentionally NOT depended upon:**
- Models do not depend on `reqwest`, `tokio`, or any I/O crates
- Library code (non-CLI) does not depend on `clap`, `anyhow`, or `tracing-subscriber`
- Test doubles (`MockOcrProvider`, `StubDateNormalizer`) have no external network calls

## 3. Code Map (Physical)

```
tokenbel-pdf/
├── AGENTS.md                    # Repository-level guidance (note: references stale rust/ paths)
├── README.md                    # Repository overview (Russian)
├── pdf_pipeline/                # Cargo workspace root
│   ├── Cargo.toml               # Workspace manifest, single member: tbel-pdf
│   ├── Cargo.lock               # Locked dependencies
│   ├── rust-toolchain.toml      # Pinned: 1.94.0 with rustfmt, clippy
│   ├── README.md                # Detailed operational docs, report types, CLI usage
│   ├── AGENTS.md                # Rust-specific guidance (note: references stale rust/ paths)
│   ├── docs/
│   │   └── cli-contract.md      # CLI JSON contract specification
│   ├── prompts/
│   │   └── financial_date_extraction.txt  # Mistral prompt template for date normalization
│   ├── tests/                   # Integration test fixtures
│   │   ├── *.pdf                # Sample financial reports
│   │   ├── fixtures/            # Test manifests and source-of-truth data
│   │   ├── golden/              # Regression golden files
│   │   └── output/              # Test output artifacts
│   └── tbel-pdf/                # Single crate: library + CLI
│       ├── Cargo.toml           # Crate manifest, features: default=[], cli
│       ├── src/
│       │   ├── lib.rs           # Public API re-exports
│       │   ├── bin/
│       │   │   └── tbel-pdf.rs  # CLI binary entrypoint
│       │   ├── commands/
│       │   │   ├── mod.rs       # clap App/Commands definitions
│       │   │   └── pipeline.rs  # Pipeline command implementation
│       │   ├── contract/
│       │   │   └── mod.rs       # ExitCode, ErrorCode, SuccessContract, FailureContract
│       │   ├── models/
│       │   │   ├── mod.rs
│       │   │   ├── report_table.rs    # ReportTable, TableCell
│       │   │   ├── pdf_input.rs       # PdfInput
│       │   │   ├── ocr_output.rs      # OcrOutput
│       │   │   ├── cleaned_report.rs  # CleanedReport, DataColumn
│       │   │   ├── code_value.rs      # CodeValue
│       │   │   └── report_type.rs     # ReportType enum
│       │   ├── adapters/
│       │   │   └── mod.rs       # Re-exports from parent modules
│       │   ├── ocr.rs           # OcrProvider trait, MistralOcrProvider, MockOcrProvider
│       │   ├── pdf.rs           # PdfReader
│       │   ├── scraper.rs       # HTML parsing, company name extraction
│       │   ├── date.rs          # DateNormalizer trait, RuleBasedDateNormalizer
│       │   ├── table_extraction.rs  # extract_table_candidates, is_valid_financial_table
│       │   ├── markdown.rs      # preprocess_markdown, clean_latex_from_markdown
│       │   ├── cleaner.rs       # DataFrameCleaner, numeric parsing
│       │   ├── normalization.rs # normalize_table_structure
│       │   ├── error.rs         # PipelineError, Result
│       │   └── types.rs         # PdfError
│       ├── tests/
│       │   └── pipeline.rs      # Integration tests
│       └── prompts/             # (duplicate of parent prompts/, for crate-local access)
```

**Where is X?**
- OCR provider implementations → `pdf_pipeline/tbel-pdf/src/ocr.rs`
- Table extraction logic → `pdf_pipeline/tbel-pdf/src/table_extraction.rs`
- Date normalization → `pdf_pipeline/tbel-pdf/src/date.rs`
- CLI argument handling → `pdf_pipeline/tbel-pdf/src/commands/`
- JSON output schemas → `pdf_pipeline/tbel-pdf/src/contract/mod.rs`
- Domain types → `pdf_pipeline/tbel-pdf/src/models/`
- Test fixtures → `pdf_pipeline/tests/fixtures/`
- Golden files → `pdf_pipeline/tests/golden/`

## 4. Life of a Request / Primary Data Flow

**CLI Pipeline Flow:**

```
CLI args (input-url, report-type)
        │
        ▼
┌──────────────────────────────────────────────────────────────────┐
│ 1. Entrypoint: src/bin/tbel-pdf.rs                              │
│    - Parse clap args via App::parse()                           │
│    - Initialize tracing subscriber                              │
│    - Dispatch to Commands::Pipeline                             │
└──────────────────────────────────────────────────────────────────┘
        │
        ▼
┌──────────────────────────────────────────────────────────────────┐
│ 2. Command: src/commands/pipeline.rs                            │
│    - Infer ReportType from filename if not provided             │
│    - Validate arguments                                         │
│    - Orchestrate pipeline execution (Inferred: full pipeline    │
│      orchestration is stubbed in current code)                  │
└──────────────────────────────────────────────────────────────────┘
        │
        ▼
┌──────────────────────────────────────────────────────────────────┐
│ 3. OCR: src/ocr.rs (MistralOcrProvider)                         │
│    - Send PDF URL to Mistral OCR API                            │
│    - Receive markdown content with embedded images               │
│    - Return OcrOutput (markdown, page_count, document_id)       │
│    - Fallback: MockOcrProvider for testing                      │
└──────────────────────────────────────────────────────────────────┘
        │
        ▼
┌──────────────────────────────────────────────────────────────────┐
│ 4. Preprocessing: src/markdown.rs                               │
│    - Clean LaTeX artifacts from OCR markdown                    │
│    - Merge fragmented tables                                    │
└──────────────────────────────────────────────────────────────────┘
        │
        ▼
┌──────────────────────────────────────────────────────────────────┐
│ 5. Table Extraction: src/table_extraction.rs                    │
│    - Parse HTML/Markdown for table candidates                   │
│    - Filter by financial header patterns                        │
│    - Validate dimensions (min 3 cols × 10 rows)                 │
│    - Output: Vec<ReportTable>                                   │
└──────────────────────────────────────────────────────────────────┘
        │
        ▼
┌──────────────────────────────────────────────────────────────────┐
│ 6. Cleaning & Normalization: src/cleaner.rs, src/date.rs        │
│    - Parse Belarusian number formats (spaces, commas, parens)   │
│    - Normalize date headers via Mistral or rule-based fallback  │
│    - Validate financial codes (010-090, 100-999)                │
└──────────────────────────────────────────────────────────────────┘
        │
        ▼
┌──────────────────────────────────────────────────────────────────┐
│ 7. Output: src/contract/mod.rs                                  │
│    - Build SuccessContract or FailureContract                   │
│    - Emit JSON contract to stdout/file                          │
│    - Write XLSX output (feature-gated via rust_xlsxwriter)      │
│    - Return exit code (0=success, 1=usage, 2=pipeline, 3=provider)│
└──────────────────────────────────────────────────────────────────┘
```

**Observed vs Inferred:**
- Steps 1-2 and 7 are fully observable in `src/commands/pipeline.rs`
- Steps 3-6 are documented in `pdf_pipeline/README.md` and implemented in respective modules, but the orchestration wiring in `pipeline.rs:execute()` is currently a stub (returns a placeholder `SuccessContract`)
- Full pipeline orchestration is `Inferred` from module structure and README documentation

## 5. Architectural Invariants & Constraints

1. **Rule:** Domain models in `src/models/` must not perform I/O
   - **Rationale:** Enables pure unit testing, clear separation of concerns, and stable data contracts
   - **Enforcement / Signals (Observed):** Models only contain data structures with `Serialize`/`Deserialize`; no `reqwest`, `tokio`, or filesystem imports in model files

2. **Rule:** CLI is feature-gated and optional
   - **Rationale:** Allows library use without CLI dependencies; reduces compile time and binary size for embedded use
   - **Enforcement / Signals (Observed):** `cli` feature in `Cargo.toml` with optional dependencies; `#[cfg(feature = "cli")]` guards in `src/bin/` and `src/commands/`

3. **Rule:** Exit codes are standardized and stable
   - **Rationale:** Enables reliable scripting and CI integration
   - **Enforcement / Signals (Observed):** `ExitCode` enum in `src/contract/mod.rs` with explicit values 0-3

4. **Rule:** JSON contract schema must remain backward-compatible
   - **Rationale:** External callers (Python integration, CI) depend on stable output format
   - **Enforcement / Signals (Inferred):** `SuccessContract` and `FailureContract` are versioned types; changes require updating contract tests (anti-pattern noted in `AGENTS.md`)

5. **Rule:** OCR and date normalization must have test doubles
   - **Rationale:** Enables offline unit testing without network calls or API keys
   - **Enforcement / Signals (Observed):** `MockOcrProvider`, `StubOcrProvider`, `StubDateNormalizer` in `src/ocr.rs` and `src/date.rs`

6. **Rule:** Rust toolchain is pinned to exact version 1.94.0
   - **Rationale:** Ensures reproducible builds across environments
   - **Enforcement / Signals (Observed):** `rust-toolchain.toml` specifies `channel = "1.94.0"`

7. **Rule:** All external HTTP calls go through adapter traits
   - **Rationale:** Enables mocking, retry logic, and provider swapping without changing business logic
   - **Enforcement / Signals (Observed):** `OcrProvider` trait in `src/ocr.rs`; no direct `reqwest` calls outside adapters

8. **Rule:** Table validation enforces minimum dimensions
   - **Rationale:** Filters out noise from OCR artifacts and non-financial tables
   - **Enforcement / Signals (Observed):** `is_valid_financial_table` checks 3 cols × 10 rows minimum; documented in `README.md`

9. **Rule:** Report type is inferred from URL filename
   - **Rationale:** Reduces required CLI arguments; enforces naming convention
   - **Enforcement / Signals (Observed):** `ReportType::try_from_filename` in `src/models/report_type.rs`

## 6. Documentation Strategy

**Hierarchy:**

1. **`ARCHITECTURE.md`** (this file) — Global map, logical components, invariants, and high-level data flow. The authoritative source for architectural decisions and boundaries.

2. **`pdf_pipeline/AGENTS.md`** — Rust-specific guidance including commands, workspace constraints, and module-level "where to look" table. Supersedes root `AGENTS.md` for Rust work.

3. **`pdf_pipeline/README.md`** — Operational reference: CLI usage, environment variables, supported report types, error codes, and development workflow.

4. **`pdf_pipeline/docs/cli-contract.md`** — JSON contract specification for machine-readable CLI output.

5. **Module-level docs** — Each major module has doc comments explaining purpose and key exports (observable in `src/lib.rs`, `src/ocr.rs`, `src/table_extraction.rs`).

**Note on path discrepancies:** Root `AGENTS.md` and `pdf_pipeline/AGENTS.md` reference paths like `rust/pdf_pipeline/` which do not match the actual repository structure. The actual workspace is at `pdf_pipeline/` relative to repository root. This is a documentation artifact from a prior structure.

**What belongs where:**
- Global architecture and invariants → `ARCHITECTURE.md`
- Rust build/lint/test commands → `pdf_pipeline/AGENTS.md`
- CLI usage and configuration → `pdf_pipeline/README.md`
- JSON schema details → `pdf_pipeline/docs/cli-contract.md`
- Module internals → inline rustdoc comments
