#!/usr/bin/env bash
# Validates all decks to ensure they only reference existing cards in the database.
# This prevents runtime errors from missing card references.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

cd "$PROJECT_ROOT"

# Build if needed
if [[ ! -f target/release/validate_decks ]] || [[ Cargo.toml -nt target/release/validate_decks ]]; then
    echo "Building validate_decks..."
    cargo build --release --bin validate_decks
    echo
fi

# Run validation
exec ./target/release/validate_decks "$@"
