# Architecture

## 1. High-Level Overview

This repository is a Rust PDF/OCR processing pipeline that turns Belarusian statutory financial PDF reports into normalized tabular output (cleaned tables, JSON contracts, XLSX workbooks). It contains a single Cargo workspace under `pdf_pipeline/` with one crate, `tbel-pdf`, built three ways: a target-neutral library (`rlib`), a `wasm32` library artifact (`cdylib`, JS-facing via `wasm-bindgen`), and a feature-gated native CLI binary. Evidence: `pdf_pipeline/Cargo.toml`, `pdf_pipeline/tbel-pdf/Cargo.toml`, `pdf_pipeline/tbel-pdf/src/lib.rs`, `pdf_pipeline/tbel-pdf/src/bin/tbel-pdf.rs`, `pdf_pipeline/tbel-pdf/src/wasm_bridge.rs`, `README.md`.

The architectural paradigm is one unified crate with compile-time target/feature boundaries instead of crate splits: native CLI code lives behind the `cli` feature, the wasm bridge is `wasm32`-only, and both entrypoints converge on a single orchestrator, `ProcessingFacade` in `pdf_pipeline/tbel-pdf/src/processing.rs`. External interactions (Mistral OCR API, PDF bytes via `lopdf`, optional Mistral date translation) sit behind traits or narrow adapter modules so tests run offline against committed fixtures. Evidence: `src/ocr.rs`, `src/date.rs`, `pdf_pipeline/ci-check.sh`, `pdf_pipeline/tbel-pdf/tests/pipeline.rs`.

Inferred: the wasm bridge plus the Node worker smoke test (`pdf_pipeline/tbel-pdf/tests/worker_smoke.mjs`) indicate a browser/worker consumer of the library, and `homepage = "https://dashboard.tokenbel.info/"` in the crate manifest indicates a TokenBel dashboard as downstream product.

### Unknowns

- Production deployment topology, hosted runtime, and hosted CI configuration are not represented in the repository; validation is script-driven via `pdf_pipeline/ci-check.sh`.

## 2. System Architecture (Logical)

```text
Dependency direction (top → bottom):

  Native CLI (feature "cli")        Wasm bridge (wasm32 only)
  src/bin/, src/commands/           src/wasm_bridge.rs
  src/contract/ (native-only)       (Js* types mirror the CLI contract)
            |                               |
            └───────────────┬───────────────┘
                            v
              ProcessingFacade (src/processing.rs)
                            |
                            v
     Pipeline transformations: markdown, table_extraction,
     report_cleaning, normalization, cleaner, date
                            |
                            v
     Adapters: ocr, pdf, scraper      Models: models/ (pure data)
```

Boundaries are enforced by Cargo features and `cfg` gates rather than by separate crates.

### Native CLI entrypoint

- Responsibility: parses arguments and owns native-only concerns (environment lookup, local file output, XLSX writing, stage artifacts); drives the pipeline end to end.
- Code locations: `pdf_pipeline/tbel-pdf/src/bin/tbel-pdf.rs`, `pdf_pipeline/tbel-pdf/src/commands/`
- Entry points: `bin/tbel-pdf.rs::main` → `commands::App::execute` (single `pipeline` subcommand)
- Depends on: `cli` feature dependencies (clap, tokio, tracing-subscriber, rust_xlsxwriter), `src/contract/`, `src/ocr.rs`, `src/processing.rs`
- Must not depend on: `wasm32` code — the binary requires the `cli` feature, which is compile-blocked on `wasm32`
- Owns: CLI argument shape, file/XLSX output, process exit codes
- State and external boundaries: process environment (`MISTRAL_API_KEY`), local filesystem
- Evidence: `pdf_pipeline/tbel-pdf/src/commands/mod.rs`, `pdf_pipeline/tbel-pdf/src/commands/pipeline.rs`, `required-features = ["cli"]` in `pdf_pipeline/tbel-pdf/Cargo.toml`

### CLI output contract

- Responsibility: stable JSON success/failure shape and exit codes for the CLI; mirrored by wasm `JsSuccess`/`JsError` types.
- Code locations: `pdf_pipeline/tbel-pdf/src/contract/mod.rs` (native-only module); mirrors in `src/wasm_bridge.rs`
- Entry points: `SuccessContract`, `FailureContract`, `ExitCode` (re-exported from `src/lib.rs`)
- Depends on: serde/serde_json, `src/models/`
- Must not depend on: CLI argument handling or transport internals
- Owns: the externally visible output schema
- Evidence: `src/contract/mod.rs`, `pdf_pipeline/docs/cli-contract.md`

### Wasm bridge

