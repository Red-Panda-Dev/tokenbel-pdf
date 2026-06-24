# Architecture

## 1. High-Level Overview

Observed: this repository is a Rust PDF/OCR processing pipeline for Belarusian financial reports. It is rooted at the Git repository root and contains a single Cargo workspace under `pdf_pipeline/` with one crate, `tbel-pdf`; the crate builds as a reusable Rust library, a feature-gated native CLI, and a wasm-compatible library artifact. Evidence: `pdf_pipeline/Cargo.toml`, `pdf_pipeline/tbel-pdf/Cargo.toml`, `pdf_pipeline/tbel-pdf/src/lib.rs`, `pdf_pipeline/tbel-pdf/src/bin/tbel-pdf.rs`, `pdf_pipeline/tbel-pdf/src/wasm_bridge.rs`.

Observed business purpose: the pipeline turns statutory financial PDF reports into normalized tabular output for TokenBel products. This is stated in `README.md` and `pdf_pipeline/README.md`, and is reflected in code paths that produce structured JSON contracts and XLSX output. Inferred architectural paradigm: a small Cargo workspace organized around a shared processing facade, with native CLI and wasm entrypoints depending inward on target-neutral pipeline modules and domain models. Unknown: repository artifacts do not define a production deployment topology or hosted runtime.

## 2. System Architecture (Logical)

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

- Workspace and target boundary: `pdf_pipeline/Cargo.toml` declares one member, `tbel-pdf`. `pdf_pipeline/tbel-pdf/Cargo.toml` declares `rlib` and `cdylib` outputs, a `cli` feature, target-specific dependencies, and a binary requiring `cli`.
- Native CLI layer: `pdf_pipeline/tbel-pdf/src/bin/tbel-pdf.rs` parses the feature-gated CLI and dispatches to `src/commands/`. `src/commands/pipeline.rs` owns CLI concerns: argument handling, environment lookup, native file output, XLSX writing, stage artifacts, and contract emission.
- CLI contract layer: `pdf_pipeline/tbel-pdf/src/contract/mod.rs` defines the externally visible JSON success/failure contract and exit codes. It is native-only in `src/lib.rs` and documented in `pdf_pipeline/docs/cli-contract.md`.
- Wasm bridge: `pdf_pipeline/tbel-pdf/src/wasm_bridge.rs` is compiled only for `wasm32` and exposes JavaScript-facing functions through `wasm_bindgen`. It adapts JS values to the same library pipeline and serializes results back to JS.
- Shared processing facade: `pdf_pipeline/tbel-pdf/src/processing.rs` is the central orchestration boundary for OCR output, markdown preprocessing, table extraction, validation, and `ProcessingResult` creation.
- Pipeline and cleaning modules: `src/markdown.rs`, `src/table_extraction.rs`, `src/report_cleaning.rs`, `src/date.rs`, `src/normalization.rs`, and `src/cleaner.rs` contain reusable transformation logic. These modules sit below CLI/wasm entrypoints and above domain models.
- Adapter and model boundary: `src/ocr.rs`, `src/pdf.rs`, and `src/scraper.rs` contain external or format-facing adapters. `src/models/` contains data shapes used by the pipeline and does not own HTTP, filesystem, CLI, or PDF-reader behavior.

Observed dependency direction is entrypoints to shared pipeline to adapters/models. The visible architectural boundary is mostly enforced by Cargo features and `cfg` gates rather than by separate crates: native CLI code is feature-gated, wasm code is target-gated, and models are kept as local data types.

## 3. Code Map (Physical)

```text
tokenbel-pdf/
├── README.md                    # Business overview and normalization context
├── AGENTS.md                    # Repository-wide contributor guidance
├── ARCHITECTURE.md              # Global map and invariants
└── pdf_pipeline/                # Cargo workspace root
    ├── Cargo.toml               # Workspace manifest; member: tbel-pdf
    ├── Cargo.lock               # Locked Rust dependencies
    ├── rust-toolchain.toml      # Pinned Rust toolchain
    ├── ci-check.sh              # Native, CLI, wasm, and optional coverage checks
    ├── docs/
    │   └── cli-contract.md      # Stable CLI JSON contract documentation
    ├── tests/                   # Workspace fixtures and golden regression data
    │   ├── fixtures/            # OCR and source-of-truth inputs
    │   └── golden/              # JSON/XLSX regression baselines
    └── tbel-pdf/                # Unified Rust crate: library + CLI + wasm bridge
        ├── Cargo.toml           # Crate metadata, features, crate types, target deps
        ├── prompts/             # Prompt assets used by OCR/date-related behavior
        ├── src/
        │   ├── lib.rs           # Public module surface and target/feature gates
        │   ├── bin/             # Native CLI binary entrypoint
        │   ├── commands/        # CLI command dispatch and output path
        │   ├── contract/        # CLI JSON contracts and exit codes
        │   ├── models/          # Domain data shapes
        │   ├── processing.rs    # Shared ProcessingFacade
        │   ├── wasm_bridge.rs   # wasm-bindgen bridge
        │   ├── ocr.rs           # OCR provider trait and implementations
        │   ├── pdf.rs           # PDF input/reader boundary
        │   ├── scraper.rs       # HTML/document scraping boundary
        │   └── *_extraction / cleaning / normalization modules
        └── tests/               # Crate integration and wasm smoke tests
```

