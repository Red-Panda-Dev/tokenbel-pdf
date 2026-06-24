---
type: Build Configuration
title: Target and feature boundaries
description: Native/wasm target boundaries, the cli feature, and crate types.
resource: pdf_pipeline/tbel-pdf/Cargo.toml
source_paths:
  - pdf_pipeline/tbel-pdf/Cargo.toml
  - pdf_pipeline/tbel-pdf/src/lib.rs
  - ARCHITECTURE.md
  - AGENTS.md
confidence: observed
---

# Target and feature boundaries

The crate `tbel-pdf` is unified: it builds as `rlib` (library) and `cdylib`
(wasm), with a `cli` feature gating the native binary [1][3]. Changing crate
types affects wasm consumers [4].

## Native vs wasm

* **Native (default)** - full library + CLI. The binary has
  `required-features = ["cli"]` [1].
* **wasm32** - library-only. A `compile_error!` in `src/lib.rs` rejects
  `feature = "cli"` on `wasm32` [2]:
  ```rust
  #[cfg(all(feature = "cli", target_arch = "wasm32"))]
  compile_error!("The 'cli' feature is not supported on wasm32...");
  ```
* Module visibility is `cfg`-gated: `contract` and `commands` are
  `cfg(not(target_arch = "wasm32"))`; `wasm_bridge` is
  `cfg(target_arch = "wasm32")` [2].

## Toolchain

Rust toolchain is pinned to `1.94.0` in `pdf_pipeline/rust-toolchain.toml`; do
not change it casually [4].

## Target-specific dependencies

`Cargo.toml` declares target-specific dependencies so native and wasm builds
share the same crate under different platform constraints [1][3]. OCR and date
adapters use `#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]` to match
the single-threaded wasm runtime (see
[OCR provider boundary](/pipeline/ocr-provider-boundary.md)).

## Validation

From `pdf_pipeline/` [4]:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --features cli -- -D warnings
cargo test --workspace --features cli
bash ci-check.sh
```

`--features cli` is required for CLI builds and workspace integration tests [4].

## Relationships

* Enforces the [dependency invariants](/architecture/dependency-invariants.md).
* Gates the [CLI JSON contract](/contracts/cli-json-contract.md) to native.

# Citations

[1] `pdf_pipeline/tbel-pdf/Cargo.toml` — crate types, `cli` feature, target deps.
[2] `pdf_pipeline/tbel-pdf/src/lib.rs` — `compile_error!` and `cfg` module gates.
[3] `ARCHITECTURE.md` — §2 target boundary and §5 invariants.
[4] `AGENTS.md` — toolchain pin, crate-type warning, validation commands.
