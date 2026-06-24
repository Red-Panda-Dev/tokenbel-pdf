---
type: Business Rule
title: Normalization rules
description: Stable output format for reporting period, codes, and integer values.
source_paths:
  - README.md
  - pdf_pipeline/README.md
  - pdf_pipeline/tbel-pdf/src/report_cleaning.rs
  - pdf_pipeline/tbel-pdf/src/date.rs
confidence: observed
---

# Normalization rules

The pipeline converts each report's accounting line-code table into a stable
normalized shape [1][2]:

* **Reporting period** - date columns normalized to `MM.YYYY` (e.g. `01.2025`,
  `12.2025`) [1].
* **Value cells** - integer values in thousand BYN [1][2].
* **Codes** - the numeric line-code column aligned and kept as 2-3 ASCII digits
  [3].
* **Dash values** - normalized to `0` [2].
* **Decimal truncation** - decimal comma values are truncated to integer
  thousands: `1 986,99` → `1986` [1].

## Where enforced

* `parse_belarusian_integer` in `report_cleaning.rs` handles value parsing [3].
* `align_code_column` / `is_code_column` / `code_digits` align the code column [3].
* Date headers go through [date normalization](/pipeline/date-normalization.md)
  (`RuleBasedDateNormalizer` or the rule-based `normalize_date_header` fallback)
  to reach `MM.YYYY` [3][4].
* `remove_blank_columns` strips OCR-introduced empty columns [3].

## Relationships

* Implemented by [table extraction / report cleaning](/pipeline/table-extraction.md).
* Date format produced by [date normalization](/pipeline/date-normalization.md).
* Final shape serialized by the [CLI JSON contract](/contracts/cli-json-contract.md)
  and XLSX export.

# Citations

[1] `README.md` — normalized output contract and decimal-truncation rule.
[2] `pdf_pipeline/README.md` — `MM.YYYY`, integer thousands BYN, dash→0.
[3] `pdf_pipeline/tbel-pdf/src/report_cleaning.rs` — `parse_belarusian_integer`, code alignment, blank-column removal.
[4] `pdf_pipeline/tbel-pdf/src/date.rs` — `DateNormalizer` and rule-based fallback.
