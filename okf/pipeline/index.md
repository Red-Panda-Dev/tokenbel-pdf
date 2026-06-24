# Pipeline

The processing stages and external boundaries that turn OCR markdown into cleaned business tables.

* [ProcessingFacade orchestration](processing-facade.md) - the shared entrypoint for CLI and wasm.
* [OCR provider boundary](ocr-provider-boundary.md) - the trait that isolates external OCR access.
* [Table extraction and validation](table-extraction.md) - parsing and filtering financial tables from markdown.
* [Date normalization](date-normalization.md) - converting Russian/Belarusian date headers to `MM.YYYY`.
