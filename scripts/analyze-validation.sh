#!/bin/bash
#
# Analyze Validation Results (Text Report)
#
# Usage:
#   ./scripts/analyze-validation.sh                    # Use latest validation
#   ./scripts/analyze-validation.sh path/to/results.json
#   ./scripts/analyze-validation.sh --latest

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

cd "$PROJECT_ROOT"

# Default to latest if no args
ARGS=("$@")
if [[ ${#ARGS[@]} -eq 0 ]]; then
    ARGS=("--latest")
fi

PYTHONPATH="$PROJECT_ROOT/python" uv run python "$PROJECT_ROOT/python/cardgame/analysis/validation_cli.py" "${ARGS[@]}"
