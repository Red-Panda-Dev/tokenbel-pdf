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

echo "=== Wasm Tests Compile Check ==="
cargo check -p tbel-pdf --tests --target wasm32-unknown-unknown

if command -v wasm-bindgen >/dev/null 2>&1 && command -v node >/dev/null 2>&1; then
  echo "=== Wasm Worker Smoke Test ==="
  TMP_DIR="$(mktemp -d)"
  trap 'rm -rf "$TMP_DIR"' EXIT
  cargo build -p tbel-pdf --target wasm32-unknown-unknown
  wasm-bindgen \
    --target web \
    --out-dir "$TMP_DIR/pkg" \
    target/wasm32-unknown-unknown/debug/tbel_pdf.wasm
  node tbel-pdf/tests/worker_smoke.mjs "$TMP_DIR/pkg"
else
  echo "=== Wasm Worker Smoke Test Skipped (requires wasm-bindgen and node) ==="
fi

echo "=== All checks passed! ==="
