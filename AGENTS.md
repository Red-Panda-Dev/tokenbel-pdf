# RUST WORKSPACE

**Generated:** 2026-03-26
**Commit:** 78698182
**Branch:** main

## OVERVIEW

`rust/` is the parent workspace for Rust code in this repo. Today it routes almost all work into `rust/pdf_pipeline/AGENTS.md`, while keeping the Rust-wide integration boundary in one place.

## STRUCTURE

```
rust/
└── pdf_pipeline/  # Cargo workspace for PDF extraction + OCR pipeline
```

## WHERE TO LOOK

| Task                 | Location                               | Notes                           |
| -------------------- | -------------------------------------- | ------------------------------- |
| PDF extraction logic | `rust/pdf_pipeline/tbel-pdf-core/`     | Core crate, no I/O              |
| OCR or file adapters | `rust/pdf_pipeline/tbel-pdf-adapters/` | External integrations           |
| CLI behavior         | `rust/pdf_pipeline/tbel-pdf-cli/`      | Python integration entrypoint   |
| Rust-specific rules  | `rust/pdf_pipeline/AGENTS.md`          | Commands, workspace constraints |

## CONVENTIONS

- Treat `rust/pdf_pipeline/AGENTS.md` as the authoritative child guide; this parent exists to make the Rust workspace discoverable from repo root.
- The current Rust surface is the PDF pipeline only, including shadow-mode integration with Python.
- Keep Rust-only build, lint, and test commands scoped to `rust/pdf_pipeline/`.

## ANTI-PATTERNS

- Do not copy crate-level guidance from `rust/pdf_pipeline/AGENTS.md` into this parent.
- Do not mix Python workflow rules into Rust crate docs unless they describe the integration boundary.
- Do not add new Rust top-level guidance here unless it applies to more than one Rust workspace.

## COMMANDS

```bash
cd rust/pdf_pipeline && cargo fmt --all
cd rust/pdf_pipeline && cargo clippy --workspace --all-targets -- -D warnings
cd rust/pdf_pipeline && cargo test --workspace
```

## NOTES

- `rust/pdf_pipeline/README.md` requires Rust `1.94.0` and documents shadow-mode env vars such as `TBEL_PDF_MODE` and `TBEL_RUST_CLI_PATH`.
- If additional Rust workspaces appear under `rust/`, expand this parent instead of duplicating repo-root guidance.
