"""Validation command for Essence Wars."""

from __future__ import annotations

import shutil
import subprocess
import sys
from pathlib import Path
from typing import TYPE_CHECKING, Any

if TYPE_CHECKING:
    import click


def register_validate_command(cli: click.Group) -> None:
    """Register the validate command."""
    import click

    @cli.command("validate")
    @click.option(
        "--games", "-n", default=10, help="Games per matchup per player order"
    )
    @click.option("--run-id", default=None, help="Run ID (default: timestamp)")
    @click.option("--seed", "-s", default=42, type=int, help="Random seed")
    @click.option("--threads", "-j", default=0, type=int, help="Threads (0=all cores)")
    @click.option("--matrix", is_flag=True, help="Print matchup matrix")
    @click.option("--auto-diagnose", is_flag=True, help="Auto-diagnose outlier decks")
    @click.option(
        "--generate-report", is_flag=True, help="Generate HTML report after validation"
    )
    @click.option("--open", "open_browser", is_flag=True, help="Open report in browser")
    def validate(**kwargs: Any) -> None:
        """Run game balance validation across all decks.

        Plays round-robin matchups between all decks using the GreedyBot
        to measure deck balance. Results can be used to generate reports.

        This command wraps the Rust 'validate' binary for high-performance
        parallel game simulation.

        \b
        Examples:
          essence-wars validate                         # Quick validation
          essence-wars validate --games 50 --matrix     # More games, show matrix
          essence-wars validate --generate-report --open  # With HTML report
          essence-wars validate --auto-diagnose         # Diagnose outlier decks
        """
        _run_validate(kwargs)


def _run_validate(kwargs: dict[str, Any]) -> None:
    """Run validation using the Rust binary."""
    # Find the project root (contains Cargo.toml and data/)
    project_root = Path(__file__).parent.parent.parent.parent.parent
    if not (project_root / "Cargo.toml").exists():
        # Try other locations
        for root in [Path.cwd(), Path.cwd().parent]:
            if (root / "Cargo.toml").exists():
                project_root = root
                break

    # Find the validate binary
    binary = shutil.which("validate")
    if binary is None:
        # Try common locations relative to project root
        possible_paths = [
            project_root / "target/release/validate",
            project_root / "target/debug/validate",
        ]
        for p in possible_paths:
            if p.exists():
                binary = str(p)
                break

    if binary is None:
        print("Error: Could not find 'validate' binary.")
        print("Build it with: cargo build --release --bin validate")
        sys.exit(1)

    # Build command
    cmd = [binary, "--games-per-matchup", str(kwargs["games"])]

    if kwargs.get("run_id"):
        cmd.extend(["--run-id", kwargs["run_id"]])
    if kwargs["seed"] != 42:
        cmd.extend(["--seed", str(kwargs["seed"])])
    if kwargs["threads"] != 0:
        cmd.extend(["--threads", str(kwargs["threads"])])
    if kwargs["matrix"]:
        cmd.append("--matrix")
    if kwargs["auto_diagnose"]:
        cmd.append("--auto-diagnose")

    # Run validation from project root (where data/ exists)
    result = subprocess.run(cmd, cwd=str(project_root))

    if result.returncode != 0:
        sys.exit(result.returncode)

    # Generate report if requested
    if kwargs.get("generate_report"):
        from .report import _run_report_generate

        print("\nGenerating HTML report...")
        run_id = kwargs.get("run_id") or "latest"
        report_kwargs = {
            "run_id": run_id,
            "validation_dir": "experiments/validation",
            "tuning_dir": "experiments/mcts",
            "elo_file": None,
            "output": None,
            "theme": "dark",
            "tabs": None,
            "verbose": True,
            "open_browser": kwargs.get("open_browser", False),
        }
        _run_report_generate(report_kwargs)
