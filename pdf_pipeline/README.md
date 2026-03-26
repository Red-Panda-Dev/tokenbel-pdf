# TBel PDF Pipeline (Rust)

High-performance PDF processing pipeline for Belarusian financial reports. Extracts tables from PDF documents using OCR, normalizes data, and exports to XLSX.

## Overview

This Rust implementation processes Belarusian financial reports (Баланс, Отчёт о прибылях и убытках, etc.) with the following capabilities:

- **OCR Integration**: Mistral OCR provider with adapter boundary and mock fallback
- **Table Extraction**: Markdown-based table candidate detection
- **Data Normalization**: Belarusian financial format handling (spaces as thousand separators, commas as decimals, parentheses for negatives)
- **Date Normalization**: Mistral prompt-based extraction to `MM.YYYY` with cache and safe fallback
- **Export**: XLSX output format

## Architecture

```
rust/pdf_pipeline/
├── tbel-pdf/               # Unified crate (core + adapters + CLI)
│   ├── src/
│   │   ├── models/         # PdfInput, OcrOutput, ReportTable, CleanedReport, ReportType
│   │   ├── adapters/       # OcrProvider, PdfReader, date normalization, table extraction
│   │   ├── bin/            # CLI binary (feature-gated)
│   │   ├── cleaner.rs      # DataFrameCleaner, numeric parsing
│   │   ├── normalization.rs# Data transformation logic
│   │   └── lib.rs          # Re-exports all public APIs
│   └── prompts/            # Mistral prompt templates
└── tests/                  # Integration tests, fixtures, golden files
```

### Module Structure

| Module          | Purpose                       | Key Exports                                                     |
| --------------- | ----------------------------- | --------------------------------------------------------------- |
| `models`        | Domain models, errors         | `PipelineError`, `CleanedReport`, `ReportType`, `PdfInput`      |
| `adapters`      | External service abstractions | `OcrProvider`, `MistralOcrProvider`, `extract_table_candidates` |
| `cleaner`       | Data cleaning                 | `DataFrameCleaner`, numeric parsing functions                   |
| `normalization` | Table structure normalization | `normalize_table_structure`, `fix_table_columns`                |
| `cli`           | CLI binary (optional feature) | `tbel-pdf` binary, `Pipeline` struct                            |

## Supported Report Types

```rust
enum ReportType {
    BalanceSheet,              // Баланс
    IncomeStatement,           // Отчёт о прибылях и убытках
    StatementCashFlow,         // Отчёт о движении денежных средств
    StatementEquityChanges,    // Отчёт об изменениях капитала
}
```

## Prerequisites

- **Rust 1.94.0** (exact version required, see `rust-toolchain.toml`)
- `rustfmt` and `clippy` components

## Installation

```bash
# Navigate to the Rust workspace
cd rust/pdf_pipeline

# Build all crates
cargo build --release

# The binary will be at:
# target/release/tbel-pdf
```

### Development Build

```bash
# Debug build with symbols
cargo build

# Run clippy lints
cargo clippy --all-targets --all-features -- -D warnings

# Format code
cargo fmt

# Run tests
cargo test --all
```

## Usage

### Basic Pipeline

```bash
# Process a PDF directly from URL (no local download)
tbel-pdf pipeline --input-url https://example.com/2025.09_balance_sheet_company.pdf
```

The CLI always writes `<url_file_stem>_output.xlsx` into the current directory.

### CLI Arguments

| Argument                 | Required | Description                                          |
| ------------------------ | -------- | ---------------------------------------------------- |
| `--input-url <URL>`      | Yes      | Input PDF URL passed directly to Mistral OCR         |
| `--emit-contract <PATH>` | No       | Write JSON contract to file (also printed to stdout) |
| `--emit-stage-artifacts` | No       | Emit intermediate files for debugging                |

Report type is inferred from the URL filename and must contain one of:

- `balance_sheet`
- `income_statement`
- `statement_cash_flow`
- `statement_equity_changes`

### Runtime Configuration

```bash
# Required for real OCR and AI date normalization
export MISTRAL_API_KEY=<your_key>

# Optional overrides
export MISTRAL_OCR_MODEL=mistral-ocr-latest
export MISTRAL_DATE_MODEL=mistral-large-latest

```

