"""Unified CLI for Essence Wars ML Research Toolkit.

This module provides a single entry point for all Essence Wars Python tools:
- Training ML agents (PPO, AlphaZero, Behavioral Cloning, etc.)
- Evaluating and benchmarking agents
- Generating reports and dashboards
- Data generation for training

Usage:
    essence-wars --help                    # Show all commands
    essence-wars train --help              # Training commands
    essence-wars report --help             # Report generation
    essence-wars benchmark --help          # Agent evaluation

Examples:
    # Train a PPO agent
    essence-wars train ppo --timesteps 500000

    # Generate a report for the latest validation run
    essence-wars report generate --run-id latest --open

    # Benchmark an agent
    essence-wars benchmark --checkpoint models/my_agent.pt

    # Generate training data
    essence-wars data generate-distillation --games 10000
"""

from __future__ import annotations

import sys

from .utils import get_version, setup_path


def main() -> int:
    """Main entry point for the CLI."""
    try:
        import click
    except ImportError:
        print("Error: Click is required for the CLI.")
        print("Install with: pip install click")
        print("Or: uv sync --group analysis")
        return 1

    setup_path()

    # Lazy imports to keep CLI startup fast
    from .data import create_data_group
    from .evaluate import register_evaluate_commands
    from .models import create_models_group
    from .report import create_report_group
    from .train import create_train_group
    from .unified_eval import register_unified_eval_command
    from .validate import register_validate_command

    @click.group(context_settings={"help_option_names": ["-h", "--help"]})
    @click.version_option(version=get_version(), prog_name="essence-wars")
    def cli() -> None:
        """Essence Wars - ML Research Toolkit for AI Card Game Development.

        A unified command-line interface for training ML agents, running
        benchmarks, and generating analysis reports.

        \b
        Quick Start:
          essence-wars train ppo --timesteps 100000
          essence-wars report generate --run-id latest
          essence-wars benchmark --checkpoint model.pt
        """

    # Register command groups
    cli.add_command(create_train_group())
    register_evaluate_commands(cli)
    cli.add_command(create_models_group())
    register_unified_eval_command(cli)
    cli.add_command(create_report_group())
    cli.add_command(create_data_group())
    register_validate_command(cli)

    return cli(standalone_mode=False) or 0


if __name__ == "__main__":
    sys.exit(main())
