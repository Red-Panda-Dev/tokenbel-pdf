# TokenBel PDF Pipeline

Rust pipeline for turning Belarusian statutory financial PDF reports into normalized tabular data for TokenBel products.

The project handles OCR output from financial statements, finds report tables, normalizes report codes and values, and writes business-ready XLSX/JSON output.

## Business Purpose

The pipeline is built for Belarusian individual accounting reports submitted as PDFs. It extracts the accounting line-code table from each report and converts it to a stable format:

| Column | Meaning |
| ------ | ------- |
| `code` | statutory report line code, for example `010`, `300`, `700` |
| `MM.YYYY` | normalized reporting period, for example `01.2025` or `12.2025` |
| value cells | integer values in thousand BYN |

Normalization rules:

- Spaces and non-breaking spaces are removed from numbers: `5 622` -> `5622`.
- Parentheses become negative values: `(4 218)` -> `-4218`.
- Dash values become zero: `-` -> `0`.
- Decimal comma values are truncated to integer thousands: `1 986,99` -> `1986`.
- OCR-only numbering rows such as `1 | 2 | 3 | 4` are skipped.

## Supported Reports

| Report type | CLI value | Business rows currently covered by real fixtures |
| ----------- | --------- | ------------------------------------------------ |
| Balance sheet | `balance_sheet` | assets `110-300`, equity/liabilities `410-700` |
| Income statement | `income_statement` | rows `010-260` |
| Cash flow statement | `cashflow` or `cash_flow` | rows `020-140` |
| Equity changes statement | `statement_equity_changes` | supported by model/tests |

## Repository Layout

```text
tokenbel-pdf/
├── ARCHITECTURE.md
├── README.md
└── pdf_pipeline/
    ├── README.md
    ├── tests/fixtures/ocr/       # committed real OCR markdown fixtures
    ├── tests/golden/             # golden regression files
    └── tbel-pdf/                 # single Rust crate: library + CLI + wasm bridge
```

## Typical Commands

```bash
cd pdf_pipeline
cargo fmt --all --check
cargo clippy --workspace --all-targets --features cli -- -D warnings
cargo test --workspace --features cli
```

Live OCR requires `MISTRAL_API_KEY`:

```bash
export MISTRAL_API_KEY="your-key"
cargo run --features cli -- pipeline \
  --input-url https://cetco-resurs.by/file111.pdf \
  --report-type income_statement \
  --emit-stage-artifacts
```

## Real Regression Fixtures

The test suite includes committed OCR markdown from real reports:

- `pdf_pipeline/tests/fixtures/ocr/file111_income_statement.md`
- `pdf_pipeline/tests/fixtures/ocr/file122_balance_sheet.md`
- `pdf_pipeline/tests/fixtures/ocr/file133_cashflow.md`

These tests run offline and protect business-critical OCR edge cases: metadata merged into a financial table, blank OCR columns, split continuation labels, and multi-page report sections.

## Related TokenBel Projects

- `tokenbel.info`
- `dashboard.tokenbel.info`