Where to look:

- Shared orchestration: `pdf_pipeline/tbel-pdf/src/processing.rs`
- Native CLI startup and command flow: `pdf_pipeline/tbel-pdf/src/bin/tbel-pdf.rs`, `pdf_pipeline/tbel-pdf/src/commands/`
- Wasm interop: `pdf_pipeline/tbel-pdf/src/wasm_bridge.rs`
- OCR provider boundary: `pdf_pipeline/tbel-pdf/src/ocr.rs`
- Table extraction and validation: `pdf_pipeline/tbel-pdf/src/table_extraction.rs`
- Business table cleaning: `pdf_pipeline/tbel-pdf/src/report_cleaning.rs`
- Domain data types: `pdf_pipeline/tbel-pdf/src/models/`
- Compatibility re-exports for older adapter paths: `pdf_pipeline/tbel-pdf/src/adapters/mod.rs`
- Regression fixtures: `pdf_pipeline/tests/fixtures/`, `pdf_pipeline/tests/golden/`

Omitted from this map: `target/`, `.osgrep/`, test output directories, and other generated/cache artifacts that do not define source architecture.

## 4. Life of a Request / Primary Data Flow

Observed native CLI path:

```text
CLI invocation
  -> pdf_pipeline/tbel-pdf/src/bin/tbel-pdf.rs
  -> pdf_pipeline/tbel-pdf/src/commands/mod.rs
  -> pdf_pipeline/tbel-pdf/src/commands/pipeline.rs
  -> MistralOcrProvider through OcrProvider in src/ocr.rs
  -> ProcessingFacade::process_markdown in src/processing.rs
  -> markdown preprocessing in src/markdown.rs
  -> table extraction/validation in src/table_extraction.rs
  -> business cleaning/date normalization in src/report_cleaning.rs and src/date.rs
  -> XLSX output and SuccessContract JSON from src/commands/pipeline.rs
```

The CLI path is request-like but not a web server: input is a PDF URL/path accepted by the command, OCR is acquired through the provider boundary, markdown is transformed into `ReportTable` values, cleaned tables are produced, and the CLI writes XLSX plus a JSON contract. Live OCR requires `MISTRAL_API_KEY`, as documented in `README.md` and used in `src/commands/pipeline.rs`.

Observed library/wasm paths:

```text
Rust caller or JS caller
  -> public API from src/lib.rs or wasm exports in src/wasm_bridge.rs
  -> ProcessingFacade::process or ProcessingFacade::process_markdown
  -> shared markdown/table pipeline
  -> ProcessingResult and serialized caller-facing output
```

`ProcessingFacade::process` accepts `PdfInput` plus an `OcrProvider`; `ProcessingFacade::process_markdown` accepts pre-extracted OCR markdown. Inferred: this split exists so native callers can run the full OCR path while wasm or integration callers can reuse the parsing path when OCR markdown is already available. Repository artifacts do not describe any persistent database, queue, or long-running server request lifecycle.

## 5. Architectural Invariants & Constraints

- Rule: Keep `tbel-pdf` as the single crate in the Cargo workspace unless the workspace architecture is intentionally changed.
  - Rationale: The repository currently centralizes library, CLI, and wasm surfaces in one crate with shared modules.
  - Enforcement / Signals (Observed or Inferred): Observed in `pdf_pipeline/Cargo.toml` with one workspace member and in `pdf_pipeline/tbel-pdf/Cargo.toml` with both library crate types and the binary target.

- Rule: Native CLI code must stay behind the `cli` feature and must not be built for `wasm32`.
  - Rationale: CLI dependencies and native process behavior are not part of the wasm library surface.
  - Enforcement / Signals (Observed or Inferred): Observed `required-features = ["cli"]` in `pdf_pipeline/tbel-pdf/Cargo.toml`; observed `compile_error!` for `feature = "cli"` with `target_arch = "wasm32"` in `src/lib.rs`.

