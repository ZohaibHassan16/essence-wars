#!/usr/bin/env python3
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
from pathlib import Path
from typing import TYPE_CHECKING, Any

if TYPE_CHECKING:
    import click

    from essence_wars.ratings import LeaderboardEntry

# Lazy imports to keep CLI startup fast


def _setup_path() -> None:
    """Add parent directory to path for local development."""
    parent = Path(__file__).parent.parent.parent
    scripts_dir = parent / "scripts"
    if scripts_dir.exists() and str(parent) not in sys.path:
        sys.path.insert(0, str(parent))


# =============================================================================
# Main CLI Group
# =============================================================================

def main() -> int:
    """Main entry point for the CLI."""
    try:
        import click
    except ImportError:
        print("Error: Click is required for the CLI.")
        print("Install with: pip install click")
        print("Or: uv sync --group analysis")
        return 1

    _setup_path()

    @click.group(context_settings={"help_option_names": ["-h", "--help"]})
    @click.version_option(version=_get_version(), prog_name="essence-wars")
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
    _register_train_commands(cli)
    _register_evaluate_commands(cli)
    _register_report_commands(cli)
    _register_data_commands(cli)
    _register_validate_commands(cli)

    return cli(standalone_mode=False) or 0


def _get_version() -> str:
    """Get the package version."""
    try:
        from essence_wars import __version__
        return __version__
    except ImportError:
        return "unknown"


# =============================================================================
# Train Command Group
# =============================================================================

