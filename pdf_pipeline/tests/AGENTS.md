# AGENTS.md

## Scope and inheritance

Applies to: `pdf_pipeline/tests/` (workspace regression inputs and baselines).

Inherits workspace guidance from `../AGENTS.md` and repository-wide guidance from `../../AGENTS.md`. This file defines only local differences for this subtree.

## What lives here

```text
tests/
├── *.pdf                        # Sample Belarusian financial reports
├── fixtures/
│   ├── manifest.json            # Fixture definitions consumed by tests
│   ├── manifest.lock.json       # Locked fixture metadata
│   ├── ocr/                     # Committed OCR markdown used for offline tests
│   └── source_of_truth/         # Reference XLSX data for real reports
├── golden/                      # Paired JSON + XLSX regression baselines
└── output/                      # Local test artifacts; gitignored except .gitkeep
```

## Local boundaries and invariants

- Tests must be able to run offline from committed PDFs/OCR markdown. Do not make normal regression tests depend on `MISTRAL_API_KEY`.
- `fixtures/ocr/` captures real Mistral OCR edge cases: metadata merged into tables, blank OCR columns, numbering rows, split labels, and page continuations.
- Each golden case in `golden/` is a regression baseline. JSON and XLSX files are paired and should be updated together when behavior intentionally changes.
- `output/` is for local artifacts only; do not rely on its contents in tests.

## Safe change rules

- When adding a real fixture, add the OCR markdown and update `fixtures/manifest.json`/`manifest.lock.json` if the tests consume the manifest.
- When changing extraction or cleaning behavior, inspect the corresponding golden diff instead of blindly overwriting baselines.
- Keep fixture and golden filenames descriptive enough to identify report type and scenario, following existing names such as `file111_income_statement.md` (OCR fixture) and `under_dimension_rows.json` (golden case).
- Do not remove source-of-truth XLSX files unless the test coverage using them is removed or replaced intentionally.

## Validation

From `../` (`pdf_pipeline/`):

```bash
cargo test --workspace --features cli
cargo test -p tbel-pdf --features cli --test pipeline
```

Run `bash ci-check.sh` after broad fixture or golden changes.

## Nearby docs

| Doc | Path |
| --- | --- |
| Workspace README | `../README.md` |
| Crate fixture notes | `../tbel-pdf/README.md` |
| Workspace guide | `../AGENTS.md` |
