# AGENTS.md

## Repository overview

PDF extraction + OCR pipeline for Belarusian financial reports. Pure Rust project. Single Cargo workspace under `pdf_pipeline/` with one crate `tbel-pdf` that compiles as both a native CLI binary and a wasm32 library.

## Where to work

```text
tokenbel-pdf/
├── ARCHITECTURE.md              # Full architecture doc (authoritative for design decisions)
├── README.md                    # Business overview, normalization rules, supported reports
├── pdf_pipeline/                # Cargo workspace root — all Rust code lives here
│   ├── AGENTS.md                # Authoritative guide for Rust work (read this first)
│   ├── ci-check.sh              # CI verification matrix
│   ├── coverage.sh              # Library coverage script (cargo-llvm-cov, 70% threshold)
│   ├── docs/cli-contract.md     # JSON contract specification
│   ├── tests/
│   │   ├── *.pdf                # Sample financial reports (3 files)
│   │   ├── fixtures/
│   │   │   ├── ocr/             # Committed real OCR markdown (file111, file122, file133)
│   │   │   ├── manifest.json   # Test fixture definitions
│   │   │   └── source_of_truth/
│   │   └── golden/              # Regression baselines (10 JSON+XLSX pairs)
│   └── tbel-pdf/                # The single crate: lib + CLI + wasm bridge
│       ├── src/
│       │   ├── models/          # Pure domain types — no I/O
│       │   ├── adapters/        # Re-exports + adapter submodules (ocr, pdf, scraper, etc.)
│       │   ├── commands/        # CLI command dispatch (feature-gated)
│       │   ├── contract/        # JSON output schemas and exit codes
│       │   ├── bin/             # CLI binary entrypoint (feature-gated)
│       │   ├── processing.rs    # Shared processing facade (native + wasm32)
│       │   ├── report_cleaning.rs # Business-level table cleaning (library + CLI)
│       │   ├── wasm_bridge.rs   # wasm32 JS interop (feature-gated, not on native)
│       │   ├── ocr.rs, pdf.rs, scraper.rs, date.rs, markdown.rs
│       │   ├── table_extraction.rs, cleaner.rs, normalization.rs
│       │   ├── error.rs, types.rs
│       │   └── lib.rs           # Public API re-exports
│       ├── prompts/             # Mistral prompt templates
│       └── tests/               # Integration tests + wasm smoke test
```

## Architecture and boundaries

- **Dependency direction**: CLI → Contract → Processing Pipeline → Models + Adapters
- **Models are pure**: `src/models/` has zero I/O imports — no `reqwest`, `tokio`, filesystem access
- **Adapters own I/O**: OCR providers, PDF reader, HTML scraper — all external service boundaries
- **ProcessingFacade** (`processing.rs`) is the shared entry point used by both CLI and wasm bridge
- **report_cleaning.rs** is a library module with `clean_report_tables()`, `CleanedTable`, and normalization helpers — shared between CLI export and integration tests
- **Feature gates**: `cli` feature enables CLI binary; blocked on wasm32 via `compile_error!` guard
- **wasm32 target**: library-only, exposed through `wasm_bridge.rs` with `wasm_bindgen` exports

## Change rules

- Rust toolchain is pinned to **1.94.0** (`pdf_pipeline/rust-toolchain.toml`). Do not change.
- Do not split the crate into core/adapters/cli — it is unified by design.
- Do not add I/O to model types in `src/models/`.
- Do not change JSON contract types without updating contract tests and `docs/cli-contract.md`.
- `pdf_pipeline/AGENTS.md` is authoritative for Rust-specific guidance; this root file routes into it.

## Validation

```bash
cd pdf_pipeline && cargo fmt --all --check
cd pdf_pipeline && cargo clippy --workspace --all-targets --features cli -- -D warnings
cd pdf_pipeline && cargo test --workspace --features cli
cd pdf_pipeline && bash ci-check.sh
```

## Key docs

| Doc | Path |
|-----|------|
| Architecture decisions | `ARCHITECTURE.md` |
| Rust-specific guide | `pdf_pipeline/AGENTS.md` |
| Operational reference | `pdf_pipeline/README.md` |
| Crate-level docs | `pdf_pipeline/tbel-pdf/README.md` |
| CLI JSON contract | `pdf_pipeline/docs/cli-contract.md` |

## Gotchas

- `src/adapters/` contains both a `mod.rs` (re-exports from parent modules) and real submodules — the canonical implementations are the top-level modules (`ocr.rs`, `pdf.rs`, `scraper.rs`, `date.rs`, `markdown.rs`, `table_extraction.rs`).
- The crate publishes as `cdylib` + `rlib` — changing crate types affects the wasm build.
- Golden files in `pdf_pipeline/tests/golden/` are regression baselines; update intentionally.
- `MISTRAL_API_KEY` is required for real OCR; tests use `MockOcrProvider` / `StubOcrProvider` to run offline.
- `--features cli` is required for `cargo test`, `cargo clippy`, and CLI builds because integration tests and the pipeline command depend on it.