def _register_train_commands(cli: click.Group) -> None:
    """Register training commands."""
    import click

    @cli.group()
    def train() -> None:
        """Train ML agents for Essence Wars.

        \b
        Available agents:
          ppo                 - Proximal Policy Optimization
          alphazero           - AlphaZero (MCTS + neural network)
          behavioral-cloning  - Supervised learning from MCTS data
          card2vec            - Card embedding model
          decision-transformer - Sequence modeling approach
        """

    @train.command("ppo")
    @click.option("--timesteps", "-t", default=500_000, help="Total training timesteps")
    @click.option("--num-envs", default=64, help="Number of parallel environments")
    @click.option("--lr", default=3e-4, help="Learning rate")
    @click.option("--gamma", default=0.99, help="Discount factor")
    @click.option("--ent-coef", default=0.02, help="Entropy coefficient")
    @click.option("--hidden-dim", default=256, help="Hidden layer dimension")
    @click.option("--observation-mode", type=click.Choice(["flat", "embedded", "embedded_pretrained"]),
                  default="flat", help="Observation mode")
    @click.option("--player-faction", type=click.Choice(["argentum", "symbiote", "obsidion"]),
                  default=None, help="Train as faction specialist")
    @click.option("--player-deck", default=None, help="Train with specific deck")
    @click.option("--reward-shaping", is_flag=True, help="Enable dense reward shaping")
    @click.option("--eval-interval", default=25_000, help="Evaluation interval")
    @click.option("--save-path", type=click.Path(), default=None, help="Save directory")
    @click.option("--load", type=click.Path(exists=True), default=None, help="Load checkpoint")
    @click.option("--seed", default=42, help="Random seed")
    @click.option("--device", default="auto", help="Device (cpu, cuda, auto)")
    @click.option("--no-tensorboard", is_flag=True, help="Disable TensorBoard logging")
    def train_ppo(**kwargs: Any) -> None:
        """Train a PPO (Proximal Policy Optimization) agent.

        \b
        Examples:
          essence-wars train ppo --timesteps 300000
          essence-wars train ppo --player-faction argentum --timesteps 500000
          essence-wars train ppo --observation-mode embedded --embed-dim 64
        """
        _run_training_script("train_ppo", kwargs)

    @train.command("alphazero")
    @click.option("--iterations", "-i", default=100, help="Number of training iterations")
    @click.option("--games-per-iter", default=100, help="Self-play games per iteration")
    @click.option("--training-steps", default=100, help="Training steps per iteration")
    @click.option("--batch-size", default=256, help="Training batch size")
    @click.option("--sims", default=100, help="MCTS simulations per move")
    @click.option("--c-puct", default=1.5, help="MCTS exploration constant")
    @click.option("--hidden-dim", default=256, help="Network hidden dimension")
    @click.option("--lr", default=1e-3, help="Learning rate")
    @click.option("--save-path", type=click.Path(), default=None, help="Save directory")
    @click.option("--load", type=click.Path(exists=True), default=None, help="Load checkpoint")
    @click.option("--bc-data", type=click.Path(exists=True), default=None,
                  help="BC data for warm-start (prevents catastrophic forgetting)")
    @click.option("--bc-ratio", default=0.7, help="Ratio of BC data in training batches")
    @click.option("--seed", default=42, help="Random seed")
    @click.option("--device", default="auto", help="Device (cpu, cuda, auto)")
    def train_alphazero(**kwargs: Any) -> None:
        """Train an AlphaZero agent using MCTS self-play.

        AlphaZero combines Monte Carlo Tree Search with a neural network
        that predicts both policy (move probabilities) and value (win probability).

        \b
        Examples:
          essence-wars train alphazero --iterations 100
          essence-wars train alphazero --sims 200 --games-per-iter 200
          essence-wars train alphazero --load model.pt --bc-data data.jsonl.gz
        """
        _run_training_script("train_alphazero", kwargs)

    @train.command("behavioral-cloning")
    @click.option("--data", "-d", type=click.Path(exists=True), required=True,
                  help="Training data file (JSONL or JSONL.gz)")
    @click.option("--epochs", "-e", default=10, help="Number of training epochs")
    @click.option("--batch-size", default=256, help="Training batch size")
    @click.option("--lr", default=1e-3, help="Learning rate")
    @click.option("--hidden-dim", default=256, help="Network hidden dimension")
    @click.option("--save-path", type=click.Path(), default=None, help="Save directory")
    @click.option("--seed", default=42, help="Random seed")
    @click.option("--device", default="auto", help="Device (cpu, cuda, auto)")
    def train_bc(**kwargs: Any) -> None:
        """Train a Behavioral Cloning agent from MCTS demonstration data.

        Supervised learning approach that imitates MCTS policy decisions.
        Fast to train and provides a good baseline.

        \b
        Examples:
          essence-wars train behavioral-cloning --data mcts_data.jsonl.gz
          essence-wars train behavioral-cloning --data mcts_data.jsonl.gz --epochs 20
        """
        _run_training_script("train_behavioral_cloning", kwargs)

    @train.command("card2vec")
    @click.option("--data", "-d", type=click.Path(exists=True), required=True,
                  help="Training data file")
    @click.option("--embed-dim", default=64, help="Embedding dimension")
    @click.option("--epochs", "-e", default=50, help="Number of training epochs")
    @click.option("--batch-size", default=512, help="Training batch size")
    @click.option("--lr", default=1e-3, help="Learning rate")
    @click.option("--save-path", type=click.Path(), default=None, help="Save directory")
    @click.option("--seed", default=42, help="Random seed")
    @click.option("--device", default="auto", help="Device (cpu, cuda, auto)")
    def train_card2vec(**kwargs: Any) -> None:
        """Train card embeddings using Card2Vec approach.

        Learns dense vector representations for cards based on co-occurrence
        patterns in gameplay. Can be used with PPO's embedded observation mode.

        \b
        Examples:
          essence-wars train card2vec --data mcts_data.jsonl.gz --embed-dim 64
        """
        _run_training_script("train_card2vec", kwargs)

    @train.command("decision-transformer")
    @click.option("--data", "-d", type=click.Path(exists=True), required=True,
                  help="Training data file")
    @click.option("--epochs", "-e", default=10, help="Number of training epochs")
    @click.option("--batch-size", default=64, help="Training batch size")
    @click.option("--context-length", default=20, help="Sequence context length")
    @click.option("--hidden-dim", default=128, help="Transformer hidden dimension")
    @click.option("--num-layers", default=4, help="Number of transformer layers")
    @click.option("--num-heads", default=4, help="Number of attention heads")
    @click.option("--lr", default=1e-4, help="Learning rate")
    @click.option("--save-path", type=click.Path(), default=None, help="Save directory")
    @click.option("--seed", default=42, help="Random seed")
    @click.option("--device", default="auto", help="Device (cpu, cuda, auto)")
    def train_dt(**kwargs: Any) -> None:
        """Train a Decision Transformer agent.

        Sequence modeling approach that learns to predict actions conditioned
        on desired returns (reward-to-go).

        \b
        Examples:
          essence-wars train decision-transformer --data mcts_data.jsonl.gz
          essence-wars train decision-transformer --context-length 30 --num-layers 6
        """
        _run_training_script("train_decision_transformer", kwargs)


