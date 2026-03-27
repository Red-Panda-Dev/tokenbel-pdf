# TOKENBEL-PDF

**Generated:** 2026-03-27
**Commit:** updated from structure audit
**Branch:** main

## OVERVIEW

PDF extraction + OCR pipeline for Belarusian financial reports. Pure Rust project — no Python, JS, or Go code. Single Cargo workspace under `pdf_pipeline/` with one unified crate `tbel-pdf`.

## STRUCTURE

```
tokenbel-pdf/
├── AGENTS.md          # This file — repo root guide
├── ARCHITECTURE.md    # Full architecture doc (~280 lines)
├── README.md          # Russian overview
├── LICENSE            # MIT
└── pdf_pipeline/      # Cargo workspace root
    ├── Cargo.toml     # workspace: members=["tbel-pdf"]
    ├── rust-toolchain.toml  # pinned 1.94.0
    ├── ci-check.sh    # CI script
    ├── AGENTS.md      # Pipeline-level guide (authoritative for Rust work)
    ├── docs/          # CLI contract docs
    ├── tbel-pdf/      # Unified crate (models, adapters, CLI)
    └── tests/         # Golden files, fixtures, sample PDFs
```

## WHERE TO LOOK

| Task                 | Location                    | Notes                          |
| -------------------- | --------------------------- | ------------------------------ |
| PDF extraction logic | `pdf_pipeline/tbel-pdf/src/` | Core logic, no I/O in models  |
| CLI behavior         | `pdf_pipeline/tbel-pdf/src/commands/` | Exit codes, JSON contract |
| OCR providers        | `pdf_pipeline/tbel-pdf/src/ocr.rs` | MistralOcrProvider, mock   |
| Test fixtures        | `pdf_pipeline/tests/`       | Golden files, manifest.json    |
| Rust-specific rules  | `pdf_pipeline/AGENTS.md`    | Authoritative child guide      |

## CONVENTIONS

- Rust toolchain pinned to `1.94.0`.
- Single unified crate — NOT split into core/adapters/cli.
- `pdf_pipeline/AGENTS.md` is the authoritative guide for Rust work; this root file is a routing layer.
- CLI is feature-gated (`cli` feature), not available on wasm32.

## ANTI-PATTERNS

- Do not duplicate `pdf_pipeline/AGENTS.md` content here.
- Do not add new top-level guidance unless it applies outside `pdf_pipeline/`.

## COMMANDS

```bash
cd pdf_pipeline && cargo fmt --all
cd pdf_pipeline && cargo clippy --workspace --all-targets -- -D warnings
cd pdf_pipeline && cargo test --workspace
```
