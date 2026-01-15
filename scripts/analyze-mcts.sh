#!/bin/bash
# Convenience wrapper for MCTS analysis tool
# Uses uv to manage dependencies automatically

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

cd "$PROJECT_ROOT"

# Check if uv is available
if ! command -v uv &> /dev/null; then
    echo "❌ Error: 'uv' is not installed"
    echo "Install it with: curl -LsSf https://astral.sh/uv/install.sh | sh"
    exit 1
fi

# Install analysis dependencies if needed (only if not already installed)
if ! python -c "import pandas" 2>/dev/null; then
    echo "🔧 Installing analysis dependencies..."
    uv pip install -e ".[analysis]"
fi

# Run the analysis tool with PYTHONPATH set
echo "🚀 Running MCTS analysis..."
echo ""

PYTHONPATH="$PROJECT_ROOT/python:$PYTHONPATH" uv run python python/scripts/mcts_analysis.py "$@"
