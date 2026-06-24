# Knowledge Bundle Update Log

## 2026-06-25

* **Initialization**: Created OKF knowledge bundle for the repository root (`tokenbel-pdf`).
* **Creation**: Added [System architecture](architecture/system-architecture.md), [Dependency direction and invariants](architecture/dependency-invariants.md).
* **Creation**: Added [ProcessingFacade orchestration](pipeline/processing-facade.md), [OCR provider boundary](pipeline/ocr-provider-boundary.md), [Table extraction and validation](pipeline/table-extraction.md), [Date normalization](pipeline/date-normalization.md).
* **Creation**: Added [CLI JSON contract](contracts/cli-json-contract.md), [Domain models](contracts/domain-models.md).
* **Creation**: Added [Report types](domain/report-types.md), [Normalization rules](domain/normalization-rules.md).
* **Creation**: Added [Target and feature boundaries](build/target-feature-boundaries.md).
* **Note**: Concepts were grounded in observed source under `pdf_pipeline/tbel-pdf/src/` plus `ARCHITECTURE.md`, `README.md`, and `pdf_pipeline/docs/cli-contract.md`. Where documentation and code could diverge, executable code was treated as source of truth per `ARCHITECTURE.md` §6.
