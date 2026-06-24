---
type: Architectural Invariant
title: Dependency direction and invariants
description: Enforced boundaries between models, adapters, CLI, and wasm targets.
source_paths:
  - ARCHITECTURE.md
  - pdf_pipeline/tbel-pdf/src/lib.rs
  - pdf_pipeline/tbel-pdf/src/models/mod.rs
  - pdf_pipeline/tbel-pdf/src/ocr.rs
  - AGENTS.md
confidence: observed
---

# Dependency direction and invariants

The observed dependency direction is entrypoints → shared pipeline →
adapters/models [1]. The boundary is enforced mostly by Cargo features and
`cfg` gates rather than by separate crates.

## Invariants

* **CLI is native-only.** `required-features = ["cli"]` gates the binary; a
  `compile_error!` in `src/lib.rs` rejects `feature = "cli"` on `wasm32` [1][2].
  See [target and feature boundaries](/build/target-feature-boundaries.md).
* **Models stay domain-data-only.** `src/models/` must not acquire HTTP,
  filesystem, Tokio, scraper, PDF-reader, CLI, or wasm-bridge responsibilities.
  Observed: model files import only `serde`/`std`/formatting utilities [1][3].
* **External OCR stays behind `OcrProvider`.** `ProcessingFacade::process`
  depends on `&dyn OcrProvider`; production uses `MistralOcrProvider` and tests
  use `MockOcrProvider`/`StubOcrProvider` [1][4]. See
  [OCR provider boundary](/pipeline/ocr-provider-boundary.md).
* **Shared extraction routes through `ProcessingFacade`.** Both CLI and wasm
  entrypoints funnel through it so native and wasm behavior stay aligned [1].
  See [ProcessingFacade](/pipeline/processing-facade.md).
* **CLI JSON output is a contract.** Exit codes and JSON shape must evolve with
  `pdf_pipeline/docs/cli-contract.md` and contract tests together [1]. See
  [CLI JSON contract](/contracts/cli-json-contract.md).
* **Normal tests stay offline.** Real OCR/network dependence is kept out of
  regression tests via committed fixtures and mock/stub providers [1][5].

## Enforcement

Most rules are structural conventions rather than separate-crate boundaries.
The hard compile-time enforcements are the `cli` feature gate, the wasm32
`compile_error!`, and the `cfg`-gated module visibility in `src/lib.rs` [2].

# Citations

[1] `ARCHITECTURE.md` — §5 Architectural invariants and constraints.
[2] `pdf_pipeline/tbel-pdf/src/lib.rs` — `compile_error!` and `cfg` gates.
[3] `pdf_pipeline/tbel-pdf/src/models/mod.rs` — domain-data-only module surface.
[4] `pdf_pipeline/tbel-pdf/src/ocr.rs` — `OcrProvider` trait and implementations.
[5] `AGENTS.md` — Repository-specific gotchas and offline-test rule.
