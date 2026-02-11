"""Evaluation and benchmark commands for Essence Wars."""

from __future__ import annotations

import sys
from pathlib import Path
from typing import TYPE_CHECKING, Any

if TYPE_CHECKING:
    import click


def register_evaluate_commands(cli: click.Group) -> None:
    """Register evaluation commands."""
    import click

    from .utils import run_script

    @cli.command("benchmark")
    @click.option(
        "--checkpoint",
        "-c",
        type=click.Path(exists=True),
        required=True,
        help="Path to agent checkpoint",
    )
    @click.option(
        "--output",
        "-o",
        type=click.Path(),
        default="benchmark_results.json",
        help="Output JSON file",
    )
    @click.option(
        "--full-eval",
        is_flag=True,
        help="Full evaluation (100 games per opponent vs 20)",
    )
    @click.option(
        "--games-per-opponent",
        type=int,
        default=None,
        help="Override games per opponent",
    )
    @click.option(
        "--baselines",
        multiple=True,
        default=None,
        help="Baselines to evaluate against (random, greedy, mcts50, mcts100)",
    )
    def benchmark(**kwargs: Any) -> None:
        """Benchmark an agent against baseline opponents.

        Evaluates the agent against RandomBot, GreedyBot, and optionally
        MCTS bots. Reports win rates and estimated Elo rating.

        \b
        Examples:
          essence-wars benchmark --checkpoint model.pt
          essence-wars benchmark --checkpoint model.pt --full-eval
          essence-wars benchmark --checkpoint model.pt --games-per-opponent 50
        """
        _run_benchmark(kwargs)

    @cli.command("evaluate-mcts")
    @click.option(
        "--checkpoint",
        "-c",
        type=click.Path(exists=True),
        required=True,
        help="Path to neural network checkpoint",
    )
    @click.option("--sims", default=100, help="MCTS simulations per move")
    @click.option("--games", "-n", default=100, help="Number of evaluation games")
    @click.option(
        "--opponent",
        default="greedy",
        type=click.Choice(["random", "greedy", "mcts"]),
        help="Opponent type",
    )
    @click.option(
        "--output", "-o", type=click.Path(), default=None, help="Output JSON file"
    )
    def evaluate_mcts(**kwargs: Any) -> None:
        """Evaluate a neural MCTS agent.

        Uses the neural network as policy prior and value estimate within MCTS.

        \b
        Examples:
          essence-wars evaluate-mcts --checkpoint model.pt --sims 200
          essence-wars evaluate-mcts --checkpoint model.pt --opponent mcts --games 50
        """
        run_script("evaluate_neural_mcts", kwargs)


def _run_benchmark(kwargs: dict[str, Any]) -> None:
    """Run benchmark evaluation."""
    try:
        from essence_wars.benchmark.agents import NeuralAgent
        from essence_wars.benchmark.api import EssenceWarsBenchmark
    except ImportError as e:
        print(f"Error: Missing dependencies for benchmarking: {e}")
        print("Install with: pip install essence-wars[train]")
        sys.exit(1)

    import json

    checkpoint = kwargs["checkpoint"]
    output = kwargs["output"]
    full_eval = kwargs["full_eval"]
    games = kwargs.get("games_per_opponent")
    baselines = kwargs.get("baselines")

    print(f"Loading checkpoint: {checkpoint}")
    agent = NeuralAgent.from_checkpoint(checkpoint, name="evaluated")

    # Determine games per opponent
    if games is None:
        games = 100 if full_eval else 20

    # Determine baselines
    if not baselines:
        baselines = (
            ["random", "greedy", "mcts50", "mcts100"]
            if full_eval
            else ["random", "greedy"]
        )
    else:
        baselines = list(baselines)

    print(f"Running evaluation ({games} games per opponent)...")
    bench = EssenceWarsBenchmark(games_per_opponent=games, verbose=True)
    results = bench.evaluate(agent, baselines=baselines)

    # Build and save results
    output_data = {
        "checkpoint": checkpoint,
        "games_per_opponent": games,
        "games_played": results.total_games,
        "win_rate_vs_random": results.win_rate_vs_random,
        "win_rate_vs_greedy": results.win_rate_vs_greedy,
        "elo_rating": round(results.elo_rating),
    }

    if "mcts50" in baselines:
        output_data["win_rate_vs_mcts50"] = results.win_rate_vs_mcts50
    if "mcts100" in baselines:
        output_data["win_rate_vs_mcts100"] = results.win_rate_vs_mcts100

    with Path(output).open("w") as f:
        json.dump(output_data, f, indent=2)

    print(f"\nResults saved to: {output}")
    print(f"Estimated Elo: {output_data['elo_rating']}")
