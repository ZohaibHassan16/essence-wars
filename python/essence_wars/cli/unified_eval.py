"""Unified evaluation command for Essence Wars."""

from __future__ import annotations

import json
import sys
from pathlib import Path
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    import click


def register_unified_eval_command(cli: click.Group) -> None:
    """Register the unified eval command."""
    import click

    @cli.command("eval")
    @click.option(
        "--agents",
        "-a",
        multiple=True,
        required=False,
        help="Agent specs (e.g., ppo-generalist greedy mcts-100)",
    )
    @click.option(
        "--preset",
        type=click.Choice(["validate", "benchmark", "neural"]),
        help="Use a preset configuration",
    )
    @click.option(
        "--games", "-n", type=int, default=None, help="Games per matchup (default: 50)"
    )
    @click.option(
        "--output", "-o", type=click.Path(), default=None, help="Save results to JSON file"
    )
    @click.option("--no-elo", is_flag=True, help="Don't update agent_elo.json")
    @click.option("--quiet", "-q", is_flag=True, help="Minimal output")
    def unified_eval(
        agents: tuple[str, ...],
        preset: str | None,
        games: int | None,
        output: str | None,
        no_elo: bool,
        quiet: bool,
    ) -> None:
        """Unified evaluation for neural and algorithmic agents.

        Evaluates agents against each other, computing win rates and ELO ratings.
        Supports both neural agents (from registry) and algorithmic bots.

        \b
        Agent specs:
          random          - Random bot
          greedy          - Greedy bot
          mcts-100        - MCTS with 100 simulations
          alphabeta-6     - Alpha-Beta depth 6
          ppo-generalist  - Neural agent from registry
          path:model.pt   - Neural agent from explicit path

        \b
        Examples:
          essence-wars eval --agents greedy mcts-100
          essence-wars eval --agents ppo-generalist greedy --games 50
          essence-wars eval --preset benchmark
          essence-wars eval --preset neural -o results.json
        """
        from essence_wars.eval import UnifiedEvaluator
        from essence_wars.eval.unified import PRESETS

        # Determine agents from preset or explicit list
        agent_list: list[str]
        games_per: int
        if preset:
            preset_config = PRESETS[preset]
            # Cast from preset dict values
            preset_agents = preset_config["agents"]
            preset_games = preset_config["games_per_matchup"]
            assert isinstance(preset_agents, list)
            assert isinstance(preset_games, int)
            agent_list = list(preset_agents)
            games_per = games if games is not None else preset_games
            if not quiet:
                print(f"Using preset '{preset}': {preset_config['description']}")
                print()
        else:
            agent_list = list(agents)
            games_per = games if games is not None else 50

        if not agent_list:
            print("No agents specified. Use --agents or --preset.")
            sys.exit(1)

        # Create evaluator
        evaluator = UnifiedEvaluator(
            games_per_matchup=games_per,
            verbose=not quiet,
            update_elo=not no_elo,
        )

        # Check availability and prompt for training if needed
        available, missing = evaluator.check_availability(agent_list)

        if missing and not quiet:
            print("Missing agents:")
            for spec in missing:
                print(f"  - {spec.name}")
            print()

        if not available:
            print("No agents available for evaluation.")
            sys.exit(1)

        # Run evaluation
        try:
            results = evaluator.evaluate(agent_list, games_per_matchup=games_per)
        except KeyboardInterrupt:
            print("\nEvaluation interrupted.")
            sys.exit(130)

        # Print summary
        if not quiet:
            print()
            print(results.summary())

        # Save results if output specified
        if output:
            output_data = {
                "agents": results.agents,
                "elo_ratings": results.elo_ratings,
                "total_games": results.total_games,
                "elapsed_seconds": results.elapsed_seconds,
                "matchups": [
                    {
                        "agent1": m.agent1,
                        "agent2": m.agent2,
                        "games": m.games,
                        "agent1_wins": m.agent1_wins,
                        "agent2_wins": m.agent2_wins,
                        "draws": m.draws,
                    }
                    for m in results.matchups
                ],
            }
            Path(output).write_text(json.dumps(output_data, indent=2))
            print(f"\nResults saved to: {output}")
