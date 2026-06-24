# AGENTS.md

## Repository overview

PDF extraction + OCR pipeline for Belarusian financial reports. The repository is a Rust project with one Cargo workspace under `pdf_pipeline/` and one crate, `tbel-pdf`, which builds as a native CLI and a wasm32 library.

## Where to work

```text
tokenbel-pdf/
├── ARCHITECTURE.md              # Authoritative architecture and invariants
├── README.md                    # Business overview and normalization rules
├── AGENTS.md                    # This global guide
└── pdf_pipeline/                # Cargo workspace root; all Rust code and tests
    ├── AGENTS.md                # Workspace/build/test guidance
    ├── docs/cli-contract.md     # Stable CLI JSON contract
    ├── tests/                   # PDFs, OCR fixtures, golden regressions
    │   └── AGENTS.md            # Fixture and golden-file rules
    └── tbel-pdf/                # Unified crate: lib + CLI + wasm bridge
        ├── AGENTS.md            # Crate-local boundaries and change rules
        ├── prompts/             # Mistral prompt templates
        ├── src/                 # Rust library, CLI, wasm, pipeline modules
        └── tests/               # Crate integration and wasm smoke tests
```

## Architecture and boundaries

- Main dependency direction: CLI/wasm entrypoints → `ProcessingFacade` + `report_cleaning` → pipeline modules + adapters → pure models.
- `pdf_pipeline/tbel-pdf/src/models/` is pure domain data. Do not add HTTP, filesystem, Tokio, scraper, or PDF-reader logic there.
- External OCR must stay behind `OcrProvider` in `src/ocr.rs`; offline tests use mock/stub providers.
- `ProcessingFacade` in `src/processing.rs` is the shared orchestration path for CLI and wasm callers.
- Native CLI code is feature-gated with `cli`; wasm32 is library-only and rejects `cli` at compile time.
- CLI JSON output is a contract. If `src/contract/mod.rs` changes, update `pdf_pipeline/docs/cli-contract.md` and contract tests together.

## Change rules

- Rust toolchain is pinned to `1.94.0` in `pdf_pipeline/rust-toolchain.toml`; do not change it casually.
- Keep the crate unified. Do not split `tbel-pdf` into separate core/adapters/cli crates unless the architecture docs are intentionally rewritten.
- Prefer the smallest relevant subtree: workspace/build changes in `pdf_pipeline/`, crate code in `pdf_pipeline/tbel-pdf/`, regression data in `pdf_pipeline/tests/`.
- Do not commit generated outputs from `pdf_pipeline/tests/output/`, `target/`, or `.osgrep/`.
- Live OCR requires `MISTRAL_API_KEY`; normal tests should remain offline and use committed fixtures.

## Validation

Run from `pdf_pipeline/` unless noted:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --features cli -- -D warnings
cargo test --workspace --features cli
bash ci-check.sh
```

`ci-check.sh` also checks native library/CLI builds, wasm32 library builds, optional wasm smoke, and optional coverage when the required tools are installed.

## Key docs

| Doc | Path |
| --- | --- |
| Architecture and invariants | `ARCHITECTURE.md` |
| Business overview | `README.md` |
| Workspace operation | `pdf_pipeline/README.md` |
| Crate API and fixtures | `pdf_pipeline/tbel-pdf/README.md` |
| CLI JSON contract | `pdf_pipeline/docs/cli-contract.md` |

## Repository-specific gotchas

- `pdf_pipeline/tbel-pdf/src/adapters/mod.rs` re-exports top-level modules for compatibility; canonical implementations are `src/ocr.rs`, `src/pdf.rs`, `src/date.rs`, etc.
- The crate publishes both `cdylib` and `rlib`; changing crate types affects wasm consumers.
- Golden regression files are paired `.json` + `.xlsx` baselines under `pdf_pipeline/tests/golden/` and should change only intentionally.
- `--features cli` is required for CLI builds and workspace integration tests.
