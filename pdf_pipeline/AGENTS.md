# PDF Pipeline (Rust)

**Generated:** 2026-03-26
**Commit:** 78698182
**Branch:** main

## OVERVIEW

Rust crate for PDF extraction and OCR-backed report normalization. This subtree is the authoritative guide for Rust-specific work; the parent `rust/AGENTS.md` only routes into it.

## STRUCTURE

```text
rust/pdf_pipeline/
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
│   │   └── ...
│   ├── tests/           # Integration tests
│   └── prompts/         # Mistral prompt templates
├── tests/               # Golden files, fixtures, sample PDFs
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
| Regression assets      | `tests/`                           | Golden files and integration fixtures        |

## CONVENTIONS

- Rust toolchain is pinned to `1.94.0`.
- Single crate with modules; no separate core/adapters crates.
- Models in `src/models/` are pure domain types with no I/O.
- Adapters (OCR, PDF, HTTP) handle all external I/O.
- Respect CLI exit codes: `0` success, `1` usage, `2` pipeline, `3` provider.
- Use `MockOcrProvider` and `StubDateNormalizer` for unit testing.

## ANTI-PATTERNS

- Do not add I/O operations to model types.
- Do not change JSON contract without updating contract module tests.
- Do not add Rust-wide rules here that belong in `rust/AGENTS.md`.

## COMMANDS

```bash
cd rust/pdf_pipeline && cargo fmt --all
cd rust/pdf_pipeline && cargo clippy --workspace --all-targets -- -D warnings
cd rust/pdf_pipeline && cargo test --workspace
cd rust/pdf_pipeline && cargo run -p tbel-pdf --features cli -- --help
```

## NOTES

- `README.md` is the detailed operational reference for report types and fixtures.
- Golden files in `tests/golden/` are used for regression testing.
- Test fixtures are defined in `tests/fixtures/manifest.json`.