# =============================================================================
# Evaluate Command Group
# =============================================================================

def _register_evaluate_commands(cli: click.Group) -> None:
    """Register evaluation commands."""
    import click

    @cli.command("benchmark")
    @click.option("--checkpoint", "-c", type=click.Path(exists=True), required=True,
                  help="Path to agent checkpoint")
    @click.option("--output", "-o", type=click.Path(), default="benchmark_results.json",
                  help="Output JSON file")
    @click.option("--full-eval", is_flag=True,
                  help="Full evaluation (100 games per opponent vs 20)")
    @click.option("--games-per-opponent", type=int, default=None,
                  help="Override games per opponent")
    @click.option("--baselines", multiple=True, default=None,
                  help="Baselines to evaluate against (random, greedy, mcts50, mcts100)")
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
    @click.option("--checkpoint", "-c", type=click.Path(exists=True), required=True,
                  help="Path to neural network checkpoint")
    @click.option("--sims", default=100, help="MCTS simulations per move")
    @click.option("--games", "-n", default=100, help="Number of evaluation games")
    @click.option("--opponent", default="greedy",
                  type=click.Choice(["random", "greedy", "mcts"]),
                  help="Opponent type")
    @click.option("--output", "-o", type=click.Path(), default=None,
                  help="Output JSON file")
    def evaluate_mcts(**kwargs: Any) -> None:
        """Evaluate a neural MCTS agent.

        Uses the neural network as policy prior and value estimate within MCTS.

        \b
        Examples:
          essence-wars evaluate-mcts --checkpoint model.pt --sims 200
          essence-wars evaluate-mcts --checkpoint model.pt --opponent mcts --games 50
        """
        _run_script("evaluate_neural_mcts", kwargs)


# =============================================================================
# Report Command Group
# =============================================================================

