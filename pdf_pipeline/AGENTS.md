# AGENTS.md

## Scope

This subtree is the Cargo workspace root. All Rust code in the repository lives here. This file is the authoritative guide for Rust-specific work; the root `AGENTS.md` only routes into it.

## What lives here

```text
pdf_pipeline/
├── Cargo.toml                   # Workspace manifest (single member: tbel-pdf)
├── rust-toolchain.toml          # Pinned toolchain: 1.94.0
├── ci-check.sh                  # Full CI matrix (native + wasm32 checks + smoke test)
├── README.md                    # Operational reference (CLI usage, report types, env vars)
├── docs/
│   └── cli-contract.md          # JSON contract specification for CLI output
├── tbel-pdf/                    # The unified crate
│   ├── Cargo.toml               # Crate manifest (features: default=[], cli)
│   ├── src/
│   │   ├── lib.rs               # Public API re-exports
│   │   ├── processing.rs        # ProcessingFacade — shared entry for CLI + wasm
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
│       ├── pipeline.rs          # Integration tests
│       └── worker_smoke.mjs     # Node.js wasm smoke test runner
└── tests/                       # Workspace-level test fixtures
    ├── *.pdf                    # Sample financial reports (3 files)
    ├── fixtures/
    │   ├── manifest.json        # Test fixture definitions
    │   └── source_of_truth/     # Reference data for regression checks
    ├── golden/                  # Regression baselines (10 JSON+XLSX pairs)
    └── output/                  # Test output artifacts (gitignored)
```

## Local boundaries and invariants

- **Models are pure**: `src/models/` must not import `reqwest`, `tokio`, `chrono::Local`, or any filesystem API. Only `serde`, `chrono::NaiveDate`, and std.
- **Adapter trait boundary**: All external HTTP goes through `OcrProvider` trait. No direct `reqwest` calls outside `ocr.rs`.
- **CLI ↔ Library split**: `cli` feature is gated with `compile_error!` on wasm32. Library code must not depend on `clap` or `tracing-subscriber`.
- **ProcessingFacade** is the single shared entry point. CLI commands and `wasm_bridge.rs` both call it. Do not bypass it with ad-hoc orchestration.
- **Adapters directory**: `src/adapters/mod.rs` only re-exports from top-level modules. Real adapter submodules (`adapters/ocr.rs`, `adapters/pdf.rs`, etc.) exist as parallel files but are currently secondary — the canonical implementations are the top-level modules (`ocr.rs`, `pdf.rs`, etc.).
- **Golden files** in `tests/golden/` are regression baselines. Each test case has a `.json` + `.xlsx` pair. Update intentionally when pipeline output legitimately changes.
- **Exit codes**: 0 = success, 1 = usage error, 2 = pipeline error, 3 = provider error. Defined in `src/contract/`.

## Safe change rules

- Adding a new report type: add variant to `ReportType` in `models/report_type.rs`, update `FromStr`, add validation rules in `cleaner.rs`, add golden test fixture.
- Adding an OCR provider: implement `OcrProvider` trait in `ocr.rs`, add provider config, update `ProcessingFacade` or CLI as needed.
- Changing JSON contract: update `src/contract/mod.rs` and `docs/cli-contract.md` simultaneously. Contract changes are breaking for external callers.
- Adding wasm exports: add functions in `wasm_bridge.rs` with `#[wasm_bindgen]`. All new exports must handle `JsValue` serialization via `serde_wasm_bindgen`.

## Validation

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo run -p tbel-pdf --features cli -- --help
bash ci-check.sh
```

`ci-check.sh` runs: native lib check, native tests, native CLI check, wasm32 lib check, wasm32 test compile check, and optionally a wasm smoke test (requires `wasm-bindgen` + `node`).

## Nearby docs

| Doc | Path |
|-----|------|
| Architecture decisions | `../../ARCHITECTURE.md` |
| CLI JSON contract spec | `docs/cli-contract.md` |
| Operational reference (CLI usage, env vars, report types) | `README.md` |
| Root agent guide | `../../AGENTS.md` |
