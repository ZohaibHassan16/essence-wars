#!/usr/bin/env python3
"""Run benchmark evaluation on a checkpoint and output results as JSON.

This script is designed to be called by CI/CD pipelines.

Usage:
    python run_benchmark.py --checkpoint model.pt --output results.json
    python run_benchmark.py --checkpoint model.pt --full-eval --output results.json
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent.parent))


def main():
    parser = argparse.ArgumentParser(description="Run benchmark on a checkpoint")
    parser.add_argument(
        "--checkpoint",
        type=Path,
        required=True,
        help="Path to checkpoint file",
    )
    parser.add_argument(
        "--output",
        type=Path,
        default=Path("results.json"),
        help="Output JSON file",
    )
    parser.add_argument(
        "--full-eval",
        action="store_true",
        help="Run full evaluation (400 games) vs quick (40 games)",
    )
    parser.add_argument(
        "--games-per-opponent",
        type=int,
        default=None,
        help="Override games per opponent",
    )
    args = parser.parse_args()

    from essence_wars.benchmark.agents import NeuralAgent
    from essence_wars.benchmark.api import EssenceWarsBenchmark

    print(f"Loading checkpoint: {args.checkpoint}")
    agent = NeuralAgent.from_checkpoint(str(args.checkpoint), name="evaluated")

    # Determine games per opponent
    if args.games_per_opponent:
        games = args.games_per_opponent
    elif args.full_eval:
        games = 100
    else:
        games = 20

    print(f"Running evaluation ({games} games per opponent)...")

    # Determine baselines
    if args.full_eval:
        baselines = ["random", "greedy", "mcts50", "mcts100"]
    else:
        baselines = ["random", "greedy"]

    # Run benchmark
    benchmark = EssenceWarsBenchmark(games_per_opponent=games, verbose=True)
    bench_results = benchmark.evaluate(agent, baselines=baselines)

    # Build results dictionary
    results = {
        "checkpoint": str(args.checkpoint),
        "games_per_opponent": games,
        "games_played": bench_results.total_games,
        "win_rate_vs_random": bench_results.win_rate_vs_random,
        "win_rate_vs_greedy": bench_results.win_rate_vs_greedy,
    }

    if args.full_eval:
        results["win_rate_vs_mcts50"] = bench_results.win_rate_vs_mcts50
        results["win_rate_vs_mcts100"] = bench_results.win_rate_vs_mcts100

    # Use benchmark's Elo rating
    results["elo_rating"] = round(bench_results.elo_rating)

    # Save results
    with open(args.output, "w") as f:
        json.dump(results, f, indent=2)

    print(f"\nResults saved to: {args.output}")
    print(f"Estimated Elo: {results['elo_rating']}")

    return 0


if __name__ == "__main__":
    sys.exit(main())
