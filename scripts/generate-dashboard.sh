#!/bin/bash
#
# Generate Research Dashboard from Validation Results
#
# Usage:
#   ./scripts/generate-dashboard.sh                    # Use latest validation
#   ./scripts/generate-dashboard.sh path/to/results.json
#   ./scripts/generate-dashboard.sh --output custom.html

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

cd "$PROJECT_ROOT"

# Default output location
OUTPUT_DIR="$PROJECT_ROOT/docs/dashboard"
mkdir -p "$OUTPUT_DIR"

# Parse arguments
if [[ "$1" == "--help" || "$1" == "-h" ]]; then
    echo "Usage: $0 [results.json] [--output path.html]"
    echo ""
    echo "Generate an interactive research dashboard from validation results."
    echo ""
    echo "Options:"
    echo "  --latest         Use the most recent validation run (default)"
    echo "  --output, -o     Output HTML file path"
    echo ""
    echo "Examples:"
    echo "  $0                                    # Latest results -> docs/dashboard/index.html"
    echo "  $0 experiments/validation/*/results.json"
    echo "  $0 --output my_dashboard.html"
    exit 0
fi

# Build arguments
ARGS=()
OUTPUT_SET=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --output|-o)
            ARGS+=("--output" "$2")
            OUTPUT_SET=true
            shift 2
            ;;
        --latest)
            ARGS+=("--latest")
            shift
            ;;
        *)
            ARGS+=("$1")
            shift
            ;;
    esac
done

# Default to latest if no input specified
if [[ ${#ARGS[@]} -eq 0 ]] || [[ "${ARGS[0]}" == "--output" ]]; then
    ARGS=("--latest" "${ARGS[@]}")
fi

# Default output if not set
if [[ "$OUTPUT_SET" != "true" ]]; then
    ARGS+=("--output" "$OUTPUT_DIR/index.html")
fi

echo "Generating research dashboard..."
PYTHONPATH="$PROJECT_ROOT/python" uv run python "$PROJECT_ROOT/python/cardgame/analysis/research_dashboard.py" "${ARGS[@]}"

# Also copy to experiments directory if using latest
if [[ " ${ARGS[*]} " =~ " --latest " ]]; then
    LATEST_DIR=$(ls -td experiments/validation/*/ 2>/dev/null | head -1)
    if [[ -n "$LATEST_DIR" ]]; then
        cp "$OUTPUT_DIR/index.html" "${LATEST_DIR}dashboard.html" 2>/dev/null || true
        echo "Also copied to: ${LATEST_DIR}dashboard.html"
    fi
fi

echo ""
echo "Dashboard ready! Open in browser:"
echo "  file://$OUTPUT_DIR/index.html"
