# tbel-pdf crate

`tbel-pdf` is the Rust crate behind the TokenBel PDF pipeline. It turns Belarusian financial report PDFs, or OCR markdown already produced from those PDFs, into normalized financial rows suitable for XLSX export, JSON contracts, and downstream analytics.

The crate is intentionally unified: the same library powers the native CLI and the wasm32 library build. The `cli` feature only adds command-line dependencies and export plumbing.

## Business Output

The normalized business table shape is:

| Column | Meaning |
| ------ | ------- |
| `code` | Belarusian statutory report row code, usually 2 or 3 digits |
| `MM.YYYY` | Period value parsed from Russian/Belarusian date headers |

Values are normalized as integer thousand BYN values:

| Source OCR value | Normalized value |
| ---------------- | ---------------- |
| `5 622` | `5622` |
| `1 986,99` | `1986` |
| `(4 218)` | `-4218` |
| `-` | `0` |

The current real fixtures cover:

| File | CLI report type | Business report |
| ---- | --------------- | --------------- |
| `file111` | `income_statement` | Profit and loss statement |
| `file122` | `balance_sheet` | Balance sheet |
| `file133` | `cashflow` | Cash flow statement |

## Public API

Use `ProcessingFacadeBuilder` when you need the same extraction path as the CLI.

```rust
use tbel_pdf::{clean_report_tables, ProcessingFacadeBuilder, ReportType};

let result = ProcessingFacadeBuilder::default()
    .report_type(ReportType::IncomeStatement)
    .build()
    .process_markdown(ocr_markdown, 2, "file111".to_string())?;

let cleaned = clean_report_tables(&result);
```

`clean_report_tables` returns `Vec<CleanedTable>`, where each table contains normalized `headers` and string `rows`. The cleaner handles the OCR issues seen in real CETCO reports:

- Company metadata merged into the first markdown table before the financial header.
- Extra blank OCR columns between indicator name and `Код строки`.
- Page-level table splits that must keep one shared header shape.
- Russian date headers such as `За январь-декабрь 2024 года`.
- Parenthesized negative numbers and dash-as-zero placeholders.

## CLI Feature

Build the native CLI with the `cli` feature:

```bash
cargo run -p tbel-pdf --features cli -- pipeline \
  --input-url https://cetco-resurs.by/file111.pdf \
  --report-type income_statement \
  --output-xlsx file111_output.xlsx \
  --emit-stage-artifacts
```

`MISTRAL_API_KEY` is required for live OCR:

```bash
export MISTRAL_API_KEY="your-key-here"
```

Accepted report type values include:

| Business report | CLI values |
| --------------- | ---------- |
| Balance sheet | `balance_sheet`, `balance`, `баланс` |
| Profit and loss | `income_statement`, `income`, `profit_loss` |
| Cash flow | `cashflow`, `cash_flow`, `cash` |
| Equity changes | `equity_changes`, `equity`, `capital_changes` |

## Offline Regression Fixtures

Real OCR markdown fixtures live outside the crate at workspace level:

```text
../tests/fixtures/ocr/file111_income_statement.md
../tests/fixtures/ocr/file122_balance_sheet.md
../tests/fixtures/ocr/file133_cashflow.md
```

The integration tests parse these markdown files without calling Mistral. They assert row counts, expected row codes, period headers, and normalized values for the known OCR edge cases.

Run the crate tests with:

```bash
cargo test -p tbel-pdf --features cli
```

## Feature Flags

| Flag | Description |
| ---- | ----------- |
| `cli` | Enables command dispatch, XLSX export, clap, tokio runtime, and tracing setup |

The `cli` feature is rejected on wasm32. The wasm build is library-only.

## Module Map

| Module | Purpose |
| ------ | ------- |
| `processing` | Shared facade used by CLI and wasm bridge |
| `table_extraction` | Markdown/HTML table detection and financial table validation |
| `report_cleaning` | Business-level row-code, date, and numeric normalization |
| `ocr` | OCR provider trait and Mistral/mock/stub providers |
| `commands` | CLI subcommand implementation, behind native feature gates |
| `contract` | CLI success/failure JSON contracts and exit codes |

## Validation

From the workspace root (`pdf_pipeline/`):

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --features cli -- -D warnings
cargo test --workspace --features cli
```

## License

MIT
