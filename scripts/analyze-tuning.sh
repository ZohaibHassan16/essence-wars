#!/bin/bash
# Analyze MCTS tuning experiments with proper Python path

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

export PYTHONPATH="$PROJECT_ROOT/python"

exec uv run --with pandas --with matplotlib --with seaborn --with numpy \
    python "$PROJECT_ROOT/python/scripts/analyze_tuning.py" "$@"
