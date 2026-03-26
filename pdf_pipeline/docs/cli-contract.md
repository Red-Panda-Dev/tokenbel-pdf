# CLI Contract

This document defines the stable contract for the `tbel-pdf` command-line interface.

## Subcommands

### `pipeline`

Main processing command for PDF financial reports.

```bash
tbel-pdf pipeline [OPTIONS]
```

**Required Arguments:**

- `--input-url <URL>` - URL of the input PDF passed directly to OCR provider

**Optional Arguments:**

- `--emit-contract <PATH>` - Write machine-readable contract JSON
- `--emit-stage-artifacts <DIR>` - Directory for intermediate artifacts

**Report Type Inference:**

Report type is inferred from URL filename. The filename must contain one of:

- `balance_sheet`
- `income_statement`
- `statement_cash_flow`
- `statement_equity_changes`

The CLI writes XLSX output automatically as `<url_file_stem>_output.xlsx` in the current directory.

## Exit Codes

| Code | Name          | Description                                             |
| ---- | ------------- | ------------------------------------------------------- |
| 0    | Success       | Command completed successfully                          |
| 1    | UsageError    | Invalid arguments or usage                              |
| 2    | PipelineError | Processing failure (no tables, invalid data, etc.)      |
| 3    | ProviderError | External service failure (OCR timeout, API error, etc.) |

## Contract Schema

### Success Contract

```json
{
    "status": "success",
    "report_type": "balance_sheet",
    "output_xlsx": "/tmp/balance.xlsx",
    "rows_processed": 42
}
```

### Failure Contract

```json
{
    "status": "failure",
    "error_code": "NoFinancialTablesFound",
    "error_message": "No financial tables detected in document",
    "document_id": "doc_123"
}
```

## Error Codes

| Code                        | Description                                              |
| --------------------------- | -------------------------------------------------------- |
| `NoFinancialTablesFound`    | Document contains no detectable financial tables         |
| `UnsupportedLayout`         | Document layout not supported (wrong column count, etc.) |
| `InvalidHeader`             | Expected header not found                                |
| `DimensionValidationFailed` | Output dimensions below minimum (3 columns, 10 rows)     |
| `ProviderError`             | External service (OCR) failure                           |
| `ParseError`                | Failed to parse date or numeric value                    |

## Usage Example

```bash
# Process a balance sheet PDF
tbel-pdf pipeline \
  --input-url https://example.com/reports/2025.09_balance_sheet_company.pdf \
  --emit-contract /tmp/contract.json

# Check exit code
if [ $? -eq 0 ]; then
  echo "Success: $(cat /tmp/contract.json)"
else
  echo "Failed: $(cat /tmp/contract.json)"
fi
```

## Integration Notes

- Logs are written to stderr
- Contract JSON is written to stdout and duplicated to `--emit-contract` path when provided
- Do NOT parse log lines for success/failure - use the contract
