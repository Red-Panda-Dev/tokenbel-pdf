# Architecture

## 1. High-Level Overview

This repository is a Rust library and CLI for extracting structured financial tables from Belarusian PDF reports via OCR. The system takes PDF documents (by URL or bytes), sends them through the Mistral OCR API, preprocesses the resulting markdown, extracts and validates financial tables, and produces structured JSON/XLSX output. The library also compiles to wasm32 for browser and edge deployment via `wasm_bindgen`.

**Observed identity and purpose:**
- Single Cargo workspace with one crate `tbel-pdf` (`pdf_pipeline/Cargo.toml`, `pdf_pipeline/tbel-pdf/Cargo.toml`)
- Dual compilation targets: native (CLI + library) and wasm32 (library-only) (`pdf_pipeline/tbel-pdf/Cargo.toml:15-20`)
- Target domain: Belarusian financial statements — BalanceSheet, IncomeStatement, StatementCashFlow, StatementEquityChanges (`pdf_pipeline/tbel-pdf/src/models/report_type.rs`)
- External OCR dependency: Mistral OCR API, abstracted behind a trait (`pdf_pipeline/tbel-pdf/src/ocr.rs`)

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
│  clap arg parsing, subcommand dispatch, exit code mapping        │
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
│  markdown → table_extraction → validation → normalization        │
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
```

**Components:**

1. **CLI Layer** — Argument parsing via clap, subcommand dispatch to `pipeline`, exit code mapping. Gated behind the `cli` feature and blocked on wasm32 by `compile_error!`. (`src/bin/`, `src/commands/`)

2. **wasm Bridge** — JavaScript interop via `wasm_bindgen`. Exposes `process_markdown`, `process_pdf`, `validate_markdown`, and `get_supported_report_types`. Handles XLSX export internally. (`src/wasm_bridge.rs`, wasm32 only)

3. **Contract Layer** — Typed JSON output schemas (`SuccessContract`, `FailureContract`) and exit codes for CLI consumers. Native only. (`src/contract/`)

4. **ProcessingFacade** — The single shared orchestration entry point used by both the wasm bridge and (eventually) the CLI. Accepts `PdfInput` + `OcrProvider`, runs the full pipeline: OCR → markdown preprocessing → table extraction → validation. (`src/processing.rs`)

5. **Processing Pipeline** — Stateless transformation stages: markdown cleaning/LaTeX removal, table candidate extraction from HTML/markdown, financial table validation (min 3 cols × 10 rows), data cleaning (Belarusian number formats), normalization. (`src/markdown.rs`, `src/table_extraction.rs`, `src/cleaner.rs`, `src/normalization.rs`, `src/date.rs`)

6. **Domain Models** — Pure data types: `ReportTable`, `TableCell`, `PdfInput` (Url | Bytes), `OcrOutput`, `ReportType`, `CleanedReport`, `CodeValue`. Zero I/O imports. (`src/models/`)

7. **Adapters** — Trait-based external service boundaries: `OcrProvider` (Mistral, Mock, Stub), `DateNormalizer`, `PdfReader`, HTML scraper. Real implementations live in top-level modules; `src/adapters/mod.rs` only re-exports for backwards compatibility. (`src/ocr.rs`, `src/pdf.rs`, `src/scraper.rs`, `src/date.rs`)

**Dependency direction:**
- CLI → Contract → ProcessingFacade → Pipeline + Adapters → Models
- wasm_bridge → ProcessingFacade → Pipeline + Adapters → Models
- Models have zero dependency on adapters, I/O, or pipeline logic
- CLI and wasm_bridge are mutually exclusive at compile time (feature gates + target arch)

**What is intentionally NOT depended upon:**
- Models do not import `reqwest`, `tokio`, `chrono::Local`, `scraper`, or any filesystem API
- Library code (non-`cli` feature) does not depend on `clap`, `tracing-subscriber`, or `tokio`
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
    ├── ci-check.sh              # CI matrix: native lib + tests + CLI, wasm32 lib + tests + smoke
    ├── README.md                # Operational reference (CLI usage, env vars, report types)
    ├── AGENTS.md                # Authoritative Rust-specific guidance
    ├── docs/
    │   └── cli-contract.md      # CLI JSON contract specification
    ├── tests/                   # Workspace-level test fixtures
    │   ├── *.pdf                # Sample financial reports (3 files)
    │   ├── fixtures/            # manifest.json + source_of_truth/ reference data
    │   ├── golden/              # Regression golden files (10 JSON+XLSX pairs)
    │   └── output/              # Test output artifacts (gitignored)
    └── tbel-pdf/                # The unified crate: library + CLI + wasm bridge
        ├── Cargo.toml           # Crate manifest: features default=[], cli; crate-type cdylib+rlib
        ├── src/
        │   ├── lib.rs           # Public API re-exports, compile_error! guard
        │   ├── processing.rs    # ProcessingFacade — shared orchestration entry point
        │   ├── wasm_bridge.rs   # wasm_bindgen exports (wasm32 only)
        │   ├── bin/
        │   │   └── tbel-pdf.rs  # CLI binary entrypoint (feature-gated)
        │   ├── commands/        # CLI subcommand dispatch (cli feature-gated)
        │   │   ├── mod.rs       # clap App/Commands definitions
        │   │   └── pipeline.rs  # Pipeline command (currently stub)
        │   ├── contract/        # ExitCode, ErrorCode, SuccessContract, FailureContract
        │   │   └── mod.rs
        │   ├── models/          # Pure domain types — no I/O
        │   │   ├── mod.rs       # Re-exports all model types
        │   │   ├── report_table.rs
        │   │   ├── report_type.rs
        │   │   ├── pdf_input.rs
        │   │   ├── ocr_output.rs
        │   │   ├── cleaned_report.rs
        │   │   └── code_value.rs
        │   ├── adapters/        # Re-exports from top-level modules (backwards compat)
        │   │   ├── mod.rs       # pub use crate::{ocr, pdf, date, table_extraction}
        │   │   ├── ocr.rs       # Adapter submodule mirror
        │   │   ├── pdf.rs       # Adapter submodule mirror
        │   │   ├── scraper.rs   # Adapter submodule mirror
        │   │   ├── date.rs      # Adapter submodule mirror
        │   │   └── markdown.rs  # Adapter submodule mirror
        │   ├── ocr.rs           # OcrProvider trait, MistralOcrProvider, Mock/Stub providers
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
            ├── pipeline.rs      # Integration tests
            └── worker_smoke.mjs # Node.js wasm smoke test runner
```