def _register_report_commands(cli: click.Group) -> None:
    """Register report generation commands."""
    import click

    @cli.group()
    def report() -> None:
        """Generate HTML reports and dashboards.

        \b
        Report types:
          generate      - Single validation run report
          generate-all  - Batch generate all missing reports
          aggregate     - Aggregated dashboard across runs
          leaderboard   - Agent/deck leaderboard
        """

    @report.command("generate")
    @click.option("--run-id", "-r", default="latest",
                  help="Validation run ID ('latest' for most recent)")
    @click.option("--validation-dir", type=click.Path(exists=True),
                  default="experiments/validation", help="Validation results directory")
    @click.option("--tuning-dir", type=click.Path(),
                  default="experiments/mcts", help="Tuning experiments directory")
    @click.option("--elo-file", type=click.Path(),
                  default=None, help="ELO ratings file")
    @click.option("--output", "-o", type=click.Path(), default=None,
                  help="Output directory")
    @click.option("--tabs", multiple=True, default=None,
                  type=click.Choice(["overview", "validation", "tuning", "elo"]),
                  help="Tabs to include (default: all available)")
    @click.option("--theme", type=click.Choice(["dark", "light"]), default="dark",
                  help="Color theme")
    @click.option("--open", "open_browser", is_flag=True,
                  help="Open report in browser after generation")
    @click.option("--verbose", "-v", is_flag=True, help="Verbose output")
    def report_generate(**kwargs: Any) -> None:
        """Generate an HTML report for a validation run.

        Creates a multi-tab report with overview metrics, validation results,
        tuning experiment data, and ELO ratings.

        \b
        Examples:
          essence-wars report generate --run-id latest --open
          essence-wars report generate --run-id 2026-01-31_1542
          essence-wars report generate --tabs overview validation --theme light
        """
        _run_report_generate(kwargs)

    @report.command("generate-all")
    @click.option("--validation-dir", type=click.Path(exists=True),
                  default="experiments/validation", help="Validation results directory")
    @click.option("--tuning-dir", type=click.Path(),
                  default="experiments/mcts", help="Tuning experiments directory")
    @click.option("--since", type=str, default=None,
                  help="Only include runs after this date (YYYY-MM-DD)")
    @click.option("--limit", type=int, default=None,
                  help="Only process N most recent runs")
    @click.option("--force", is_flag=True,
                  help="Regenerate reports even if they exist")
    @click.option("--theme", type=click.Choice(["dark", "light"]), default="dark",
                  help="Color theme")
    @click.option("--verbose", "-v", is_flag=True, help="Verbose output")
    def report_generate_all(**kwargs: Any) -> None:
        """Generate reports for all validation runs.

        Skips runs that already have reports unless --force is specified.
        Also regenerates the aggregated dashboard.

        \b
        Examples:
          essence-wars report generate-all
          essence-wars report generate-all --since 2026-01-25
          essence-wars report generate-all --limit 10 --force
        """
        _run_report_generate_all(kwargs)

    @report.command("aggregate")
    @click.option("--validation-dir", type=click.Path(exists=True),
                  default="experiments/validation", help="Validation results directory")
    @click.option("--tuning-dir", type=click.Path(),
                  default="experiments/mcts", help="Tuning experiments directory")
    @click.option("--limit", type=int, default=20,
                  help="Maximum runs to include")
    @click.option("--output", "-o", type=click.Path(), default=None,
                  help="Output directory")
    @click.option("--theme", type=click.Choice(["dark", "light"]), default="dark",
                  help="Color theme")
    @click.option("--open", "open_browser", is_flag=True,
                  help="Open dashboard in browser")
    def report_aggregate(**kwargs: Any) -> None:
        """Generate an aggregated dashboard across all runs.

        Shows health score timeline, persistent outliers, and
        summary tables for validation and tuning experiments.

        \b
        Examples:
          essence-wars report aggregate --open
          essence-wars report aggregate --limit 50
        """
        _run_report_aggregate(kwargs)

    @report.command("leaderboard")
    @click.option("--elo-file", type=click.Path(exists=True),
                  default="data/ratings/deck_elo.json", help="ELO ratings file")
    @click.option("--output", "-o", type=click.Path(), default=None,
                  help="Output HTML file")
    @click.option("--format", "output_format", type=click.Choice(["html", "json", "table"]),
                  default="table", help="Output format")
    def report_leaderboard(**kwargs: Any) -> None:
        """Display deck/agent leaderboard from ELO ratings.

        \b
        Examples:
          essence-wars report leaderboard
          essence-wars report leaderboard --format html --output leaderboard.html
          essence-wars report leaderboard --format json
        """
        _run_leaderboard(kwargs)


# =============================================================================
# Data Command Group
# =============================================================================

def _register_data_commands(cli: click.Group) -> None:
    """Register data generation commands."""
    import click

    @cli.group()
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
    @click.option("--output", "-o", type=click.Path(), required=True,
                  help="Output file path (.jsonl or .jsonl.gz)")
    @click.option("--workers", "-j", default=None, type=int,
                  help="Number of worker processes (default: CPU count)")
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
        _run_script("generate_distillation_data", kwargs)

    @data.command("generate-exits")
    @click.option("--games", "-n", default=10000, help="Number of games to generate")
    @click.option("--output", "-o", type=click.Path(), required=True,
                  help="Output file path")
    @click.option("--bot", default="greedy", help="Bot type for game generation")
    @click.option("--seed", default=None, type=int, help="Random seed")
    def data_exits(**kwargs: Any) -> None:
        """Generate game exit/outcome data.

        Records final game states and outcomes for analysis.

        \b
        Examples:
          essence-wars data generate-exits --games 10000 --output exits.jsonl
        """
        _run_script("generate_exit_data", kwargs)


# =============================================================================
# Validate Command
# =============================================================================

def _register_validate_commands(cli: click.Group) -> None:
    """Register validation commands."""
    import click

    @cli.command("validate")
    @click.option("--games", "-n", default=10, help="Games per matchup per player order")
    @click.option("--run-id", default=None, help="Run ID (default: timestamp)")
    @click.option("--seed", "-s", default=42, type=int, help="Random seed")
    @click.option("--threads", "-j", default=0, type=int, help="Threads (0=all cores)")
    @click.option("--matrix", is_flag=True, help="Print matchup matrix")
    @click.option("--auto-diagnose", is_flag=True, help="Auto-diagnose outlier decks")
    @click.option("--generate-report", is_flag=True, help="Generate HTML report after validation")
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
    import shutil
    import subprocess
    from pathlib import Path

    # Find the project root (contains Cargo.toml and data/)
    project_root = Path(__file__).parent.parent.parent.parent
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


