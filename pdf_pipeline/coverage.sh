#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

THRESHOLD="${COVERAGE_FAIL_UNDER_LINES:-70}"

if ! command -v cargo-llvm-cov >/dev/null 2>&1; then
  echo "cargo-llvm-cov is not installed. Install it with: cargo install cargo-llvm-cov" >&2
  exit 1
fi

echo "=== Library Coverage ==="
echo "Line coverage threshold: ${THRESHOLD}%"

cargo llvm-cov clean --workspace
cargo llvm-cov \
  -p tbel-pdf \
  --lib \
  --html \
  --output-dir target/coverage \
  --show-missing-lines \
  --fail-under-lines "$THRESHOLD"

echo "=== Coverage report written to target/coverage/index.html ==="
