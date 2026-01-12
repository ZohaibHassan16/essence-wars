#!/bin/bash
# Run comprehensive benchmark suite for Essence Wars engine
# Usage: ./scripts/run-benchmarks.sh [--open-report]

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
cd "$PROJECT_ROOT"

echo "════════════════════════════════════════════════════════════════"
echo "  Essence Wars - Performance Benchmark Suite"
echo "════════════════════════════════════════════════════════════════"
echo ""

# Check for Rust toolchain
if ! command -v cargo &> /dev/null; then
    echo "❌ Error: cargo not found. Please install Rust."
    exit 1
fi

# Create results directory
RESULTS_DIR="benchmark_results_$(date +%Y%m%d_%H%M%S)"
mkdir -p "$RESULTS_DIR"
echo "📁 Results will be saved to: $RESULTS_DIR"
echo ""

# 1. Run Criterion benchmarks
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🔬 Running Criterion Benchmarks..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
cargo bench --bench game_benchmarks 2>&1 | tee "$RESULTS_DIR/criterion_output.txt"
echo ""

# 2. Run MCTS profiling
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "⚡ Running MCTS Profiling..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
cargo run --release --bin profile_mcts 2>&1 | tee "$RESULTS_DIR/profile_mcts.txt"
echo ""

# 3. Run arena throughput test
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🎮 Running Arena Throughput Test (100K games)..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
cargo run --release --bin arena -- \
    --bot1 random --bot2 random \
    --games 100000 --progress 2>&1 | tee "$RESULTS_DIR/arena_throughput.txt"
echo ""

# 4. Generate summary
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📊 Generating Summary Report..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

SUMMARY_FILE="$RESULTS_DIR/SUMMARY.txt"

cat > "$SUMMARY_FILE" << 'EOF'
════════════════════════════════════════════════════════════════
  ESSENCE WARS - BENCHMARK SUMMARY
════════════════════════════════════════════════════════════════

Generated: $(date)
Platform: $(uname -s) $(uname -m)
Rust: $(rustc --version)

────────────────────────────────────────────────────────────────
  CORE ENGINE PERFORMANCE
────────────────────────────────────────────────────────────────

EOF

# Extract key metrics from Criterion output
echo "Engine Operations:" >> "$SUMMARY_FILE"
grep "engine_fork" "$RESULTS_DIR/criterion_output.txt" | grep "time:" | head -1 >> "$SUMMARY_FILE" 2>/dev/null || echo "  (data not available)" >> "$SUMMARY_FILE"
grep "state_tensor" "$RESULTS_DIR/criterion_output.txt" | grep "time:" | head -1 >> "$SUMMARY_FILE" 2>/dev/null || echo "  (data not available)" >> "$SUMMARY_FILE"
grep "legal_actions" "$RESULTS_DIR/criterion_output.txt" | grep "time:" | head -1 >> "$SUMMARY_FILE" 2>/dev/null || echo "  (data not available)" >> "$SUMMARY_FILE"
echo "" >> "$SUMMARY_FILE"

echo "Full Game Simulation:" >> "$SUMMARY_FILE"
grep "random_game" "$RESULTS_DIR/criterion_output.txt" | grep "time:" | head -1 >> "$SUMMARY_FILE" 2>/dev/null || echo "  (data not available)" >> "$SUMMARY_FILE"
grep "greedy_game" "$RESULTS_DIR/criterion_output.txt" | grep "time:" | head -1 >> "$SUMMARY_FILE" 2>/dev/null || echo "  (data not available)" >> "$SUMMARY_FILE"
grep "games_per_second" "$RESULTS_DIR/criterion_output.txt" | grep "time:" | head -1 >> "$SUMMARY_FILE" 2>/dev/null || echo "  (data not available)" >> "$SUMMARY_FILE"
echo "" >> "$SUMMARY_FILE"

cat >> "$SUMMARY_FILE" << 'EOF'
────────────────────────────────────────────────────────────────
  MCTS PERFORMANCE
────────────────────────────────────────────────────────────────

EOF

# Extract MCTS metrics
cat "$RESULTS_DIR/profile_mcts.txt" >> "$SUMMARY_FILE"

cat >> "$SUMMARY_FILE" << 'EOF'

────────────────────────────────────────────────────────────────
  DETAILED REPORTS
────────────────────────────────────────────────────────────────

• Criterion HTML Report: target/criterion/report/index.html
• Full Criterion Output: criterion_output.txt
• MCTS Profiling: profile_mcts.txt
• Arena Throughput: arena_throughput.txt

════════════════════════════════════════════════════════════════
EOF

# Display summary
cat "$SUMMARY_FILE"

# Copy Criterion report
if [ -d "target/criterion/report" ]; then
    cp -r target/criterion/report "$RESULTS_DIR/"
    echo ""
    echo "📈 Criterion HTML report copied to: $RESULTS_DIR/report/index.html"
fi

echo ""
echo "✅ Benchmark suite complete!"
echo "📁 All results saved to: $RESULTS_DIR"
echo ""

# Open report if requested
if [[ "$1" == "--open-report" ]]; then
    if command -v xdg-open &> /dev/null; then
        xdg-open "$RESULTS_DIR/report/index.html" 2>/dev/null || true
    elif command -v open &> /dev/null; then
        open "$RESULTS_DIR/report/index.html" 2>/dev/null || true
    fi
fi

echo "════════════════════════════════════════════════════════════════"
