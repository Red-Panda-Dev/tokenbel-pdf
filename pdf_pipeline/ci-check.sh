#!/usr/bin/env bash
set -euo pipefail

# CI verification matrix for `tbel-pdf`:
# 1) Native library check: cargo check -p tbel-pdf --lib
# 2) Native library tests: cargo test -p tbel-pdf --lib
# 3) Native CLI check: cargo check -p tbel-pdf --features cli --bin tbel-pdf
# 4) Wasm library check: cargo check -p tbel-pdf --lib --target wasm32-unknown-unknown
#
# Explicitly unsupported target/feature pair:
# - `--features cli` with `--target wasm32-unknown-unknown` is intentionally not run,
#   because the crate has a compile_error! guard for that combination.

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "=== Native Library Check ==="
cargo check -p tbel-pdf --lib

echo "=== Native Tests ==="
cargo test -p tbel-pdf --lib

echo "=== Native CLI Check ==="
cargo check -p tbel-pdf --features cli --bin tbel-pdf

echo "=== Wasm Library Check ==="
cargo check -p tbel-pdf --lib --target wasm32-unknown-unknown

echo "=== All checks passed! ==="