# =============================================================================
# Helper Functions
# =============================================================================

def _run_training_script(script_name: str, kwargs: dict[str, Any]) -> None:
    """Run a training script with the given arguments."""
    import subprocess
    import sys

    # Build command line arguments
    args = _build_args(kwargs)

    # Map script names to their actual paths under python/scripts/
    script_paths: dict[str, str] = {
        # Training scripts
        "train_ppo": "training/ppo.py",
        "train_alphazero": "training/alphazero.py",
        "train_behavioral_cloning": "training/behavioral_cloning.py",
        "train_card2vec": "training/card2vec.py",
        "train_decision_transformer": "training/decision_transformer.py",
        # Evaluation scripts
        "evaluate_neural_mcts": "evaluation/neural_mcts.py",
        # Data generation scripts
        "generate_distillation_data": "data/generate_distillation.py",
        "generate_exit_data": "data/generate_exits.py",
    }

    # Find script path
    scripts_dir = Path(__file__).parent.parent / "scripts"
    if script_name in script_paths:
        script_path = scripts_dir / script_paths[script_name]
    else:
        # Fallback: try to find in any subdirectory
        script_path = scripts_dir / f"{script_name}.py"

    if not script_path.exists():
        print(f"Error: Script not found: {script_path}")
        print(f"Available scripts: {', '.join(script_paths.keys())}")
        sys.exit(1)

    # Run the script
    cmd = [sys.executable, str(script_path), *args]
    print(f"Running: {' '.join(cmd[:3])}...")
    result = subprocess.run(cmd)
    sys.exit(result.returncode)


def _run_script(script_name: str, kwargs: dict[str, Any]) -> None:
    """Run a generic script with the given arguments."""
    _run_training_script(script_name, kwargs)


def _run_benchmark(kwargs: dict[str, Any]) -> None:
    """Run benchmark evaluation."""
    try:
        from essence_wars.benchmark.agents import NeuralAgent
        from essence_wars.benchmark.api import EssenceWarsBenchmark
    except ImportError as e:
        print(f"Error: Missing dependencies for benchmarking: {e}")
        print("Install with: pip install essence-wars[train]")
        sys.exit(1)

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
        baselines = ["random", "greedy", "mcts50", "mcts100"] if full_eval else ["random", "greedy"]
    else:
        baselines = list(baselines)

    print(f"Running evaluation ({games} games per opponent)...")
    benchmark = EssenceWarsBenchmark(games_per_opponent=games, verbose=True)
    results = benchmark.evaluate(agent, baselines=baselines)

    # Build and save results
    import json
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


def _run_report_generate(kwargs: dict[str, Any]) -> None:
    """Run report generation."""
    try:
        from essence_wars.analysis.report import ReportGenerator
    except ImportError as e:
        print(f"Error: Missing dependencies for report generation: {e}")
        print("Install with: pip install essence-wars[analysis]")
        print("Or: uv sync --group analysis")
        sys.exit(1)

    import webbrowser

    generator = ReportGenerator(
        output_dir=kwargs.get("output"),
        theme=kwargs["theme"],
    )

    tabs = list(kwargs["tabs"]) if kwargs["tabs"] else None

    if kwargs["verbose"]:
        print(f"Generating report for: {kwargs['run_id']}")

    output_path = generator.generate_validation_report(
        run_id=kwargs["run_id"],
        validation_dir=Path(kwargs["validation_dir"]),
        tuning_dir=Path(kwargs["tuning_dir"]),
        elo_file=kwargs.get("elo_file"),
        tabs=tabs,
    )

    print(f"Report generated: {output_path}")

    if kwargs["open_browser"]:
        webbrowser.open(output_path.absolute().as_uri())


