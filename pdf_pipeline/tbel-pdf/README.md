# TBel PDF

A unified Rust library for processing Belarusian financial PDF reports with OCR, table extraction, and data normalization.

## Features

- **OCR Integration**: Mistral OCR provider with mock fallback for testing
- **Table Extraction**: Markdown-based table candidate detection
- **Data Normalization**: Belarusian financial format handling
- **Date Normalization**: Mistral prompt-based extraction to `MM.YYYY`
- **Export**: JSON and XLSX output formats

## Installation

```toml
[dependencies]
tbel-pdf = "0.1"
```

### CLI Binary

```bash
cargo install tbel-pdf --features cli
```

## Usage

### Library

```rust
use tbel_pdf::{Pipeline, ReportType, PdfInput};

let pipeline = Pipeline::new();
let report = pipeline.process(input, ReportType::BalanceSheet).await?;
```

### CLI

```bash
tbel-pdf pipeline --input-url https://example.com/balance_sheet.pdf
```

## Supported Report Types

- `BalanceSheet` - Баланс
- `IncomeStatement` - Отчёт о прибылях и убытках
- `StatementCashFlow` - Отчёт о движении денежных средств
- `StatementEquityChanges` - Отчёт об изменениях капитала

## Feature Flags

| Flag  | Description                            |
| ----- | -------------------------------------- |
| `cli` | CLI binary (clap, anyhow, xlsx, tokio) |

## Configuration

```bash
export MISTRAL_API_KEY=your_key_here
```

## License

MIT

## Связанные проекты TokenBel

- 🔗 **[Лендинг - tokenbel.info](https://tokenbel.info/)**
- 🔗 **[Агрегатор токенов - dashboard.tokenbel.info](https://dashboard.tokenbel.info/)**