**Where is X?**
- OCR provider implementations → `pdf_pipeline/tbel-pdf/src/ocr.rs`
- wasm JS interop → `pdf_pipeline/tbel-pdf/src/wasm_bridge.rs`
- Shared processing orchestration → `pdf_pipeline/tbel-pdf/src/processing.rs`
- Table extraction logic → `pdf_pipeline/tbel-pdf/src/table_extraction.rs`
- Date normalization → `pdf_pipeline/tbel-pdf/src/date.rs`
- CLI argument handling → `pdf_pipeline/tbel-pdf/src/commands/`
- JSON output schemas → `pdf_pipeline/tbel-pdf/src/contract/mod.rs`
- Domain types → `pdf_pipeline/tbel-pdf/src/models/`
- Test fixtures → `pdf_pipeline/tests/fixtures/`
- Golden regression files → `pdf_pipeline/tests/golden/`
- CI verification script → `pdf_pipeline/ci-check.sh`

## 4. Life of a Request / Primary Data Flow

### Native CLI path

```
CLI args (--input-url, --report-type)
    │
    ▼
src/bin/tbel-pdf.rs
  - Parse clap args via App::parse()
  - Initialize tracing subscriber
  - Dispatch to Commands::Pipeline
    │
    ▼
src/commands/pipeline.rs — execute()
  - Infer ReportType from filename if not provided
  - Currently a stub: constructs placeholder SuccessContract
  - (Inferred) Full pipeline orchestration pending
    │
    ▼
stdout / --emit-contract file
```

**Status:** The CLI pipeline command is `Observed` as a stub in `src/commands/pipeline.rs:39-75`. It does not yet call `ProcessingFacade`.

