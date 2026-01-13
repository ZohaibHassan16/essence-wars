#!/usr/bin/env bash
# Run clippy on library and binaries (excluding tests)
#
# Usage: ./scripts/run-clippy.sh

set -e

echo "Running clippy on library and binaries (tests excluded)..."
cargo clippy --lib --bins --all-features -- -D warnings

echo ""
echo "✓ Clippy checks passed!"
