---
type: Adapter
title: Date normalization
description: Adapter that converts Russian/Belarusian date headers to MM.YYYY.
resource: pdf_pipeline/tbel-pdf/src/date.rs
source_paths:
  - pdf_pipeline/tbel-pdf/src/date.rs
  - pdf_pipeline/tbel-pdf/src/report_cleaning.rs
  - README.md
confidence: observed
---

# Date normalization

Date normalization converts Russian/Belarusian financial date headers (e.g.
month names, period phrases) into the stable `MM.YYYY` format required by the
normalized output contract [1][3].

## Trait

```rust
pub trait DateNormalizer: Send + Sync {
    async fn normalize_header(&self, header: &str) -> Result<String, DateError>;
}
```

(From [1].) Parsing failures from the model return the original header rather
than an `Err`; `DateError` is reserved for execution-level failures [1].

## Implementations

* `RuleBasedDateNormalizer` - the canonical implementation despite the
  historical name. Uses a Mistral chat completion with the
  `prompts/financial_date_extraction.txt` template (up to `MAX_RETRIES = 3`),
  with an in-memory cache. Falls back to rule-based logic when no API key is
  configured [1]. Env: `MISTRAL_API_KEY`, `MISTRAL_DATE_MODEL` (defaults to
  `mistral-large-latest`) [1].
* `StubDateNormalizer` - offline test double [1].

## Rule-based fallback

`normalize_date_header(header, fallback_index)` in `report_cleaning.rs` maps
Russian month-name stems (`январ` → `01`, `феврал` → `02`, …) and otherwise
synthesizes a header from the fallback index [2]. This is the offline path used
when the adapter has no API key, keeping tests deterministic.

## Relationships

* Used by [table extraction / report cleaning](/pipeline/table-extraction.md).
* Implements the [normalization rules](/domain/normalization-rules.md) for the
  `MM.YYYY` column format.
* Errors are scoped to `DateError` and never abort the cleaning stage for a
  parsing problem [1].

# Citations

[1] `pdf_pipeline/tbel-pdf/src/date.rs` — `DateNormalizer`, `RuleBasedDateNormalizer`, `StubDateNormalizer`, `DateError`, config.
[2] `pdf_pipeline/tbel-pdf/src/report_cleaning.rs` — `normalize_date_header` rule-based fallback.
[3] `README.md` — `MM.YYYY` normalized reporting period contract.
