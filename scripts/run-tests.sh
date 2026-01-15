#!/bin/bash
# Test Runner Script - Run tests by tier
#
# Usage:
#   ./scripts/run-tests.sh             # Run standard tests only
#   ./scripts/run-tests.sh quick       # + quick stress tests (~2-4 min)
#   ./scripts/run-tests.sh medium      # + medium stress tests (~15 min total)
#   ./scripts/run-tests.sh long        # + long stress tests (~45 min total)
#   ./scripts/run-tests.sh overnight   # + 100k game tests (~2 hours total)
#   ./scripts/run-tests.sh all         # Same as overnight

set -e

TIER=${1:-""}

echo "=== Essence Wars Test Runner ==="
echo ""

# Always run standard tests first
echo "Running standard tests..."
cargo nextest run --status-level=fail
echo ""

if [ -z "$TIER" ]; then
    echo "Standard tests complete!"
    echo ""
    echo "To run stress tests, specify a tier:"
    echo "  ./scripts/run-tests.sh quick      # ~2-4 min extra"
    echo "  ./scripts/run-tests.sh medium     # ~15 min total"
    echo "  ./scripts/run-tests.sh long       # ~45 min total"
    echo "  ./scripts/run-tests.sh overnight  # ~2 hours total"
    exit 0
fi

echo "Building release for stress tests..."
cargo build --release
echo ""

case $TIER in
    quick)
        echo "=== Running quick tier tests (~2-4 min) ==="
        # 20-50 game tests
        cargo nextest run --release --status-level=fail -- --ignored \
            test_mcts_vs_mcts_20 test_mcts_simulation_count test_mcts_vs_greedy \
            stress_test_mcts_fork
        ;;
    medium)
        echo "=== Running quick + medium tier tests (~15 min) ==="
        # Exclude 100k tests and coverage 10k
        cargo nextest run --release --status-level=fail \
            -E 'not (test(/100k/) | test(/coverage_10k/) | test(/500_games/) | test(/high_sims/) | test(/generate_golden/))' \
            -- --ignored
        ;;
    long)
        echo "=== Running quick + medium + long tier tests (~45 min) ==="
        # Exclude only 100k tests
        cargo nextest run --release --status-level=fail \
            -E 'not (test(/100k/) | test(/generate_golden/))' \
            -- --ignored
        ;;
    overnight|all)
        echo "=== Running ALL stress tests (~2 hours) ==="
        cargo nextest run --release --status-level=fail \
            -E 'not test(/generate_golden/)' \
            -- --ignored
        ;;
    *)
        echo "Unknown tier: $TIER"
        echo "Valid tiers: quick, medium, long, overnight, all"
        exit 1
        ;;
esac

echo ""
echo "=== All requested tests complete! ==="
