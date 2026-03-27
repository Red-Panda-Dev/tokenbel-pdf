# PDF Pipeline (Rust)

**Generated:** 2026-03-27
**Commit:** updated from structure audit
**Branch:** main

## OVERVIEW

Rust crate for PDF extraction and OCR-backed report normalization. This subtree is the authoritative guide for Rust-specific work; the root `AGENTS.md` only routes into it.

## STRUCTURE

```text
pdf_pipeline/
├── tbel-pdf/            # Unified crate with models, adapters, CLI
│   ├── src/
│   │   ├── models/      # Domain types (ReportTable, PdfInput, etc.)
│   │   ├── adapters/    # Re-exports from parent modules
│   │   ├── commands/    # CLI commands (feature-gated)
│   │   ├── contract/    # CLI JSON contract types
│   │   ├── date.rs      # Date normalization
│   │   ├── markdown.rs  # Markdown preprocessing
│   │   ├── ocr.rs       # OCR providers
│   │   ├── pdf.rs       # PDF reader
│   │   ├── scraper.rs   # HTML scraping
│   │   ├── table_extraction.rs  # HTML/Markdown table parsing
│   │   ├── normalization.rs     # Code normalization
│   │   ├── cleaner.rs           # Report cleaning
│   │   ├── types.rs             # Shared type aliases
│   │   ├── error.rs             # Error types
│   │   └── lib.rs               # Public API re-exports
│   ├── src/bin/tbel-pdf.rs      # CLI binary entrypoint
│   ├── tests/           # Integration tests
│   └── prompts/         # Mistral prompt templates
├── tests/               # Golden files, fixtures, sample PDFs
│   ├── fixtures/manifest.json
│   └── golden/          # *.json + *.xlsx regression outputs
└── docs/                # Pipeline documentation
```

## WHERE TO LOOK

| Task                   | Location                           | Notes                                        |
| ---------------------- | ---------------------------------- | -------------------------------------------- |
| Domain models          | `tbel-pdf/src/models/`             | ReportTable, PdfInput, OcrOutput, etc.       |
| OCR providers          | `tbel-pdf/src/ocr.rs`              | MistralOcrProvider, MockOcrProvider          |
| Date normalization     | `tbel-pdf/src/date.rs`             | RuleBasedDateNormalizer                      |
| Table extraction       | `tbel-pdf/src/table_extraction.rs` | HTML/Markdown table parsing                  |
| Markdown preprocessing | `tbel-pdf/src/markdown.rs`         | LaTeX cleaning, table merging                |
| CLI behavior           | `tbel-pdf/src/commands/`           | Exit codes, command surface, contract output |
| JSON contract types    | `tbel-pdf/src/contract/`           | ExitCode, ErrorCode, SuccessContract, FailureContract |
| Regression assets      | `tests/`                           | Golden files and integration fixtures        |
| Public API surface     | `tbel-pdf/src/lib.rs`              | Re-exports of models, adapters, utilities    |

## CONVENTIONS

- Rust toolchain is pinned to `1.94.0` (`rust-toolchain.toml`).
- Single crate with modules; no separate core/adapters crates.
- Models in `src/models/` are pure domain types with no I/O.
- Adapters (OCR, PDF, HTTP) handle all external I/O.
- CLI is feature-gated behind `cli` feature — not compiled on wasm32.
- wasm32 target: library only (`compile_error!` guard prevents cli+wasm32).
- Respect CLI exit codes: `0` success, `1` usage, `2` pipeline, `3` provider.
- Use `MockOcrProvider` and `StubDateNormalizer` for unit testing.
- Testing: `rstest` for parametrized tests, golden files for regression.

## ANTI-PATTERNS

- Do not add I/O operations to model types.
- Do not change JSON contract without updating contract module tests.
- Do not add Rust-wide rules here that belong in root `AGENTS.md`.
- Do not split the crate into core/adapters/cli — it's unified by design.

## COMMANDS

```bash
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo run -p tbel-pdf --features cli -- --help
```

## NOTES

- `README.md` is the detailed operational reference for report types and fixtures.
- Golden files in `tests/golden/` are used for regression testing.
- Test fixtures are defined in `tests/fixtures/manifest.json`.
- CI runs via `ci-check.sh` (native lib+CLI check + wasm32 lib check).