- Rule: CLI and wasm entrypoints should route shared extraction behavior through `ProcessingFacade` rather than duplicating pipeline stages.
  - Rationale: One orchestration boundary keeps native and wasm behavior aligned.
  - Enforcement / Signals (Observed or Inferred): Observed `ProcessingFacade` in `src/processing.rs`; observed uses from `src/commands/pipeline.rs` and `src/wasm_bridge.rs`. This is a structural convention, not a separate-crate enforcement boundary.

- Rule: `src/models/` must remain domain-data-only and must not acquire HTTP, filesystem, Tokio, scraper, PDF-reader, CLI, or wasm bridge responsibilities.
  - Rationale: Model types are the stable data vocabulary shared across targets and tests.
  - Enforcement / Signals (Observed or Inferred): Observed model files import serialization/std path or formatting utilities but no `reqwest`, `tokio`, `scraper`, `lopdf`, `clap`, `rust_xlsxwriter`, or filesystem APIs. Also stated in `pdf_pipeline/tbel-pdf/AGENTS.md`.

- Rule: External OCR access must stay behind `OcrProvider`.
  - Rationale: The pipeline needs production Mistral OCR and offline test doubles without changing transformation logic.
  - Enforcement / Signals (Observed or Inferred): Observed `OcrProvider`, `MistralOcrProvider`, `MockOcrProvider`, and `StubOcrProvider` in `src/ocr.rs`; observed `ProcessingFacade::process` depends on `&dyn OcrProvider`.

- Rule: CLI JSON output is a contract and must evolve with its documentation and tests.
  - Rationale: Downstream scripts or products can depend on stable exit codes and JSON shape.
  - Enforcement / Signals (Observed or Inferred): Observed `src/contract/mod.rs`, `pdf_pipeline/docs/cli-contract.md`, and workspace guidance in `AGENTS.md`. Inferred enforcement through contract/golden tests under `pdf_pipeline/tests/` and `pdf_pipeline/tbel-pdf/tests/`.

- Rule: Report cleaning must remain reusable library logic, not CLI-only plumbing.
  - Rationale: The same normalized business tables are needed by tests, CLI export, and library consumers.
  - Enforcement / Signals (Observed or Inferred): Observed `src/report_cleaning.rs` is re-exported from `src/lib.rs` and used by `src/commands/pipeline.rs`; no CLI feature gate is visible on the module.

- Rule: Keep real OCR/network dependence out of normal regression tests.
  - Rationale: Tests should run offline and deterministically while still covering observed OCR edge cases.
  - Enforcement / Signals (Observed or Inferred): Observed committed OCR fixtures in `pdf_pipeline/tests/fixtures/ocr/`, golden outputs in `pdf_pipeline/tests/golden/`, mock/stub OCR providers in `src/ocr.rs`, and local guidance in `pdf_pipeline/tests/AGENTS.md`.

- Rule: Preserve target-specific dependency boundaries in `Cargo.toml`.
  - Rationale: Native and wasm builds have different platform constraints but share the same crate.
  - Enforcement / Signals (Observed or Inferred): Observed target-specific dependency sections in `pdf_pipeline/tbel-pdf/Cargo.toml`; observed `ci-check.sh` checks native library, native CLI, wasm library, and wasm test compilation.

- Rule: Use the pinned Rust toolchain for repository validation.
  - Rationale: Formatting, linting, and wasm checks should be reproducible across contributors and CI.
  - Enforcement / Signals (Observed or Inferred): Observed `pdf_pipeline/rust-toolchain.toml` and validation commands in `README.md`, `pdf_pipeline/README.md`, and `pdf_pipeline/ci-check.sh`.

## 6. Documentation Strategy

`ARCHITECTURE.md` is the global map plus invariants: it should explain repository shape, dependency direction, target boundaries, and where major concerns live. It should not duplicate detailed CLI schemas, fixture rules, or module implementation notes.

Local documentation owns local detail:

- `README.md` explains business purpose, supported reports, normalization expectations, and typical commands.
- `pdf_pipeline/README.md` explains workspace operation, pipeline stages, fixture usage, CLI usage, coverage, and development notes.
- `pdf_pipeline/docs/cli-contract.md` is the CLI JSON/exit-code contract reference.
- `AGENTS.md`, `pdf_pipeline/AGENTS.md`, `pdf_pipeline/tbel-pdf/AGENTS.md`, and `pdf_pipeline/tests/AGENTS.md` describe contributor guidance and local boundaries.
- `pdf_pipeline/tbel-pdf/README.md` describes the crate API, feature flags, and module map.

When code and documentation diverge, prefer observable source and manifest facts for architecture decisions, then update the relevant local document. Put new cross-cutting boundaries here; put command examples, schema details, fixture maintenance rules, and module-level usage notes in the nearest README, AGENTS file, or `pdf_pipeline/docs/cli-contract.md`.