def _run_report_generate_all(kwargs: dict[str, Any]) -> None:
    """Run batch report generation."""
    try:
        from essence_wars.analysis.report import ReportGenerator
    except ImportError as e:
        print(f"Error: Missing dependencies for report generation: {e}")
        print("Install with: pip install essence-wars[analysis]")
        sys.exit(1)

    from datetime import datetime

    # Parse since date
    since = None
    if kwargs["since"]:
        try:
            since = datetime.strptime(kwargs["since"], "%Y-%m-%d")
        except ValueError:
            print("Error: Invalid date format. Use YYYY-MM-DD.")
            sys.exit(1)

    generator = ReportGenerator(theme=kwargs["theme"])

    if kwargs["verbose"]:
        print(f"Scanning for validation runs in: {kwargs['validation_dir']}")

    generated = generator.generate_all_reports(
        validation_dir=Path(kwargs["validation_dir"]),
        tuning_dir=Path(kwargs["tuning_dir"]),
        since=since,
        limit=kwargs.get("limit"),
        force=kwargs["force"],
    )

    if generated:
        print(f"Generated {len(generated)} report(s)")
        for path in generated[:5]:
            print(f"  {path}")
        if len(generated) > 5:
            print(f"  ... and {len(generated) - 5} more")
    else:
        print("No new reports to generate (all up to date)")

    # Regenerate dashboard
    if kwargs["verbose"]:
        print("Regenerating aggregated dashboard...")
    dashboard_path = generator.generate_aggregated_dashboard(
        validation_dir=Path(kwargs["validation_dir"]),
        tuning_dir=Path(kwargs["tuning_dir"]),
        limit=kwargs.get("limit") or 20,
    )
    print(f"Dashboard: {dashboard_path}")


def _run_report_aggregate(kwargs: dict[str, Any]) -> None:
    """Run aggregated dashboard generation."""
    try:
        from essence_wars.analysis.report import ReportGenerator
    except ImportError as e:
        print(f"Error: Missing dependencies for report generation: {e}")
        print("Install with: pip install essence-wars[analysis]")
        sys.exit(1)

    import webbrowser

    generator = ReportGenerator(
        output_dir=kwargs.get("output"),
        theme=kwargs["theme"],
    )

    output_path = generator.generate_aggregated_dashboard(
        validation_dir=Path(kwargs["validation_dir"]),
        tuning_dir=Path(kwargs["tuning_dir"]),
        limit=kwargs["limit"],
    )

    print(f"Dashboard generated: {output_path}")

    if kwargs["open_browser"]:
        webbrowser.open(output_path.absolute().as_uri())


def _run_leaderboard(kwargs: dict[str, Any]) -> None:
    """Display leaderboard from ELO ratings using unified ratings system."""
    import json
    from pathlib import Path

    try:
        from essence_wars.ratings import UnifiedRatings
    except ImportError:
        print("Error: ratings module not available")
        sys.exit(1)

    # Load unified ratings (handles missing files gracefully)
    elo_file = kwargs.get("elo_file")
    ratings = UnifiedRatings.load(deck_file=elo_file)

    entries = ratings.get_leaderboard()
    if not entries:
        print("No ratings data found.")
        return

    output_format = kwargs["output_format"]

    if output_format == "json":
        print(json.dumps({"leaderboard": [
            {
                "rank": e.rank,
                "identifier": e.identifier,
                "display_name": e.display_name,
                "rating": e.rating,
                "games": e.games,
                "wins": e.wins,
                "losses": e.losses,
                "draws": e.draws,
                "win_rate": e.win_rate,
                "category": e.category.value,
                "faction": e.faction,
                "agent_type": e.agent_type,
            }
            for e in entries
        ]}, indent=2))

    elif output_format == "table":
        print("\n" + "=" * 75)
        print("ESSENCE WARS LEADERBOARD")
        print("=" * 75)
        print(f"{'#':>3}  {'Name':<25}  {'Type':>6}  {'ELO':>6}  {'W-L-D':>10}  {'Win%':>6}")
        print("-" * 75)
        for e in entries:
            wld = f"{e.wins}-{e.losses}-{e.draws}"
            cat = e.category.value[:5].upper()
            print(f"{e.rank:>3}  {e.display_name:<25}  {cat:>6}  {e.rating:>6.0f}  {wld:>10}  {e.win_rate * 100:>5.1f}%")
        print("=" * 75)

        # Show summary
        summary = ratings.get_summary()
        print(f"\nTotal: {summary.get('total_entries', 0)} entries "
              f"({summary.get('total_decks', 0)} decks, {summary.get('total_agents', 0)} agents)")

    else:  # HTML
        output = kwargs.get("output") or "leaderboard.html"
        # Generate HTML using unified entries
        html = _generate_leaderboard_html_unified(entries)
        Path(output).write_text(html)
        print(f"Leaderboard saved to: {output}")