### wasm bridge path (fully wired)

```
JS caller → wasm_bridge exports (process_markdown | process_pdf)
    │
    ▼
wasm_bridge.rs: decode_request → parse_report_type
    │
    ▼
ProcessingFacadeBuilder → build ProcessingFacade
    │
    ├─ process_markdown path:
    │    ProcessingFacade::process_markdown(markdown, page_count, doc_id)
    │      → clean_latex_from_markdown
    │      → preprocess_markdown
    │      → extract_table_candidates_from_markdown
    │      → filter is_valid_financial_table
    │
    └─ process_pdf path:
         ProcessingFacade::process(PdfInput, &dyn OcrProvider)
           → ocr.acquire_ocr(input)  [Mistral API call]
           → process_ocr_output (same pipeline as above)
    │
    ▼
ProcessingResult → JsSuccess (with optional XLSX via rust_xlsxwriter)
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
    ▼ 3. extract_table_candidates_from_markdown — parse HTML/Markdown tables
    ▼ 4. is_valid_financial_table — filter (min 3 cols × 10 rows)
    ▼ 5. ReportType detection (filename heuristic or explicit)
    │
    ▼
ProcessingResult { document_id, report_type, tables, page_count }
```

## 5. Architectural Invariants & Constraints

- **Rule:** Domain models in `src/models/` must not perform I/O
  - **Rationale:** Enables pure unit testing and clear separation of data contracts from infrastructure.
  - **Enforcement / Signals (Observed):** Models only contain data structures with `Serialize`/`Deserialize`; no `reqwest`, `tokio`, `scraper`, `lopdf`, or filesystem imports. All model submodules are declared `mod` (private) in `models/mod.rs` with `pub use` re-exports.

- **Rule:** CLI feature is gated and blocked on wasm32
  - **Rationale:** The crate compiles as both a native binary and a wasm32 library; CLI dependencies (`clap`, `tokio`, `tracing-subscriber`) are unnecessary and unresolvable on wasm32.
  - **Enforcement / Signals (Observed):** `compile_error!` guard in `src/lib.rs:16-19`; `cli` feature in `Cargo.toml:20` with optional native-only dependencies; `[[bin]]` entry requires `cli` feature.

- **Rule:** ProcessingFacade is the single shared orchestration entry point
  - **Rationale:** Prevents duplicated pipeline logic between CLI and wasm bridge.
  - **Enforcement / Signals (Observed):** Both `src/wasm_bridge.rs` and `src/processing.rs` itself use `ProcessingFacade`/`ProcessingFacadeBuilder`. The CLI pipeline command does not yet use it (`Inferred` pending integration).

- **Rule:** All external HTTP calls go through adapter traits
  - **Rationale:** Enables mocking, provider swapping, and offline testing without changing business logic.
  - **Enforcement / Signals (Observed):** `OcrProvider` trait in `src/ocr.rs` with `async fn acquire_ocr`; `MockOcrProvider` and `StubOcrProvider` test doubles defined alongside production `MistralOcrProvider`.

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

- **Rule:** JSON contract schema changes must update contract tests
  - **Rationale:** External callers (Python integration, CI) depend on stable output format.
  - **Enforcement / Signals (Inferred):** `SuccessContract` and `FailureContract` have serialization round-trip tests in `src/contract/mod.rs`; `docs/cli-contract.md` defines the stable schema.

- **Rule:** wasm32 builds exclude native-only dependencies
  - **Rationale:** `chrono`, `scraper`, `lopdf`, and `tokio` cannot compile or are unnecessary on wasm32.
  - **Enforcement / Signals (Observed):** `Cargo.toml` uses `cfg(not(target_arch = "wasm32"))` and `cfg(target_arch = "wasm32")` for dependency and module gating.

- **Rule:** `src/adapters/mod.rs` only re-exports; canonical implementations are top-level modules
  - **Rationale:** Historical backwards compatibility; the real adapter code lives in `src/ocr.rs`, `src/pdf.rs`, etc.
  - **Enforcement / Signals (Observed):** `src/adapters/mod.rs` contains only `pub use crate::...` statements; adapter submodules exist as mirrors.

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
