"""Validation data loader for HTML reports.

Loads and parses validation results.json files from experiments/validation/.
"""

from __future__ import annotations

import json
from dataclasses import dataclass, field
from datetime import datetime
from pathlib import Path
from typing import Any

import pandas as pd


@dataclass
class DeckStats:
    """Statistics for a single deck."""

    deck_id: str
    commander_id: int
    commander_name: str
    faction: str
    total_games: int
    total_wins: int
    win_rate: float
    win_rate_ci_lower: float
    win_rate_ci_upper: float
    best_matchup: tuple[str, float]
    worst_matchup: tuple[str, float]

    @classmethod
    def from_dict(cls, data: dict[str, Any]) -> DeckStats:
        return cls(
            deck_id=data["deck_id"],
            commander_id=data["commander_id"],
            commander_name=data["commander_name"],
            faction=data["faction"],
            total_games=data["total_games"],
            total_wins=data["total_wins"],
            win_rate=data["win_rate"],
            win_rate_ci_lower=data["win_rate_ci_lower"],
            win_rate_ci_upper=data["win_rate_ci_upper"],
            best_matchup=tuple(data["best_matchup"]),
            worst_matchup=tuple(data["worst_matchup"]),
        )


@dataclass
class MatchupResult:
    """Results for a single matchup between two decks."""

    deck1_id: str
    deck2_id: str
    commander1_name: str
    commander2_name: str
    faction1: str
    faction2: str
    total_games: int
    faction1_win_rate: float
    faction2_win_rate: float
    avg_turns: float
    p1_win_rate: float
    p1_significance: str
    diagnostics: dict[str, Any] = field(default_factory=dict)

    @classmethod
    def from_dict(cls, data: dict[str, Any]) -> MatchupResult:
        diag = data.get("diagnostics", {})
        return cls(
            deck1_id=data["deck1_id"],
            deck2_id=data["deck2_id"],
            commander1_name=data["commander1_name"],
            commander2_name=data["commander2_name"],
            faction1=data["faction1"],
            faction2=data["faction2"],
            total_games=data["total_games"],
            faction1_win_rate=data["faction1_win_rate"],
            faction2_win_rate=data["faction2_win_rate"],
            avg_turns=data["avg_turns"],
            p1_win_rate=diag.get("p1_win_rate", 0.5),
            p1_significance=diag.get("p1_significance", "not_significant"),
            diagnostics=diag,
        )


@dataclass
class ValidationData:
    """Loaded and parsed validation data from a single run."""

    run_id: str
    timestamp: datetime
    version: str
    git_hash: str
    config: dict[str, Any]
    matchups: list[MatchupResult]
    deck_stats: list[DeckStats]
    summary: dict[str, Any]
    raw_matchups: list[dict[str, Any]] = field(default_factory=list)

    @classmethod
    def from_json(cls, path: Path) -> ValidationData:
        """Load validation data from a results.json file."""
        with path.open() as f:
            data = json.load(f)

        # Extract run_id from directory name
        run_id = path.parent.name

        # Parse timestamp
        ts_str = data["timestamp"]
        # Handle timezone offset format
        if "+" in ts_str:
            ts_str = ts_str.split("+")[0]
        timestamp = datetime.fromisoformat(ts_str)

        # Version info
        version_info = data.get("version", {})
        version = version_info.get("version", "unknown")
        git_hash = version_info.get("git_hash", "unknown")[:8]

        # Parse matchups
        matchups = [MatchupResult.from_dict(m) for m in data.get("matchups", [])]

        # Parse deck stats from summary
        summary = data.get("summary", {})
        deck_stats = [DeckStats.from_dict(d) for d in summary.get("deck_stats", [])]

        return cls(
            run_id=run_id,
            timestamp=timestamp,
            version=version,
            git_hash=git_hash,
            config=data.get("config", {}),
            matchups=matchups,
            deck_stats=deck_stats,
            summary=summary,
            raw_matchups=data.get("matchups", []),
        )

    def get_matchup_matrix(self) -> pd.DataFrame:
        """Build a deck vs deck win rate matrix."""
        # Collect all unique deck IDs
        deck_ids = sorted({m.deck1_id for m in self.matchups} | {m.deck2_id for m in self.matchups})

        # Initialize matrix with NaN
        matrix = pd.DataFrame(index=deck_ids, columns=deck_ids, dtype=float)
        matrix[:] = float("nan")

        # Fill in win rates (deck1 win rate vs deck2)
        for m in self.matchups:
            matrix.loc[m.deck1_id, m.deck2_id] = m.faction1_win_rate
            matrix.loc[m.deck2_id, m.deck1_id] = m.faction2_win_rate

        return matrix

    def get_health_score(self) -> float:
        """Calculate overall balance health score (0-100).

        Based on:
        - Deck win rates within 40-60% range (50 points)
        - P1/P2 balance (25 points)
        - Faction spread (25 points)
        """
        score = 0.0

        # Deck balance: count decks in 40-60% range
        balanced_decks = sum(1 for d in self.deck_stats if 0.4 <= d.win_rate <= 0.6)
        total_decks = len(self.deck_stats) or 1
        score += 50 * (balanced_decks / total_decks)

        # P1/P2 balance: ideal is 50%
        p1_rate = self.summary.get("p1_win_rate", 0.5)
        p1_deviation = abs(p1_rate - 0.5)
        # Score decreases as deviation increases (0.1 = 0 points, 0 = 25 points)
        p1_score = max(0, 25 * (1 - p1_deviation / 0.1))
        score += p1_score

        # Faction spread: max delta should be < 10%
        faction_delta = self.summary.get("max_faction_delta", 0)
        # Score decreases as delta increases (0.2 = 0 points, 0 = 25 points)
        faction_score = max(0, 25 * (1 - faction_delta / 0.2))
        score += faction_score

        return min(100, max(0, score))

    def get_outlier_decks(self, _threshold: float = 0.1) -> list[DeckStats]:
        """Get decks with win rates outside balanced range."""
        return [d for d in self.deck_stats if d.win_rate < 0.4 or d.win_rate > 0.6]

    @property
    def total_games(self) -> int:
        """Total games played across all matchups."""
        return sum(m.total_games for m in self.matchups)

    @property
    def p1_win_rate(self) -> float:
        """Overall P1 win rate."""
        return self.summary.get("p1_win_rate", 0.5)

    @property
    def p1_status(self) -> str:
        """P1/P2 balance status."""
        return self.summary.get("p1_status", "unknown")

    @property
    def overall_status(self) -> str:
        """Overall balance status."""
        return self.summary.get("overall_status", "unknown")


