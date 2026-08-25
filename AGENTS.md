# AGENTS.md

## Repository overview

PDF extraction + OCR pipeline for Belarusian financial reports. The repository is a Rust project with one Cargo workspace under `pdf_pipeline/` and one crate, `tbel-pdf`, which builds as a native CLI and a wasm32 library.

## Where to work

```text
tokenbel-pdf/
├── ARCHITECTURE.md              # Authoritative architecture and invariants
├── README.md                    # Business overview and normalization rules
├── AGENTS.md                    # This global guide
├── okf/                         # Agent-traversable OKF v0.1 knowledge docs
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

## Context routing

Read only when the task touches that area:

- Cross-module changes, dependency direction, target boundaries → `ARCHITECTURE.md`
- Normalization semantics or supported report types → `README.md`
- CLI JSON shape, exit codes, downstream compatibility → `pdf_pipeline/docs/cli-contract.md`
- Workspace commands, pipeline stage details, coverage → `pdf_pipeline/README.md`
- Crate public API, feature flags, module map → `pdf_pipeline/tbel-pdf/README.md`
- Deep pipeline/domain concepts (OCR boundary, table extraction, date rules) → `okf/index.md`; append to `okf/log.md` when concepts are added or refreshed

## Change rules

- Rust toolchain is pinned to `1.94.0` in `pdf_pipeline/rust-toolchain.toml`; do not change it casually.
- Keep the crate unified. Do not split `tbel-pdf` into separate core/adapters/cli crates unless the architecture docs are intentionally rewritten.
- Prefer the smallest relevant subtree: workspace/build changes in `pdf_pipeline/`, crate code in `pdf_pipeline/tbel-pdf/`, regression data in `pdf_pipeline/tests/`.
- Do not commit generated outputs from `pdf_pipeline/tests/output/`, `target/`, or `.osgrep/`.
- `okf/` is generated knowledge documentation; keep concept docs citation-backed and in sync with code.

## Validation

Run from `pdf_pipeline/` unless noted:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --features cli -- -D warnings
cargo test --workspace --features cli
bash ci-check.sh
```

`ci-check.sh` also checks native library/CLI builds, wasm32 library builds, optional wasm smoke, and optional coverage when the required tools are installed.

## Repository-specific gotchas

- `pdf_pipeline/tbel-pdf/src/adapters/mod.rs` re-exports top-level modules for compatibility; canonical implementations are `src/ocr.rs`, `src/pdf.rs`, `src/date.rs`, etc.
- The crate publishes both `cdylib` and `rlib`; changing crate types affects wasm consumers.
- Golden regression files are paired `.json` + `.xlsx` baselines under `pdf_pipeline/tests/golden/` and should change only intentionally.
- `--features cli` is required for CLI builds and workspace integration tests.