- Responsibility: JS-facing API over the same pipeline; adapts JS values to `ProcessingFacade` and serializes results (including optional XLSX bytes) back to JS.
- Code locations: `pdf_pipeline/tbel-pdf/src/wasm_bridge.rs` (compiled only for `wasm32`)
- Entry points: `#[wasm_bindgen]` exports `process_markdown`, `process_pdf`, `validate_markdown`, `get_supported_report_types`
- Depends on: `src/processing.rs`, `src/ocr.rs` (`process_pdf` runs the full Mistral OCR path via wasm `reqwest`), `src/models/`
- Must not depend on: `src/commands/` or `src/contract/` (native-only modules)
- Owns: request/response DTOs (`MarkdownRequest`, `PdfRequest`, `JsSuccess`, …)
- State and external boundaries: JS host runtime; Mistral API via wasm `reqwest`
- Evidence: `src/wasm_bridge.rs`, `wasm32` dependency tables in `pdf_pipeline/tbel-pdf/Cargo.toml`

### Shared processing facade

- Responsibility: the single orchestration path for OCR output → markdown preprocessing → table extraction/validation → cleaned tables; used by CLI, wasm, library callers, and tests.
- Code locations: `pdf_pipeline/tbel-pdf/src/processing.rs`
- Entry points: `ProcessingFacade::process` (async; `PdfInput` + `&dyn OcrProvider`), `ProcessingFacade::process_markdown` (pre-extracted OCR markdown), `ProcessingFacadeBuilder`
- Depends on: `OcrProvider` trait, `src/markdown.rs`, `src/table_extraction.rs`, `src/report_cleaning.rs`, `src/models/`
- Must not depend on: CLI feature, wasm-bindgen, or filesystem concerns
- Owns: `ProcessingResult`, `ProcessingOptions`, stage ordering
- Evidence: `src/processing.rs`; call sites in `src/commands/pipeline.rs` and `src/wasm_bridge.rs`

### Pipeline transformations

- Responsibility: reusable, target-neutral extraction and business-cleaning logic.
- Code locations: `src/markdown.rs`, `src/table_extraction.rs`, `src/report_cleaning.rs`, `src/normalization.rs`, `src/cleaner.rs`, `src/date.rs`; shared error/type modules `src/error.rs`, `src/types.rs`
- Entry points: module functions re-exported from `src/lib.rs`
- Depends on: `src/models/`; `src/date.rs` optionally calls the Mistral chat API using the prompt template in `pdf_pipeline/tbel-pdf/prompts/financial_date_extraction.txt`
- Must not depend on: CLI feature, wasm-bindgen, or filesystem I/O
- Owns: `DateNormalizer` trait (offline rule-based default plus `StubDateNormalizer`), `CleanedTable` production
- Evidence: re-exports in `src/lib.rs`; `src/report_cleaning.rs` consumes `DateNormalizer`

### External adapters

- Responsibility: isolate format and network boundaries behind traits or narrow modules.
- Code locations: `src/ocr.rs` (`OcrProvider` trait with `MistralOcrProvider`, `MockOcrProvider`, `StubOcrProvider`), `src/pdf.rs` (`PdfReader` via `lopdf`), `src/scraper.rs` (regex/HTML helpers)
- Depends on: `src/models/`; native-only crates stay in the non-`wasm32` dependency table
- Must not depend on: pipeline stage logic
- Owns: provider configuration, `ProviderError`, PDF input reading
- Evidence: `src/ocr.rs`, `src/pdf.rs`, target-specific dependency sections in `pdf_pipeline/tbel-pdf/Cargo.toml`

### Domain models

- Responsibility: pure data vocabulary shared by all targets and tests.
- Code locations: `pdf_pipeline/tbel-pdf/src/models/` (`ReportTable`, `CleanedReport`, `OcrOutput`, `PdfInput`, `ReportType`, `CodeValue`)
- Depends on: serde/std only
- Must not depend on: HTTP, filesystem, Tokio, scraper, PDF-reader, CLI, or wasm code
- Owns: serialized domain shapes
- Evidence: imports in `src/models/*.rs`; boundary also stated in `pdf_pipeline/tbel-pdf/AGENTS.md`

## 3. Code Map (Physical)

