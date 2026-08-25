# AGENTS.md

## Scope and inheritance

Applies to: `pdf_pipeline/` (Cargo workspace root).

Inherits repository-wide guidance from `../AGENTS.md`. This file defines only local differences for this subtree.

## What lives here

```text
pdf_pipeline/
├── Cargo.toml                   # Workspace manifest; single member: tbel-pdf
├── Cargo.lock                   # Locked Rust dependencies
├── rust-toolchain.toml          # Pinned toolchain: 1.94.0 with rustfmt/clippy
├── ci-check.sh                  # Native + wasm verification matrix
├── coverage.sh                  # cargo-llvm-cov library coverage gate
├── docs/cli-contract.md         # Stable machine-readable CLI contract
├── tbel-pdf/                    # Unified crate; see tbel-pdf/AGENTS.md
└── tests/                       # Shared PDFs, OCR fixtures, golden baselines
```

## Local boundaries and invariants

- The workspace has one member crate. Keep workspace-level changes in `Cargo.toml` minimal and verify they still apply to `tbel-pdf`.
- `rust-toolchain.toml` pins Rust `1.94.0`; do not update it without checking `rust-version` in `tbel-pdf/Cargo.toml`.
- `ci-check.sh` intentionally does **not** run `--features cli` for wasm32 because the crate rejects that combination.
- `coverage.sh` measures library code only with `cargo llvm-cov -p tbel-pdf --lib`; it intentionally excludes CLI plumbing.
- `docs/cli-contract.md` must stay aligned with `tbel-pdf/src/contract/mod.rs`.

## Safe change rules

- Run Cargo commands from this directory so the pinned toolchain and workspace paths are used.
- Keep offline validation independent of `MISTRAL_API_KEY`; live OCR commands belong in manual smoke testing, not required tests.
- If a pipeline output change is expected, update tests and golden fixtures together; see `tests/AGENTS.md` before touching baselines.
- Do not add unrelated packages to this workspace unless the single-crate architecture in `../ARCHITECTURE.md` is deliberately changed.

## Validation

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --features cli -- -D warnings
cargo test --workspace --features cli
cargo run -p tbel-pdf --features cli -- --help
bash ci-check.sh
```

Optional coverage, when `cargo-llvm-cov` is installed:

```bash
bash coverage.sh
```

## Nearby docs

| Doc | Path |
| --- | --- |
| Global architecture | `../ARCHITECTURE.md` |
| Workspace operational reference | `README.md` |
| CLI contract | `docs/cli-contract.md` |
| Crate-local guide | `tbel-pdf/AGENTS.md` |
| Fixture/golden guide | `tests/AGENTS.md` |
