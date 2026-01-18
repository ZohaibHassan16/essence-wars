"""Benchmark metrics and results dataclasses.

This module defines the data structures for benchmark results
and utility functions for computing aggregate statistics.
"""

from __future__ import annotations

import json
from dataclasses import asdict, dataclass, field
from pathlib import Path
from typing import Any


@dataclass
class MatchupResult:
    """Results from evaluating against a single opponent."""

    opponent: str
    games_played: int
    wins: int
    losses: int
    draws: int
    avg_game_length: float
    avg_decision_time_ms: float

    @property
    def win_rate(self) -> float:
        if self.games_played == 0:
            return 0.0
        return self.wins / self.games_played

    @property
    def loss_rate(self) -> float:
        if self.games_played == 0:
            return 0.0
        return self.losses / self.games_played

    def to_dict(self) -> dict[str, Any]:
        return asdict(self)


@dataclass
class DeckResult:
    """Results for a specific deck."""

    deck_id: str
    games_played: int
    wins: int
    win_rate: float


@dataclass
class BenchmarkResults:
    """Complete benchmark results for an agent.

    Contains win rates against standard baselines, Elo rating,
    and detailed per-opponent and per-deck statistics.

    Example:
        results = benchmark.evaluate(my_agent)
        print(f"Elo: {results.elo_rating}")
        print(f"vs Greedy: {results.win_rate_vs_greedy:.1%}")
        results.save("results.json")
    """

    # Agent info
    agent_name: str

    # Standard win rates
    win_rate_vs_random: float = 0.0
    win_rate_vs_greedy: float = 0.0
    win_rate_vs_mcts50: float = 0.0
    win_rate_vs_mcts100: float = 0.0

    # Elo rating
    elo_rating: float = 1500.0

    # Aggregate stats
    total_games: int = 0
    total_wins: int = 0
    avg_game_length: float = 0.0
    avg_decision_time_ms: float = 0.0

    # Detailed results
    matchup_results: list[MatchupResult] = field(default_factory=list)
    deck_results: list[DeckResult] = field(default_factory=list)

    # Metadata
    benchmark_version: str = "1.0"
    game_mode: str = "attrition"
    games_per_opponent: int = 100

    @property
    def overall_win_rate(self) -> float:
        if self.total_games == 0:
            return 0.0
        return self.total_wins / self.total_games

    def summary(self) -> str:
        """Generate human-readable summary."""
        lines = [
            f"=== Benchmark Results: {self.agent_name} ===",
            "",
            f"Elo Rating: {self.elo_rating:.0f}",
            "",
            "Win Rates:",
            f"  vs Random:   {self.win_rate_vs_random:>6.1%}",
            f"  vs Greedy:   {self.win_rate_vs_greedy:>6.1%}",
            f"  vs MCTS-50:  {self.win_rate_vs_mcts50:>6.1%}",
            f"  vs MCTS-100: {self.win_rate_vs_mcts100:>6.1%}",
            "",
            "Statistics:",
            f"  Total games:    {self.total_games}",
            f"  Avg game length: {self.avg_game_length:.1f} turns",
            f"  Avg decision:   {self.avg_decision_time_ms:.2f} ms",
        ]

        if self.deck_results:
            lines.append("")
            lines.append("Per-Deck Win Rates:")
            for deck in sorted(self.deck_results, key=lambda d: d.win_rate, reverse=True):
                lines.append(f"  {deck.deck_id:<25} {deck.win_rate:>6.1%} ({deck.games_played} games)")

        return "\n".join(lines)

    def to_dict(self) -> dict[str, Any]:
        """Convert to dictionary for JSON export."""
        return {
            "agent_name": self.agent_name,
            "elo_rating": self.elo_rating,
            "win_rates": {
                "vs_random": self.win_rate_vs_random,
                "vs_greedy": self.win_rate_vs_greedy,
                "vs_mcts50": self.win_rate_vs_mcts50,
                "vs_mcts100": self.win_rate_vs_mcts100,
            },
            "statistics": {
                "total_games": self.total_games,
                "total_wins": self.total_wins,
                "overall_win_rate": self.overall_win_rate,
                "avg_game_length": self.avg_game_length,
                "avg_decision_time_ms": self.avg_decision_time_ms,
            },
            "matchup_results": [m.to_dict() for m in self.matchup_results],
            "deck_results": [asdict(d) for d in self.deck_results],
            "metadata": {
                "benchmark_version": self.benchmark_version,
                "game_mode": self.game_mode,
                "games_per_opponent": self.games_per_opponent,
            },
        }

    def save(self, path: str | Path) -> None:
        """Save results to JSON file."""
        with open(path, "w") as f:
            json.dump(self.to_dict(), f, indent=2)

    @classmethod
    def load(cls, path: str | Path) -> BenchmarkResults:
        """Load results from JSON file."""
        with open(path) as f:
            data = json.load(f)

        results = cls(
            agent_name=data["agent_name"],
            elo_rating=data["elo_rating"],
            win_rate_vs_random=data["win_rates"]["vs_random"],
            win_rate_vs_greedy=data["win_rates"]["vs_greedy"],
            win_rate_vs_mcts50=data["win_rates"]["vs_mcts50"],
            win_rate_vs_mcts100=data["win_rates"]["vs_mcts100"],
            total_games=data["statistics"]["total_games"],
            total_wins=data["statistics"]["total_wins"],
            avg_game_length=data["statistics"]["avg_game_length"],
            avg_decision_time_ms=data["statistics"]["avg_decision_time_ms"],
            benchmark_version=data["metadata"]["benchmark_version"],
            game_mode=data["metadata"]["game_mode"],
            games_per_opponent=data["metadata"]["games_per_opponent"],
        )

        results.matchup_results = [
            MatchupResult(**m) for m in data.get("matchup_results", [])
        ]
        results.deck_results = [
            DeckResult(**d) for d in data.get("deck_results", [])
        ]

        return results

    def __repr__(self) -> str:
        return (
            f"BenchmarkResults({self.agent_name}, "
            f"elo={self.elo_rating:.0f}, "
            f"games={self.total_games})"
        )


@dataclass
class TransferResults:
    """Results from transfer learning evaluation."""

    agent_name: str
    train_mode: str
    test_mode: str
    win_rate: float
    games_played: int

    def summary(self) -> str:
        return (
            f"Transfer: {self.train_mode} -> {self.test_mode}\n"
            f"  Win rate: {self.win_rate:.1%} ({self.games_played} games)"
        )


@dataclass
class GeneralizationResults:
    """Results from cross-faction generalization evaluation."""

    agent_name: str
    train_factions: list[str]
    test_factions: list[str]
    win_rate_train: float
    win_rate_test: float
    games_played: int
    generalization_gap: float = 0.0

    def __post_init__(self) -> None:
        self.generalization_gap = self.win_rate_train - self.win_rate_test

    def summary(self) -> str:
        train = ", ".join(self.train_factions)
        test = ", ".join(self.test_factions)
        return (
            f"Generalization: [{train}] -> [{test}]\n"
            f"  Train factions: {self.win_rate_train:.1%}\n"
            f"  Test factions:  {self.win_rate_test:.1%}\n"
            f"  Gap: {self.generalization_gap:+.1%}"
        )