def _generate_leaderboard_html(sorted_ratings: list[tuple[str, dict[str, Any]]]) -> str:
    """Generate a simple HTML leaderboard."""
    rows = []
    for i, (deck_id, stats) in enumerate(sorted_ratings, 1):
        rating = stats.get("rating", 1500)
        wins = stats.get("wins", 0)
        losses = stats.get("losses", 0)
        draws = stats.get("draws", 0)
        games = stats.get("games", wins + losses + draws)
        win_rate = (wins / games * 100) if games > 0 else 0
        rows.append(f"""
        <tr>
            <td>{i}</td>
            <td>{deck_id}</td>
            <td>{rating:.0f}</td>
            <td>{wins}-{losses}-{draws}</td>
            <td>{win_rate:.1f}%</td>
        </tr>
        """)

    return f"""<!DOCTYPE html>
<html>
<head>
    <title>Essence Wars Leaderboard</title>
    <style>
        body {{ font-family: sans-serif; max-width: 800px; margin: 0 auto; padding: 20px; }}
        table {{ width: 100%; border-collapse: collapse; }}
        th, td {{ padding: 10px; text-align: left; border-bottom: 1px solid #ddd; }}
        th {{ background: #333; color: white; }}
        tr:hover {{ background: #f5f5f5; }}
    </style>
</head>
<body>
    <h1>Essence Wars Leaderboard</h1>
    <table>
        <thead><tr><th>#</th><th>Deck</th><th>ELO</th><th>W-L-D</th><th>Win%</th></tr></thead>
        <tbody>{"".join(rows)}</tbody>
    </table>
</body>
</html>"""


def _generate_leaderboard_html_unified(entries: list[LeaderboardEntry]) -> str:
    """Generate HTML leaderboard from unified rating entries."""
    rows = []
    for e in entries:
        category_badge = "deck" if e.category.value == "deck" else "agent"
        rows.append(f"""
        <tr>
            <td>{e.rank}</td>
            <td>{e.display_name}</td>
            <td><span class="badge {category_badge}">{e.category.value.upper()}</span></td>
            <td>{e.rating:.0f}</td>
            <td>{e.wins}-{e.losses}-{e.draws}</td>
            <td>{e.win_rate * 100:.1f}%</td>
        </tr>
        """)

    return f"""<!DOCTYPE html>
<html>
<head>
    <title>Essence Wars Leaderboard</title>
    <style>
        body {{ font-family: sans-serif; max-width: 900px; margin: 0 auto; padding: 20px; background: #1a1a2e; color: #eaeaea; }}
        h1 {{ color: #e94560; }}
        table {{ width: 100%; border-collapse: collapse; }}
        th, td {{ padding: 12px; text-align: left; border-bottom: 1px solid #333; }}
        th {{ background: #16213e; color: #D4AF37; }}
        tr:hover {{ background: #0f3460; }}
        .badge {{ padding: 2px 8px; border-radius: 4px; font-size: 0.8em; }}
        .badge.deck {{ background: #D4AF37; color: #000; }}
        .badge.agent {{ background: #00FFFF; color: #000; }}
    </style>
</head>
<body>
    <h1>Essence Wars Leaderboard</h1>
    <table>
        <thead><tr><th>#</th><th>Name</th><th>Type</th><th>ELO</th><th>W-L-D</th><th>Win%</th></tr></thead>
        <tbody>{"".join(rows)}</tbody>
    </table>
</body>
</html>"""


def _build_args(kwargs: dict[str, Any]) -> list[str]:
    """Convert kwargs dict to command-line arguments."""
    args = []
    for key, value in kwargs.items():
        if value is None:
            continue
        if value is True:
            args.append(f"--{key.replace('_', '-')}")
        elif value is False:
            continue
        elif isinstance(value, (list, tuple)):
            for v in value:
                args.extend([f"--{key.replace('_', '-')}", str(v)])
        else:
            args.extend([f"--{key.replace('_', '-')}", str(value)])
    return args


# =============================================================================
# Entry Point
# =============================================================================

if __name__ == "__main__":
    sys.exit(main())
