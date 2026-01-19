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

    from essence_wars.benchmark.agents import (
        GreedyAgent,
        MCTSAgent,
        NeuralAgent,
        RandomAgent,
    )
    from essence_wars.benchmark.api import run_matches

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

    results = {
        "checkpoint": str(args.checkpoint),
        "games_per_opponent": games,
        "games_played": 0,
    }

    # Evaluate vs each baseline
    baselines = [
        ("random", RandomAgent()),
        ("greedy", GreedyAgent()),
    ]

    if args.full_eval:
        baselines.extend([
            ("mcts50", MCTSAgent(simulations=50)),
            ("mcts100", MCTSAgent(simulations=100)),
        ])

    for name, opponent in baselines:
        print(f"  vs {name}...", end=" ", flush=True)
        match_results = run_matches(agent, opponent, num_games=games)
        win_rate = match_results["wins"] / games
        results[f"win_rate_vs_{name}"] = win_rate
        results["games_played"] += games
        print(f"{win_rate:.1%}")

    # Estimate Elo based on win rate vs Greedy
    import math

    win_rate = results["win_rate_vs_greedy"]
    if 0 < win_rate < 1:
        elo_diff = -400 * math.log10((1 / win_rate) - 1)
        results["elo_rating"] = round(1300 + elo_diff)
    elif win_rate >= 1:
        results["elo_rating"] = 1600
    else:
        results["elo_rating"] = 1000

    # Save results
    with open(args.output, "w") as f:
        json.dump(results, f, indent=2)

    print(f"\nResults saved to: {args.output}")
    print(f"Estimated Elo: {results['elo_rating']}")

    return 0


if __name__ == "__main__":
    sys.exit(main())