```text
tokenbel-pdf/
├── ARCHITECTURE.md, AGENTS.md, README.md   # this map; agent rules; business overview
├── okf/                        # OKF v0.1 knowledge bundle (concept docs + log.md)
└── pdf_pipeline/               # Cargo workspace root
    ├── Cargo.toml              # single member: tbel-pdf
    ├── rust-toolchain.toml     # pinned toolchain 1.94.0
    ├── ci-check.sh             # native lib/CLI + wasm32 checks; optional smoke/coverage
    ├── coverage.sh             # coverage helper
    ├── docs/cli-contract.md    # CLI JSON/exit-code contract reference
    ├── tests/                  # regression data consumed by crate tests
    │   ├── fixtures/           # OCR markdown + source-of-truth inputs
    │   ├── golden/             # paired .json/.xlsx regression baselines
    │   └── output/             # generated at test time; not committed
    └── tbel-pdf/               # unified crate: library + CLI + wasm bridge
        ├── prompts/            # Mistral date-extraction template (used by src/date.rs)
        ├── src/
        │   ├── lib.rs          # module surface + target/feature gates (incl. wasm32/cli guard)
        │   ├── bin/, commands/ # native CLI (feature-gated)
        │   ├── contract/       # CLI JSON contract (native-only)
        │   ├── processing.rs   # ProcessingFacade orchestration
        │   ├── wasm_bridge.rs  # wasm32-only JS API
        │   ├── ocr.rs, pdf.rs, scraper.rs          # external/format adapters
        │   ├── markdown.rs, table_extraction.rs, report_cleaning.rs,
        │   │   normalization.rs, cleaner.rs, date.rs                 # pipeline stages
        │   ├── models/         # pure domain data
        │   └── adapters/       # legacy compat shims; not in the module tree
        └── tests/              # pipeline.rs (golden regression), worker_smoke.mjs (wasm smoke)
```

Where to look: orchestration `src/processing.rs`; OCR boundary `src/ocr.rs`; date/LLM boundary `src/date.rs`; business cleaning `src/report_cleaning.rs`; CLI flow `src/bin/tbel-pdf.rs` → `src/commands/`; JS interop `src/wasm_bridge.rs`; regression data `pdf_pipeline/tests/`. Omitted: `target/`, `.osgrep/`, and generated output directories.

## 4. Life of a Request / Primary Data Flow

### Native CLI: PDF → normalized XLSX + JSON contract

1. Trigger: `tbel-pdf pipeline --input-url <path-or-URL>` invocation.
2. Entry point: `src/bin/tbel-pdf.rs::main` (tokio runtime) → `src/commands/mod.rs::App::execute`.
3. Coordination: `src/commands/pipeline.rs::execute` resolves input, report type, and output paths.
4. Core or domain processing: `MistralOcrProvider` (behind `OcrProvider`) acquires OCR markdown; `ProcessingFacade` runs markdown preprocessing (`src/markdown.rs`), table extraction/validation (`src/table_extraction.rs`), and business cleaning with date normalization (`src/report_cleaning.rs`, `src/date.rs`).
5. Persistence or external interaction: Mistral OCR API (requires `MISTRAL_API_KEY`); PDF reading via `src/pdf.rs`.
6. Output or side effect: XLSX workbook, optional stage artifacts, `SuccessContract`/`FailureContract` JSON, and the process exit code from `src/contract/mod.rs`.

Architectural boundaries crossed: CLI feature gate → shared facade → provider trait → domain models → output contract.

Evidence: `src/bin/tbel-pdf.rs`, `src/commands/pipeline.rs`, `src/processing.rs`, `src/ocr.rs`

### Wasm/library: JS call → shared pipeline → JS result

1. Trigger: JavaScript invokes a `wasm_bindgen` export (`process_markdown` or `process_pdf`).
2. Entry point: `src/wasm_bridge.rs`.
3. Coordination: request decoded from `JsValue`; `process_pdf` builds a `MistralOcrProvider` from the request's OCR config.
4. Core or domain processing: the same `ProcessingFacade` path as the CLI.
5. Persistence or external interaction: Mistral API via wasm `reqwest` for `process_pdf`; the markdown variant runs offline.
6. Output or side effect: `JsSuccess` (tables plus optional XLSX bytes) or a `JsError` promise rejection.

Architectural boundaries crossed: `wasm32` target gate → facade → provider trait → domain models → JS serialization.

Evidence: `src/wasm_bridge.rs`, `src/processing.rs`, `pdf_pipeline/tbel-pdf/tests/worker_smoke.mjs`

## 5. Architectural Invariants & Constraints

- Rule: Keep `tbel-pdf` as the single crate in the workspace.
  - Rationale: library, CLI, and wasm surfaces share one module set; splitting requires an intentional architecture change.
  - Enforcement / Signals: `pdf_pipeline/Cargo.toml` (one member); `cdylib` + `rlib` crate types in `pdf_pipeline/tbel-pdf/Cargo.toml`.
- Rule: The `cli` feature must never build on `wasm32`; `wasm32` is library-only.
  - Rationale: CLI dependencies and native process behavior do not belong in the wasm artifact.
  - Enforcement / Signals: `compile_error!` guard in `src/lib.rs`; `required-features = ["cli"]` on the `[[bin]]` target.
