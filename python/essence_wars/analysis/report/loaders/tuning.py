"""Tuning experiment data loader for HTML report generation."""

from __future__ import annotations

import re
from dataclasses import dataclass
from datetime import datetime
from pathlib import Path
from typing import Any

import pandas as pd


@dataclass
class TuningData:
    """Parsed tuning experiment data."""

    experiment_id: str
    tag: str
    mode: str
    timestamp: datetime
    generations_df: pd.DataFrame
    final_fitness: float
    final_win_rate: float
    final_sigma: float
    total_time_secs: float
    total_generations: int
    version: str | None = None
    git_hash: str | None = None

    def get_fitness_curve(self) -> tuple[list[int], list[float]]:
        """Return generation numbers and fitness values for plotting."""
        return (
            self.generations_df["generation"].tolist(),
            self.generations_df["fitness"].tolist(),
        )

    def get_win_rate_curve(self) -> tuple[list[int], list[float]]:
        """Return generation numbers and win rate values for plotting."""
        return (
            self.generations_df["generation"].tolist(),
            self.generations_df["win_rate"].tolist(),
        )

    def get_sigma_curve(self) -> tuple[list[int], list[float]]:
        """Return generation numbers and sigma values for plotting."""
        return (
            self.generations_df["generation"].tolist(),
            self.generations_df["sigma"].tolist(),
        )

    def get_convergence_info(self) -> dict[str, Any]:
        """Analyze convergence status."""
        df = self.generations_df
        if len(df) < 10:
            return {"converged": False, "reason": "Not enough generations"}

        # Check if fitness has been stable in last 20% of generations
        last_n = max(10, len(df) // 5)
        last_fitness = df["fitness"].tail(last_n)
        fitness_std = last_fitness.std()

        # Check sigma trend
        last_sigma = df["sigma"].tail(last_n)
        sigma_trend = last_sigma.iloc[-1] / last_sigma.iloc[0] if last_sigma.iloc[0] > 0 else 1.0

        converged = fitness_std < 0.5 and sigma_trend < 0.8
        if converged:
            reason = f"Fitness stable (std={fitness_std:.2f}), sigma decreasing ({sigma_trend:.2f}x)"
        elif fitness_std < 0.5:
            reason = f"Fitness stable but sigma not converging ({sigma_trend:.2f}x)"
        else:
            reason = f"Fitness still changing (std={fitness_std:.2f})"

        return {
            "converged": converged,
            "reason": reason,
            "fitness_std": fitness_std,
            "sigma_trend": sigma_trend,
            "last_n": last_n,
        }


def parse_experiment_id(exp_id: str) -> dict[str, Any]:
    """Parse experiment ID into components.

    Format: YYYY-MM-DD_HHMM_tag
    Example: 2026-01-14_0719_generalist-v0.4
    """
    # Try to parse timestamp and tag
    match = re.match(r"(\d{4}-\d{2}-\d{2})_(\d{4})_(.+)", exp_id)
    if match:
        date_str, time_str, tag = match.groups()
        timestamp = datetime.strptime(f"{date_str}_{time_str}", "%Y-%m-%d_%H%M")

        # Try to determine mode from tag
        tag_lower = tag.lower()
        if "generalist" in tag_lower:
            mode = "generalist"
        elif "argentum" in tag_lower or "symbiote" in tag_lower or "obsidion" in tag_lower:
            mode = "specialist"
        elif "aggro" in tag_lower or "control" in tag_lower or "tempo" in tag_lower or "midrange" in tag_lower:
            mode = "archetype"
        else:
            mode = "unknown"

        return {"timestamp": timestamp, "tag": tag, "mode": mode}

    return {"timestamp": datetime.now(), "tag": exp_id, "mode": "unknown"}


def load_tuning_data(
    experiment_id: str | None = None,
    base_dir: Path | None = None,
) -> TuningData:
    """Load tuning experiment data.

    Args:
        experiment_id: Experiment ID or 'latest'. If None, uses latest.
        base_dir: Base directory for experiments. Defaults to experiments/mcts.

    Returns:
        TuningData with parsed experiment data.

    Raises:
        FileNotFoundError: If experiment not found.
    """
    if base_dir is None:
        base_dir = Path("experiments/mcts")

    if experiment_id is None or experiment_id == "latest":
        exp_dir = find_latest_tuning(base_dir)
        if exp_dir is None:
            raise FileNotFoundError(f"No tuning experiments found in {base_dir}")
    else:
        # Try exact match first
        exp_dir = base_dir / experiment_id
        if not exp_dir.exists():
            # Try partial match
            matches = list(base_dir.glob(f"*{experiment_id}*"))
            if matches:
                exp_dir = max(matches, key=lambda p: p.name)
            else:
                raise FileNotFoundError(f"Tuning experiment '{experiment_id}' not found in {base_dir}")

    return _load_from_directory(exp_dir)


def find_latest_tuning(base_dir: Path) -> Path | None:
    """Find the most recent tuning experiment directory."""
    if not base_dir.exists():
        return None

    # Look for directories with stats.csv
    candidates = [d for d in base_dir.iterdir() if d.is_dir() and (d / "stats.csv").exists()]

    if not candidates:
        return None

    # Sort by name (which includes timestamp) and return latest
    return max(candidates, key=lambda p: p.name)


def find_all_tuning_experiments(
    base_dir: Path | None = None,
    since: datetime | None = None,
    limit: int | None = None,
) -> list[Path]:
    """Find all tuning experiment directories.

    Args:
        base_dir: Base directory for experiments. Defaults to experiments/mcts.
        since: Only include experiments after this date.
        limit: Maximum number to return (most recent first).

    Returns:
        List of experiment directories, sorted by timestamp descending.
    """
    if base_dir is None:
        base_dir = Path("experiments/mcts")

    if not base_dir.exists():
        return []

    # Find all directories with stats.csv
    candidates = [d for d in base_dir.iterdir() if d.is_dir() and (d / "stats.csv").exists()]

    # Filter by date if specified
    if since:
        filtered = []
        for d in candidates:
            parsed = parse_experiment_id(d.name)
            if parsed["timestamp"] >= since:
                filtered.append(d)
        candidates = filtered

    # Sort by name descending (includes timestamp)
    candidates.sort(key=lambda p: p.name, reverse=True)

    if limit:
        candidates = candidates[:limit]

    return candidates


def _load_from_directory(exp_dir: Path) -> TuningData:
    """Load tuning data from a specific experiment directory."""
    experiment_id = exp_dir.name

    # Load stats.csv
    stats_path = exp_dir / "stats.csv"
    if not stats_path.exists():
        raise FileNotFoundError(f"No stats.csv found in {exp_dir}")

    df = pd.read_csv(stats_path)

    # Parse experiment ID for metadata
    parsed = parse_experiment_id(experiment_id)

    # Load summary.txt if available
    summary_path = exp_dir / "summary.txt"
    mode = parsed["mode"]
    if summary_path.exists():
        with summary_path.open() as f:
            for line in f:
                if line.startswith("Mode:"):
                    mode = line.split(":", 1)[1].strip()
                    break

    # Load version.toml if available
    version = None
    git_hash = None
    version_path = exp_dir / "version.toml"
    if version_path.exists():
        with version_path.open() as f:
            for line in f:
                if line.startswith("version"):
                    version = line.split("=", 1)[1].strip().strip('"')
                elif line.startswith("git_hash"):
                    git_hash = line.split("=", 1)[1].strip().strip('"')

    # Extract final values
    final_row = df.iloc[-1]
    final_fitness = float(final_row["fitness"])
    final_win_rate = float(final_row["win_rate"])
    final_sigma = float(final_row["sigma"])
    total_time = float(final_row.get("cumulative_time", 0))
    total_generations = int(final_row["generation"]) + 1  # 0-indexed

    return TuningData(
        experiment_id=experiment_id,
        tag=parsed["tag"],
        mode=mode,
        timestamp=parsed["timestamp"],
        generations_df=df,
        final_fitness=final_fitness,
        final_win_rate=final_win_rate,
        final_sigma=final_sigma,
        total_time_secs=total_time,
        total_generations=total_generations,
        version=version,
        git_hash=git_hash,
    )


def load_multiple_tuning_experiments(
    base_dir: Path | None = None,
    since: datetime | None = None,
    limit: int | None = None,
) -> list[TuningData]:
    """Load multiple tuning experiments.

    Args:
        base_dir: Base directory for experiments.
        since: Only include experiments after this date.
        limit: Maximum number to return.

    Returns:
        List of TuningData objects, sorted by timestamp descending.
    """
    exp_dirs = find_all_tuning_experiments(base_dir, since, limit)
    results = []

    for exp_dir in exp_dirs:
        try:
            data = _load_from_directory(exp_dir)
            results.append(data)
        except Exception:
            # Skip experiments that fail to load
            continue

    return results
