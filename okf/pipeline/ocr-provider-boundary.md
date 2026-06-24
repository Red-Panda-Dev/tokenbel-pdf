---
type: Interface
title: OCR provider boundary
description: Trait that isolates external OCR access so tests run offline.
resource: pdf_pipeline/tbel-pdf/src/ocr.rs
source_paths:
  - pdf_pipeline/tbel-pdf/src/ocr.rs
  - ARCHITECTURE.md
confidence: observed
---

# OCR provider boundary

`OcrProvider` (`src/ocr.rs`) is the trait boundary that isolates all external
OCR access [1][2]. `ProcessingFacade::process` depends on `&dyn OcrProvider`,
which lets production and tests swap implementations without touching
transformation logic [2].

## Trait

```rust
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
pub trait OcrProvider: Send + Sync {
    async fn acquire_ocr(&self, input: PdfInput) -> Result<OcrOutput, ProviderError>;
}
```

(From [1].) The `?Send` variant on wasm32 reflects the single-threaded wasm
runtime.

## Implementations

* `MistralOcrProvider` - production provider. `new()` reads `MISTRAL_API_KEY`
  (required), `MISTRAL_OCR_MODEL` (defaults to `mistral-ocr-latest`), and
  `TBEL_OCR_DOCUMENT_URL`. `with_config(OcrProviderConfig)` is the portable,
  env-independent constructor used by wasm/tests [1].
* `MockOcrProvider` - deterministic test double keyed by document id; returns
  configured `OcrOutput` responses [1].
* `StubOcrProvider` - minimal double returning empty markdown [1].

## Errors

`ProviderError` variants: `Network`, `Api`, `InvalidInput`, `Parse` [1]. These
surface to the CLI as exit code 3 (`ProviderError`) — see
[CLI JSON contract](/contracts/cli-json-contract.md).

## Relationships

* Consumed by [ProcessingFacade orchestration](/pipeline/processing-facade.md).
* Constrained by the [dependency invariants](/architecture/dependency-invariants.md):
  external OCR must stay behind this trait.

# Citations

[1] `pdf_pipeline/tbel-pdf/src/ocr.rs` — `OcrProvider`, `MistralOcrProvider`, `MockOcrProvider`, `StubOcrProvider`, `ProviderError`, `OcrProviderConfig`.
[2] `ARCHITECTURE.md` — §5 invariant on external OCR access.