Notes:

- If `MISTRAL_API_KEY` is missing, OCR falls back to `MockOcrProvider`.
- Date normalization also falls back safely: on model `ERROR`, invalid model output, or API failure, the original header is kept.
- Date prompt template is loaded from `prompts/financial_date_extraction.txt`.
- Rust passes `--input-url` directly to Mistral OCR (no local PDF download).

## Exit Codes

| Code | Name          | Description                                        |
| ---- | ------------- | -------------------------------------------------- |
| 0    | Success       | Pipeline completed successfully                    |
| 1    | UsageError    | Invalid CLI arguments                              |
| 2    | PipelineError | Processing error (no tables found, invalid layout) |
| 3    | ProviderError | OCR/external service error                         |

## Error Codes (JSON Contract)

When `--emit-contract` is used, errors are returned as JSON:

```json
{
    "error_code": "NoFinancialTablesFound",
    "error_message": "No valid financial tables detected in document",
    "document_id": "doc_123"
}
```

**Error Codes:**

- `NoFinancialTablesFound` - No tables meeting minimum dimensions (3 cols × 10 rows)
- `UnsupportedLayout` - Table structure not recognized
- `InvalidHeader` - Header row validation failed
- `DimensionValidationFailed` - Table too small or malformed
- `ProviderError` - OCR service failure
- `ParseError` - Data parsing failure

## Data Processing

### Belarusian Financial Format

The pipeline handles Belarusian-specific number formats:

| Format                     | Example      | Parsed Value |
| -------------------------- | ------------ | ------------ |
| Thousand separator (space) | `1 234 567`  | `1234567`    |
| Decimal separator (comma)  | `123,45`     | `123`        |
| Negative in parentheses    | `(1 000)`    | `-1000`      |
| NBSP handling              | `1\u{a0}234` | `1234`       |

### Header and Column Cleaning Parity

- For balance-style tables, if the first column is a descriptor column and code column is not first, the descriptor column is dropped before numeric cleaning.
- Date headers are normalized after cleaning and then propagated to exported `data_columns` and XLSX headers.

### Valid Financial Codes

- **Balance sheet codes**: `010`, `020`, ..., `090` (2-3 digits, leading zero)
- **Other codes**: `100` - `999` (3 digits)

### Table Validation

Minimum dimensions:

- 3 columns
- 10 rows

## Development

### Project Structure

```bash
# Run all tests
cargo test --all

# Run specific test
cargo test -p tbel-pdf test_report_type

# Run with coverage (requires cargo-tarpaulin)
cargo tarpaulin --out Html

# Check documentation
cargo doc --open --no-deps
```

### Adding New Report Types

1. Add variant to `ReportType` enum in `tbel-pdf/src/models/report_type.rs`
2. Update `FromStr` implementation
3. Add validation rules in `DataFrameCleaner`
4. Update tests

### Adding OCR Providers

1. Implement `OcrProvider` trait in `tbel-pdf/src/ocr.rs`
2. Add provider-specific configuration
3. Update CLI to accept provider selection

## Testing

```bash
# Unit tests
cargo test --all

# URL parity test (requires MISTRAL_API_KEY)
cargo test --workspace
```

## Dependencies

### Core

| Crate       | Version | Purpose                 |
| ----------- | ------- | ----------------------- |
| `thiserror` | 2.0     | Error derive macros     |
| `serde`     | 1.0     | Serialization           |
| `tracing`   | 0.1     | Logging/Instrumentation |
| `regex`     | 1.10+   | Pattern matching        |

### Adapters

| Crate         | Version | Purpose       |
| ------------- | ------- | ------------- |
| `scraper`     | 0.22    | HTML parsing  |
| `lopdf`       | 0.34    | PDF reading   |
| `chrono`      | 0.4     | Date handling |
| `async-trait` | 0.1     | Async traits  |

### CLI

| Crate    | Version | Purpose              |
| -------- | ------- | -------------------- |
| `clap`   | 4.5     | CLI argument parsing |
| `tokio`  | 1.0     | Async runtime        |
| `anyhow` | 1.0     | Error handling       |

## License

Proprietary - TBel.info

## Related

- API endpoints: `src/tbel/api/documents/`