- Rule: Native-only modules (`commands`, `contract`) stay out of `wasm32`; the wasm bridge stays out of native builds.
  - Rationale: each target compiles only the entrypoint surface it supports.
  - Enforcement / Signals: `#[cfg]` gates in `src/lib.rs`; wasm32 checks in `pdf_pipeline/ci-check.sh`.
- Rule: CLI and wasm entrypoints route shared extraction behavior through `ProcessingFacade` instead of duplicating pipeline stages.
  - Rationale: one orchestration boundary keeps native and JS behavior aligned.
  - Enforcement / Signals: `src/processing.rs`; call sites in `src/commands/pipeline.rs` and `src/wasm_bridge.rs`. Inferred convention — structural, not mechanically enforced.
- Rule: `src/models/` remains domain-data-only (no HTTP, filesystem, Tokio, scraper, PDF-reader, CLI, or wasm code).
  - Rationale: model types are the stable data vocabulary shared across targets and tests.
  - Enforcement / Signals: imports in `src/models/*.rs` are serde/std only; rule stated in `pdf_pipeline/tbel-pdf/AGENTS.md`. Convention, not mechanically enforced.
- Rule: External OCR access stays behind `OcrProvider`.
  - Rationale: production Mistral OCR and offline test doubles interchange without touching transformation logic.
  - Enforcement / Signals: `OcrProvider` trait with `MistralOcrProvider`, `MockOcrProvider`, `StubOcrProvider` in `src/ocr.rs`; `ProcessingFacade::process` takes `&dyn OcrProvider`.
- Rule: Date normalization stays behind `DateNormalizer` with an offline rule-based default.
  - Rationale: LLM-backed date translation is optional; cleaning must work deterministically without network access.
  - Enforcement / Signals: `DateNormalizer` trait, `RuleBasedDateNormalizer`, `StubDateNormalizer` in `src/date.rs`; consumed via trait in `src/report_cleaning.rs`.
- Rule: CLI JSON output is a contract; shape and exit-code changes ship together with `pdf_pipeline/docs/cli-contract.md` and contract/golden tests.
  - Rationale: downstream scripts and products depend on stable output.
  - Enforcement / Signals: `src/contract/mod.rs`, `pdf_pipeline/docs/cli-contract.md`, golden baselines under `pdf_pipeline/tests/golden/`.
- Rule: Regression tests run offline and deterministically; real OCR/network stays out of normal tests.
  - Rationale: reproducible CI and local runs while still covering OCR edge cases.
  - Enforcement / Signals: committed fixtures in `pdf_pipeline/tests/fixtures/`, mock/stub providers in `src/ocr.rs`, golden test harness `pdf_pipeline/tbel-pdf/tests/pipeline.rs`, rules in `pdf_pipeline/tests/AGENTS.md`.
- Rule: Native and wasm32 dependency sets stay separated via target-specific dependency tables, and `ci-check.sh` must validate all four builds (native lib, native CLI, wasm lib, wasm tests).
  - Rationale: shared crate, different platform constraints.
  - Enforcement / Signals: `[target.'cfg(...)']` sections in `pdf_pipeline/tbel-pdf/Cargo.toml`; build matrix in `pdf_pipeline/ci-check.sh`.
- Rule: Use the pinned Rust toolchain (1.94.0) for validation.
  - Rationale: reproducible formatting, linting, and wasm checks.
  - Enforcement / Signals: `pdf_pipeline/rust-toolchain.toml`; `rust-version = "1.94"` in `pdf_pipeline/tbel-pdf/Cargo.toml`.

## 6. Documentation Strategy

`ARCHITECTURE.md` (this file) owns the global architecture map, representative flows, and the invariant catalog. It does not duplicate local detail:

- `AGENTS.md` owns repository-wide agent operating rules and task-based context routing; `pdf_pipeline/AGENTS.md`, `pdf_pipeline/tbel-pdf/AGENTS.md`, and `pdf_pipeline/tests/AGENTS.md` own local deltas for their subtrees.
- `README.md` owns business purpose and normalization semantics; `pdf_pipeline/README.md` owns workspace commands, stage detail, and coverage; `pdf_pipeline/tbel-pdf/README.md` owns the crate API surface, features, and module map.
- `pdf_pipeline/docs/cli-contract.md` owns the machine-facing CLI JSON/exit-code contract.
- `okf/index.md` (with `okf/log.md` as the refresh log) owns agent-traversable concept knowledge (OCR boundary, table extraction, date rules, target boundaries); concept docs must stay citation-backed and in sync with code.
- No ADRs, runbooks, or `DESIGN.md` exist today; record cross-cutting boundary changes here and put command examples, schema details, and module-level usage in the nearest local doc. When documentation and code conflict (e.g., `AGENTS.md` still describes `src/adapters/` as a compatibility path, but it is not declared in the module tree), prefer executable evidence and correct the stale document.