def find_latest_validation(base_dir: Path = Path("experiments/validation")) -> Path | None:
    """Find the most recent validation run directory.

    Looks for directories matching YYYY-MM-DD_HHMM pattern and returns
    the one with the most recent timestamp.
    """
    if not base_dir.exists():
        return None

    candidates = []
    for d in base_dir.iterdir():
        if not d.is_dir():
            continue
        # Check for results.json
        if not (d / "results.json").exists():
            continue
        # Try to parse timestamp from directory name
        name = d.name
        if len(name) >= 15 and name[4] == "-" and name[7] == "-" and name[10] == "_":
            try:
                ts = datetime.strptime(name[:15], "%Y-%m-%d_%H%M")
                candidates.append((ts, d))
            except ValueError:
                continue

    if not candidates:
        return None

    # Return the most recent
    candidates.sort(key=lambda x: x[0], reverse=True)
    return candidates[0][1]


def find_validation_by_id(
    run_id: str, base_dir: Path = Path("experiments/validation")
) -> Path | None:
    """Find a validation run by ID (partial or full match).

    Args:
        run_id: Run ID to search for (can be partial, e.g., "2026-01-31" or "phase9")
        base_dir: Base validation directory

    Returns:
        Path to the validation directory, or None if not found
    """
    if not base_dir.exists():
        return None

    # Handle "latest" special case
    if run_id.lower() == "latest":
        return find_latest_validation(base_dir)

    # Try exact match first
    exact = base_dir / run_id
    if exact.is_dir() and (exact / "results.json").exists():
        return exact

    # Try partial match (prefix)
    candidates = []
    for d in base_dir.iterdir():
        if not d.is_dir():
            continue
        if not (d / "results.json").exists():
            continue
        if run_id in d.name:
            candidates.append(d)

    if not candidates:
        return None

    # Return most recent if multiple matches
    if len(candidates) == 1:
        return candidates[0]

    # Sort by modification time
    candidates.sort(key=lambda x: x.stat().st_mtime, reverse=True)
    return candidates[0]


def load_validation_data(run_id: str | None = None, base_dir: Path | None = None) -> ValidationData:
    """Load validation data by run ID.

    Args:
        run_id: Run ID to load ("latest" for most recent, or specific ID)
        base_dir: Base validation directory (default: experiments/validation)

    Returns:
        Loaded ValidationData

    Raises:
        FileNotFoundError: If the run cannot be found
    """
    base_dir = base_dir or Path("experiments/validation")

    if run_id is None or run_id.lower() == "latest":
        run_dir = find_latest_validation(base_dir)
    else:
        run_dir = find_validation_by_id(run_id, base_dir)

    if run_dir is None:
        raise FileNotFoundError(f"Could not find validation run: {run_id}")

    results_path = run_dir / "results.json"
    if not results_path.exists():
        raise FileNotFoundError(f"No results.json in {run_dir}")

    return ValidationData.from_json(results_path)
