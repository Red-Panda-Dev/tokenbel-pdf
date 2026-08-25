# AGENTS.md

## Scope and inheritance

Applies to: `pdf_pipeline/tbel-pdf/` (the `tbel-pdf` crate).

Inherits workspace guidance from `../AGENTS.md` and repository-wide guidance from `../../AGENTS.md`. This file defines only local differences for this subtree.

## What lives here

```text
tbel-pdf/
├── Cargo.toml                   # Features: default=[], cli; crate-type cdylib+rlib
├── prompts/                     # Mistral prompt templates (e.g. date extraction)
├── src/
│   ├── lib.rs                   # Public API re-exports and wasm/cli guards
│   ├── processing.rs            # ProcessingFacade shared by CLI and wasm
│   ├── report_cleaning.rs       # CleanedTable and business normalization helpers
│   ├── wasm_bridge.rs           # wasm_bindgen exports; wasm32 only
│   ├── bin/                     # Native CLI entrypoint; cli feature only
│   ├── commands/                # clap command dispatch and XLSX export path
│   ├── contract/                # CLI JSON contracts and exit codes; native only
│   ├── models/                  # Pure domain types; no I/O
│   └── adapters/                # Back-compat re-exports only; not canonical
└── tests/                       # Integration test (pipeline.rs) + wasm smoke (worker_smoke.mjs)
```

Canonical top-level modules outside the tree above: `ocr.rs`, `pdf.rs`, `scraper.rs`, `date.rs`, `markdown.rs`, `table_extraction.rs`, `cleaner.rs`, `normalization.rs`, plus shared `error.rs` and `types.rs`.

## Local boundaries and invariants

- `src/models/` must remain pure data: no `reqwest`, `tokio`, `scraper`, `lopdf`, filesystem, or environment access.
- `src/processing.rs` is the shared orchestration entry point. CLI and wasm changes should flow through `ProcessingFacade` instead of duplicating pipeline stages.
- `src/report_cleaning.rs` is library code used by tests and exports; keep it free of CLI-only dependencies such as `clap`, `tracing-subscriber`, and native-only file output.
- `src/commands/` and `src/bin/` require the `cli` feature. The `cli` feature is not supported on wasm32.
- `src/contract/` is native-only but externally visible through CLI JSON. Keep it synchronized with `../docs/cli-contract.md`.
- `src/adapters/mod.rs` only re-exports top-level modules. Edit canonical implementations in the top-level module files.

## Safe change rules

- Adding a report type: update `models/report_type.rs`, parsing aliases, validation/cleaning rules as needed, and real or golden regression coverage.
- Adding an OCR provider: implement `OcrProvider` in `src/ocr.rs` and keep tests able to run with `MockOcrProvider`/`StubOcrProvider`.
- Changing cleaning/date/number behavior: update `report_cleaning.rs` and verify committed OCR fixtures still produce expected rows and headers.
- Adding wasm exports: place them in `wasm_bridge.rs`, use `serde_wasm_bindgen` for JS values, and keep native CLI dependencies out of wasm paths.
- Changing prompts in `prompts/`: verify the affected OCR fixture behavior or document why fixtures do not cover the prompt change.

## Validation

From `../` (`pdf_pipeline/`):

```bash
cargo test -p tbel-pdf --features cli
cargo clippy -p tbel-pdf --all-targets --features cli -- -D warnings
cargo check -p tbel-pdf --lib --target wasm32-unknown-unknown
```

Use the workspace `ci-check.sh` before merging cross-target or feature-gate changes.

## Nearby docs

| Doc | Path |
| --- | --- |
| Crate README | `README.md` |
| Workspace guide | `../AGENTS.md` |
| CLI contract | `../docs/cli-contract.md` |
| Architecture | `../../ARCHITECTURE.md` |
