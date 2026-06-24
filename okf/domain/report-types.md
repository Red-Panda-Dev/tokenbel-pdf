---
type: Domain Enum
title: Report types
description: The four supported Belarusian financial report kinds and their aliases.
resource: pdf_pipeline/tbel-pdf/src/models/report_type.rs
source_paths:
  - pdf_pipeline/tbel-pdf/src/models/report_type.rs
  - pdf_pipeline/docs/cli-contract.md
  - pdf_pipeline/tbel-pdf/src/processing.rs
  - README.md
confidence: observed
---

# Report types

The pipeline targets Belarusian individual accounting reports. `ReportType`
(`src/models/report_type.rs`) enumerates the four supported kinds, each with a
Russian label, a snake_case stem, and filename aliases [1].

## Variants

| Variant                  | Russian                          | snake_case                  | Aliases                                            |
|--------------------------|----------------------------------|-----------------------------|----------------------------------------------------|
| `BalanceSheet`           | Баланс                           | `balance_sheet`             | `balance_sheet`, `balance`                         |
| `IncomeStatement`        | Отчёт о прибылях и убытках       | `income_statement`          | `income_statement`, `income`                       |
| `StatementCashFlow`      | Отчёт о движении денежных средств| `statement_cash_flow`       | `statement_cash_flow`, `cash_flow`, `cashflow`     |
| `StatementEquityChanges` | Отчёт об изменениях капитала     | `statement_equity_changes`  | `statement_equity_changes`, `equity`, `capital`    |

(From [1].)

## Inference

Report type is inferred from the document id / URL filename via
`ReportType::try_from_filename`, matching one of the snake_case stems above [3].
If inference fails, `ProcessingFacade` falls back to `BalanceSheet` unless an
explicit type was supplied via `ProcessingOptions` [3]. The CLI requires the URL
filename to contain one of these stems [2].

## Relationships

* Part of the [domain models](/contracts/domain-models.md).
* Surfaced in the [CLI JSON contract](/contracts/cli-json-contract.md)
  `report_type` field (snake_case).
* Output is shaped by the [normalization rules](/domain/normalization-rules.md).

# Citations

[1] `pdf_pipeline/tbel-pdf/src/models/report_type.rs` — `ReportType`, `VARIANTS`, descriptors, `try_from_filename`.
[2] `pdf_pipeline/docs/cli-contract.md` — `pipeline` report-type inference from URL.
[3] `pdf_pipeline/tbel-pdf/src/processing.rs` — `ProcessingOptions.report_type` and fallback.
[4] `README.md` — supported reports context.
