# AGENTS.md

## Scope

This subtree is the Cargo workspace root. All Rust code in the repository lives here. This file is the authoritative guide for Rust-specific work; the root `AGENTS.md` only routes into it.

## What lives here

```text
pdf_pipeline/
├── Cargo.toml                   # Workspace manifest (single member: tbel-pdf)
├── rust-toolchain.toml          # Pinned toolchain: 1.94.0
├── ci-check.sh                  # Full CI matrix (native + wasm32 + optional coverage)
├── coverage.sh                  # Library coverage script (cargo-llvm-cov, 70% threshold)
├── docs/
│   └── cli-contract.md          # JSON contract specification for CLI output
├── tbel-pdf/                    # The unified crate
│   ├── Cargo.toml               # Crate manifest (features: default=[], cli)
│   ├── src/
│   │   ├── lib.rs               # Public API re-exports
│   │   ├── processing.rs        # ProcessingFacade — shared entry for CLI + wasm
│   │   ├── report_cleaning.rs   # Business-level table cleaning (CleanedTable, normalize helpers)
│   │   ├── wasm_bridge.rs       # wasm_bindgen exports (only compiled on wasm32)
│   │   ├── models/              # Pure domain types, zero I/O
│   │   │   ├── report_table.rs  # ReportTable, TableCell
│   │   │   ├── report_type.rs   # ReportType enum (4 variants)
│   │   │   ├── pdf_input.rs     # PdfInput (Url | Bytes)
│   │   │   ├── ocr_output.rs    # OcrOutput
│   │   │   ├── cleaned_report.rs # CleanedReport, DataColumn
│   │   │   └── code_value.rs    # CodeValue
│   │   ├── adapters/            # Re-exports + parallel adapter submodules
│   │   │   └── mod.rs           # Re-exports from top-level modules (backwards compat)
│   │   ├── commands/            # CLI subcommand dispatch (feature-gated)
│   │   │   ├── mod.rs           # clap App/Commands definitions
│   │   │   └── pipeline.rs      # Full pipeline: OCR → clean → XLSX export
│   │   ├── contract/            # ExitCode, ErrorCode, SuccessContract, FailureContract
│   │   ├── bin/tbel-pdf.rs      # CLI binary entrypoint (feature-gated)
│   │   ├── ocr.rs               # OcrProvider trait, MistralOcrProvider, Mock/Stub providers
│   │   ├── pdf.rs               # PdfReader
│   │   ├── scraper.rs           # HTML parsing, company name extraction
│   │   ├── date.rs              # DateNormalizer trait, RuleBasedDateNormalizer
│   │   ├── markdown.rs          # Markdown preprocessing, LaTeX cleaning, table merging
│   │   ├── table_extraction.rs  # Table candidate extraction, financial table validation
│   │   ├── cleaner.rs           # DataFrameCleaner, Belarusian number format parsing
│   │   ├── normalization.rs     # Table structure normalization
│   │   ├── error.rs             # PipelineError, Result
│   │   └── types.rs             # PdfError
│   ├── prompts/                 # Mistral prompt templates
│   └── tests/
│       ├── pipeline.rs          # Integration tests (including real OCR fixture tests)
│       └── worker_smoke.mjs     # Node.js wasm smoke test runner
└── tests/                       # Workspace-level test fixtures
    ├── *.pdf                    # Sample financial reports (3 files)
    ├── fixtures/
    │   ├── manifest.json        # Test fixture definitions
    │   ├── ocr/                 # Committed real OCR markdown (3 files)
    │   └── source_of_truth/     # Reference data for regression checks
    ├── golden/                  # Regression baselines (10 JSON+XLSX pairs)
    └── output/                  # Test output artifacts (gitignored)
```

## Local boundaries and invariants

- **Models are pure**: `src/models/` must not import `reqwest`, `tokio`, `chrono::Local`, or any filesystem API. Only `serde`, `chrono::NaiveDate`, and std.
- **Adapter trait boundary**: All external HTTP goes through `OcrProvider` trait. No direct `reqwest` calls outside `ocr.rs`.
- **CLI ↔ Library split**: `cli` feature is gated with `compile_error!` on wasm32. Library code must not depend on `clap` or `tracing-subscriber`.
- **ProcessingFacade** is the single shared entry point. CLI commands and `wasm_bridge.rs` both call it. Do not bypass it with ad-hoc orchestration.
- **report_cleaning.rs** is a library module shared between CLI export and integration tests. It depends on `ProcessingResult` from `processing.rs` and must not import CLI-only dependencies (`clap`, `rust_xlsxwriter`).
- **Adapters directory**: `src/adapters/mod.rs` only re-exports from top-level modules. Canonical implementations are the top-level modules (`ocr.rs`, `pdf.rs`, etc.).
- **Golden files** in `tests/golden/` are regression baselines. Each test case has a `.json` + `.xlsx` pair. Update intentionally when pipeline output legitimately changes.
- **Exit codes**: 0 = success, 1 = usage error, 2 = pipeline error, 3 = provider error. Defined in `src/contract/`.

## Safe change rules

- Adding a new report type: add variant to `ReportType` in `models/report_type.rs`, update `FromStr`, add validation rules in `cleaner.rs`, add golden test fixture.
- Adding an OCR provider: implement `OcrProvider` trait in `ocr.rs`, add provider config, update `ProcessingFacade` or CLI as needed.
- Changing JSON contract: update `src/contract/mod.rs` and `docs/cli-contract.md` simultaneously. Contract changes are breaking for external callers.
- Adding wasm exports: add functions in `wasm_bridge.rs` with `#[wasm_bindgen]`. All new exports must handle `JsValue` serialization via `serde_wasm_bindgen`.
- Changing cleaning logic: edit `report_cleaning.rs`. Both CLI and integration tests consume `clean_report_tables()`, so changes propagate. Run real-fixture tests to verify.

## Validation

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --features cli -- -D warnings
cargo test --workspace --features cli
cargo run -p tbel-pdf --features cli -- --help
bash ci-check.sh
```

`ci-check.sh` runs: native lib check, native tests, native CLI check, wasm32 lib check, wasm32 test compile check, optional wasm smoke test, and optional library coverage gate.

`coverage.sh` runs `cargo llvm-cov -p tbel-pdf --lib` with HTML report and fails below 70% line coverage. Coverage measures library code only (excludes `--features cli`).

## Nearby docs

| Doc | Path |
|-----|------|
| Architecture decisions | `../../ARCHITECTURE.md` |
| CLI JSON contract spec | `docs/cli-contract.md` |
| Operational reference (CLI usage, env vars, report types) | `README.md` |
| Crate API and fixtures | `tbel-pdf/README.md` |
| Root agent guide | `../../AGENTS.md` |
