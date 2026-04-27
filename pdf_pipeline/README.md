# TBel PDF Pipeline

Cargo workspace for extracting Belarusian financial reports from PDF/OCR output and exporting normalized business tables.

The workspace contains one Rust crate, `tbel-pdf`, compiled as both a native CLI and a wasm-compatible library.

## What The Pipeline Produces

For each financial PDF, the pipeline produces rows shaped like:

```text
code | 01.2025 | 01.2024
010  | 5622    | 6042
020  | -4218   | -4213
```

Business conventions:

- `code` is the statutory report line code.
- Date columns are normalized from Russian/Belarusian headers to `MM.YYYY`.
- Values are integer thousands of BYN.
- Parenthesized values are negative.
- Dash values are normalized to `0`.
- Descriptor and OCR-only helper columns are removed.

## Supported Report Types

| CLI value | Business report | Example fixture |
| --------- | --------------- | --------------- |
| `balance_sheet` | Бухгалтерский баланс | `file122_balance_sheet.md` |
| `income_statement` or `income` | Отчет о прибылях и убытках | `file111_income_statement.md` |
| `cashflow` or `cash_flow` | Отчет о движении денежных средств | `file133_cashflow.md` |
| `statement_equity_changes` | Отчет об изменениях капитала | golden tests |

## Pipeline Stages

1. OCR the PDF with Mistral and receive markdown.
2. Preprocess markdown by cleaning LaTeX fragments and joining continuation lines.
3. Extract markdown tables into `ReportTable` values.
4. Validate financial tables using headers and code-like rows.
5. Clean business rows by aligning the code column, removing blank OCR columns, skipping numbering rows, and normalizing dates/numbers.
6. Export one flattened XLSX sheet and a JSON success/error contract.
7. Optionally write stage artifacts for debugging.

## OCR Edge Cases Covered

The real fixtures lock down these observed OCR patterns:

- Company metadata merged into the first financial table before the real header.
- Blank OCR columns between label and code columns.
- Numbering rows such as `1 | 2 | 3 | 4`.
- Multi-line labels where continuation text appears outside the markdown row.
- Continuation tables split across pages or separated by signatures.

## Workspace Layout

```text
pdf_pipeline/
├── Cargo.toml
├── README.md
├── ci-check.sh
├── docs/cli-contract.md
├── tests/
│   ├── fixtures/ocr/       # committed real OCR markdown fixtures
│   ├── fixtures/source_of_truth/
│   └── golden/
└── tbel-pdf/
    ├── README.md
    ├── prompts/
    ├── src/
    └── tests/
```

## CLI Usage

```bash
export MISTRAL_API_KEY="your-key"

cargo run --features cli -- pipeline \
  --input-url https://cetco-resurs.by/file111.pdf \
  --report-type income_statement \
  --output-xlsx file111_output.xlsx \
  --emit-stage-artifacts \
  --emit-contract file111_contract.json
```

Arguments:

| Argument | Required | Description |
| -------- | -------- | ----------- |
| `--input-url <URL_OR_PATH>` | yes | PDF URL or path used as OCR input |
| `--report-type <TYPE>` | no | Explicit report type; inferred from filename when omitted |
| `--output-xlsx <PATH>` | no | XLSX destination; defaults to `<input_stem>_output.xlsx` |
| `--emit-stage-artifacts` | no | Writes OCR markdown, extracted tables, cleaned tables, and metadata |
| `--emit-contract <PATH>` | no | Writes the JSON contract to a file; contract is also printed |

## Stage Artifacts

With `--emit-stage-artifacts`, the CLI writes `<input_stem>_artifacts/`:

| File | Purpose |
| ---- | ------- |
| `ocr_output.md` | raw OCR markdown returned by Mistral |
| `tables.json` | extracted `ReportTable` candidates after preprocessing/validation |
| `cleaned_tables.json` | final normalized business rows before XLSX export |
| `meta.json` | document id, report type, table counts, and XLSX path |

## Testing

Offline validation does not require a Mistral key:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --features cli -- -D warnings
cargo test --workspace --features cli
```

## Coverage

Coverage uses `cargo-llvm-cov` and measures library code only. The CLI feature is intentionally excluded so the number tracks reusable pipeline modules rather than command/export plumbing.

Install the tool once:

```bash
cargo install cargo-llvm-cov
```

Run coverage from this workspace:

```bash
bash coverage.sh
```

The script runs `cargo llvm-cov -p tbel-pdf --lib`, prints line/function/region coverage, shows missing lines, writes the HTML report to `target/coverage/index.html`, and fails if line coverage is below `70%`.

Override the threshold locally when needed:

```bash
COVERAGE_FAIL_UNDER_LINES=80 bash coverage.sh
```

`ci-check.sh` also runs the same library coverage gate when `cargo-llvm-cov` is installed. If the tool is missing, CI prints a skip message and continues.

Real OCR smoke runs require `MISTRAL_API_KEY`:

```bash
cargo run --features cli -- pipeline --input-url https://cetco-resurs.by/file111.pdf --report-type income_statement --emit-stage-artifacts
cargo run --features cli -- pipeline --input-url https://cetco-resurs.by/file122.pdf --report-type balance_sheet --emit-stage-artifacts
cargo run --features cli -- pipeline --input-url https://cetco-resurs.by/file133.pdf --report-type cashflow --emit-stage-artifacts
```

## Development Notes

- Rust toolchain is pinned to `1.94.0` in `rust-toolchain.toml`.
- `ProcessingFacade` is the shared entry point for native CLI and wasm bridge.
- Models in `src/models/` are pure and must not own I/O.
- OCR, PDF, scraper, and export boundaries live outside model types.
- JSON contract changes must be reflected in `docs/cli-contract.md`.

## Exit Codes

| Code | Meaning |
| ---- | ------- |
| `0` | success |
| `1` | usage error |
| `2` | pipeline error |
| `3` | provider/OCR error |
