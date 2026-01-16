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

# Show help if requested
if [[ "$1" == "-h" ]] || [[ "$1" == "--help" ]]; then
    echo "MCTS Training Analysis Tool"
    echo ""
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  --tag TAG              Filter experiments by tag (substring match)"
    echo "  --mode MODE            Filter by training mode (generalist, faction-specialist, etc.)"
    echo "  --min-gens N           Minimum number of generations required"
    echo "  --output DIR           Custom output directory"
    echo "  --no-dashboard         Skip HTML dashboard generation"
    echo "  --no-csv               Skip CSV export"
    echo "  --no-cache             Disable caching (re-parse all experiments)"
    echo "  --list-experiments     List available experiments and exit"
    echo "  --verbose, -v          Enable verbose logging"
    echo "  --help, -h             Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0                                        # Analyze all experiments with caching"
    echo "  $0 --tag generalist                       # Generalist experiments only"
    echo "  $0 --mode faction-specialist --min-gens 50  # Specialists with 50+ generations"
    echo "  $0 --no-cache                             # Force re-parse all experiments"
    echo "  $0 --list-experiments                     # List available experiments"
    exit 0
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
