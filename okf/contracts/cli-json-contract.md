---
type: Contract
title: CLI JSON contract
description: Stable exit codes and JSON success/failure shapes emitted by the native CLI.
resource: pdf_pipeline/tbel-pdf/src/contract/mod.rs
source_paths:
  - pdf_pipeline/tbel-pdf/src/contract/mod.rs
  - pdf_pipeline/docs/cli-contract.md
  - pdf_pipeline/tbel-pdf/src/lib.rs
confidence: observed
---

# CLI JSON contract

The native CLI emits a machine-readable JSON contract on both success and
failure, with stable exit codes. This contract is native-only (`contract` is
`cfg(not(target_arch = "wasm32"))` in `src/lib.rs`) and documented in
`pdf_pipeline/docs/cli-contract.md` [1][2][3]. If `src/contract/mod.rs` changes,
the doc and contract tests must change together [2].

## Exit codes

| Code | Name          | Meaning                                        |
| ---- | ------------- | ---------------------------------------------- |
| 0    | Success       | Command completed successfully                 |
| 1    | UsageError    | Invalid arguments or usage                     |
| 2    | PipelineError | Processing failure (no tables, invalid data)   |
| 3    | ProviderError | External service failure (OCR timeout, API)    |

(From [1][2].)

## Success contract

```json
{
  "output_json": "/tmp/output.json",
  "output_xlsx": "/tmp/output.xlsx",
  "document_id": "doc-123",
  "report_type": "balance_sheet",
  "row_count": 42,
  "column_count": 4
}
```

Fields: `output_json` (PathBuf), optional `output_xlsx`, `document_id`,
`report_type` (snake_case), `row_count`, `column_count` [1][2].

## Failure contract

```json
{
  "error_code": "no_financial_tables_found",
  "error_message": "No tables detected",
  "document_id": "doc-456"
}
```

`error_code` is serialized `snake_case`; `document_id` is optional [1].
Variants: `no_financial_tables_found`, `unsupported_layout`, `invalid_header`,
`dimension_validation_failed`, `provider_error`, `parse_error` [1].

## `pipeline` subcommand

`--input-url <URL>` is required and passed directly to the OCR provider.
`--emit-contract <PATH>` writes the contract JSON; `--emit-stage-artifacts`
writes intermediate artifacts. Report type is inferred from the URL filename,
which must contain one of the [report type](/domain/report-types.md) stems. The
CLI writes `<url_file_stem>_output.xlsx` in the current directory [2].

## Relationships

* Built on [domain models](/contracts/domain-models.md) and the output of
  [ProcessingFacade](/pipeline/processing-facade.md).
* Constrained by [dependency invariants](/architecture/dependency-invariants.md):
  the contract is a versioned agreement.

# Citations

[1] `pdf_pipeline/tbel-pdf/src/contract/mod.rs` — `ExitCode`, `ErrorCode`, `SuccessContract`, `FailureContract`.
[2] `pdf_pipeline/docs/cli-contract.md` — subcommand, exit codes, schema, usage.
[3] `pdf_pipeline/tbel-pdf/src/lib.rs` — `cfg(not(target_arch = "wasm32"))` gate on `contract`.
