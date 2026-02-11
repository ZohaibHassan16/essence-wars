"""Data generation commands for Essence Wars."""

from __future__ import annotations

from typing import TYPE_CHECKING, Any

if TYPE_CHECKING:
    import click


def create_data_group() -> click.Group:
    """Create the data command group."""
    import click

    from .utils import run_script

    @click.group()
    def data() -> None:
        """Generate training data for ML agents.

        \b
        Data types:
          generate-distillation  - MCTS demonstration data
          generate-exits         - Game exit/outcome data
        """

    @data.command("generate-distillation")
    @click.option("--games", "-n", default=10000, help="Number of games to generate")
    @click.option("--sims", default=100, help="MCTS simulations per move")
    @click.option(
        "--output",
        "-o",
        type=click.Path(),
        required=True,
        help="Output file path (.jsonl or .jsonl.gz)",
    )
    @click.option(
        "--workers",
        "-j",
        default=None,
        type=int,
        help="Number of worker processes (default: CPU count)",
    )
    @click.option("--seed", default=None, type=int, help="Random seed")
    @click.option("--progress", is_flag=True, help="Show progress bar")
    def data_distillation(**kwargs: Any) -> None:
        """Generate MCTS demonstration data for training.

        Creates a dataset of (state, MCTS policy, value) tuples that can be
        used for behavioral cloning or AlphaZero warm-start.

        \b
        Examples:
          essence-wars data generate-distillation --games 10000 --output data.jsonl.gz
          essence-wars data generate-distillation --games 50000 --sims 200 --workers 8
        """
        run_script("generate_distillation_data", kwargs)

    @data.command("generate-exits")
    @click.option("--games", "-n", default=10000, help="Number of games to generate")
    @click.option(
        "--output", "-o", type=click.Path(), required=True, help="Output file path"
    )
    @click.option("--bot", default="greedy", help="Bot type for game generation")
    @click.option("--seed", default=None, type=int, help="Random seed")
    def data_exits(**kwargs: Any) -> None:
        """Generate game exit/outcome data.

        Records final game states and outcomes for analysis.

        \b
        Examples:
          essence-wars data generate-exits --games 10000 --output exits.jsonl
        """
        run_script("generate_exit_data", kwargs)

    return data
