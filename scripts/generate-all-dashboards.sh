#!/bin/bash
#
# Generate All Documentation Dashboards
#
# Usage:
#   ./scripts/generate-all-dashboards.sh
#
# This script regenerates all three dashboards:
# - Balance Dashboard (from validation results)
# - Training Dashboard (from MCTS experiments)
# - Performance Dashboard (from criterion benchmarks)

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

cd "$PROJECT_ROOT"

echo "==================================="
echo "Generating All Dashboards"
echo "==================================="
echo ""

# Balance Dashboard
echo "[1/3] Balance Dashboard..."
if [[ -f "experiments/validation/$(ls -t experiments/validation/ 2>/dev/null | head -1)/results.json" ]]; then
    PYTHONPATH="$PROJECT_ROOT/python" uv run python \
        "$PROJECT_ROOT/python/cardgame/analysis/research_dashboard.py" \
        --latest --output docs/dashboard/index.html
    # Also copy latest results to docs/data for GitHub Pages
    cp "experiments/validation/$(ls -t experiments/validation/ | head -1)/results.json" \
        docs/data/validation_results.json 2>/dev/null || true
elif [[ -f "docs/data/validation_results.json" ]]; then
    PYTHONPATH="$PROJECT_ROOT/python" uv run python \
        "$PROJECT_ROOT/python/cardgame/analysis/research_dashboard.py" \
        docs/data/validation_results.json --output docs/dashboard/index.html
else
    echo "   ⚠ No validation data found, skipping"
fi
echo ""

# Training Dashboard
echo "[2/3] Training Dashboard..."
if [[ -d "experiments/mcts" ]] && [[ "$(ls -A experiments/mcts 2>/dev/null)" ]]; then
    ./scripts/analyze-mcts.sh --output docs/dashboard/training_output > /dev/null 2>&1
    mv docs/dashboard/training_output/dashboard.html docs/dashboard/training.html 2>/dev/null || true
    rm -rf docs/dashboard/training_output
    echo "   ✓ Generated training.html"
else
    echo "   ⚠ No MCTS experiments found, skipping"
fi
echo ""

# Performance Dashboard
echo "[3/3] Performance Dashboard..."
if [[ -d "target/criterion" ]]; then
    PYTHONPATH="$PROJECT_ROOT/python" uv run python \
        "$PROJECT_ROOT/python/cardgame/analysis/performance_dashboard.py" \
        --output docs/dashboard/performance.html
else
    echo "   ⚠ No benchmark data found. Run 'cargo bench' first."
fi
echo ""

echo "==================================="
echo "Dashboard Generation Complete"
echo "==================================="
echo ""
echo "View at: file://$PROJECT_ROOT/docs/index.html"
echo ""
echo "For GitHub Pages, commit the docs/ folder."
