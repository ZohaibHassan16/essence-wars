#!/bin/bash
# Quick performance stats extractor
# Usage: ./scripts/perf-stats.sh

set -e

cd "$(dirname "$0")/.."

echo "════════════════════════════════════════════════════════════════"
echo "  ESSENCE WARS - QUICK PERFORMANCE STATS"
echo "════════════════════════════════════════════════════════════════"
echo ""

# Check if we have recent benchmark results
if [ ! -f "benchmark_results.txt" ]; then
    echo "⚠️  No benchmark results found. Run: cargo bench --bench game_benchmarks"
    exit 1
fi

if [ ! -f "profiling_results.txt" ]; then
    echo "⚠️  No profiling results found. Run: cargo run --release --bin profile_mcts"
    exit 1
fi

echo "📊 CORE ENGINE METRICS"
echo "────────────────────────────────────────────────────────────────"

# Parse engine fork time
FORK_TIME=$(grep "engine_fork" benchmark_results.txt | grep "time:" | head -1 | sed -E 's/.*\[([0-9.]+) ([a-z]+).*/\1 \2/')
echo "  • State Cloning (fork):      $FORK_TIME"

# Parse state tensor time
TENSOR_TIME=$(grep "state_tensor" benchmark_results.txt | grep "time:" | head -1 | sed -E 's/.*\[([0-9.]+) ([a-z]+).*/\1 \2/')
echo "  • State Tensor Generation:   $TENSOR_TIME"

# Parse legal actions time
LEGAL_TIME=$(grep "legal_actions" benchmark_results.txt | grep "time:" | head -1 | sed -E 's/.*\[([0-9.]+) ([a-z]+).*/\1 \2/')
echo "  • Legal Action Computation:  $LEGAL_TIME"

echo ""
echo "🎮 FULL GAME SIMULATION"
echo "────────────────────────────────────────────────────────────────"

# Parse random game time
RANDOM_TIME=$(grep "random_game" benchmark_results.txt | grep "time:" | head -1 | sed -E 's/.*\[([0-9.]+) ([a-z]+).*/\1 \2/')
echo "  • Random vs Random:          $RANDOM_TIME"

# Parse greedy game time
GREEDY_TIME=$(grep "greedy_game" benchmark_results.txt | grep "time:" | head -1 | sed -E 's/.*\[([0-9.]+) ([a-z]+).*/\1 \2/')
echo "  • Greedy vs Greedy:          $GREEDY_TIME"

# Parse throughput
THROUGHPUT=$(grep "games_per_second" benchmark_results.txt | grep "thrpt:" | head -1 | sed -E 's/.*\[([0-9.]+) ([KM])?elem.*/\1 \2/')
echo "  • Throughput (10-game batch): $THROUGHPUT games/sec"

echo ""
echo "🧠 MCTS BOT PERFORMANCE"
echo "────────────────────────────────────────────────────────────────"

# Extract from profiling results
grep "engine.fork():" profiling_results.txt | sed 's/engine.fork(): /  • Fork: /'
grep "GreedyBot.select_action():" profiling_results.txt | sed 's/GreedyBot.select_action(): /  • Greedy action: /'
grep "Full rollout:" profiling_results.txt | sed 's/Full rollout: /  • Full rollout: /'

echo ""
grep "MCTS-100:" profiling_results.txt | sed 's/MCTS-100:/  • MCTS-100:/'
grep "MCTS-500:" profiling_results.txt | sed 's/MCTS-500:/  • MCTS-500:/'

echo ""
echo "════════════════════════════════════════════════════════════════"
echo ""
echo "💡 KEY INSIGHTS:"
echo "  • State cloning is ~100ns (10M clones/sec)"
echo "  • Full rollout is ~85µs (12K rollouts/sec)"
echo "  • Random games: ~15µs (67K games/sec)"
echo "  • MCTS-100: ~3ms/decision (real-time capable)"
echo "  • MCTS-500: ~15ms/decision (tournament strength)"
echo ""
echo "📈 For detailed analysis, see: docs/performance-benchmarks.md"
echo "📊 For HTML report, open: target/criterion/report/index.html"
echo ""
