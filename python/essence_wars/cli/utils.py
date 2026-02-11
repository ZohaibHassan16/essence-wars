"""Shared utilities for CLI modules."""

from __future__ import annotations

import sys
from pathlib import Path
from typing import Any


def setup_path() -> None:
    """Add parent directory to path for local development."""
    parent = Path(__file__).parent.parent.parent.parent
    scripts_dir = parent / "scripts"
    if scripts_dir.exists() and str(parent) not in sys.path:
        sys.path.insert(0, str(parent))


def get_version() -> str:
    """Get the package version."""
    try:
        from essence_wars import __version__

        return __version__
    except ImportError:
        return "unknown"


def build_args(kwargs: dict[str, Any]) -> list[str]:
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


def run_training_script(script_name: str, kwargs: dict[str, Any]) -> None:
    """Run a training script with the given arguments."""
    import subprocess

    # Build command line arguments
    args = build_args(kwargs)

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
    scripts_dir = Path(__file__).parent.parent.parent / "scripts"
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


def run_script(script_name: str, kwargs: dict[str, Any]) -> None:
    """Run a generic script with the given arguments."""
    run_training_script(script_name, kwargs)
