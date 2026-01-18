"""Elo rating system for agent comparison.

This module implements a standard Elo rating system for tracking
relative agent strength across many games.

The Elo system provides:
- A single number summarizing agent strength
- Fair comparison across different opponent pools
- Confidence intervals based on games played

Reference ratings (approximate):
- Random: 1000
- Greedy: 1300
- MCTS-50: 1450
- MCTS-100: 1500
- Strong RL agent: 1600+
"""

from __future__ import annotations

import json
import math
from dataclasses import dataclass
from pathlib import Path
from typing import Any


@dataclass
class AgentRating:
    """Rating information for a single agent."""

    rating: float = 1500.0
    games_played: int = 0
    wins: int = 0
    losses: int = 0
    draws: int = 0

    @property
    def win_rate(self) -> float:
        if self.games_played == 0:
            return 0.0
        return self.wins / self.games_played

    @property
    def confidence_interval(self) -> float:
        """Approximate 95% confidence interval width."""
        if self.games_played < 10:
            return 400.0  # High uncertainty
        # Standard error decreases with sqrt(n)
        return 400.0 / math.sqrt(self.games_played)


class EloTracker:
    """Track Elo ratings for multiple agents.

    The Elo system updates ratings after each game based on:
    - The expected outcome (from rating difference)
    - The actual outcome (win/loss/draw)
    - The K-factor (how much ratings can change per game)

    Example:
        tracker = EloTracker()

        # Record game results
        tracker.update("MyAgent", "Greedy", winner=0)  # MyAgent won
        tracker.update("MyAgent", "MCTS-100", winner=1)  # MCTS won
        tracker.update("MyAgent", "Random", winner=0)  # MyAgent won

        # Get ratings
        print(tracker.get_rating("MyAgent"))  # e.g., 1520
        print(tracker.leaderboard())
    """

    def __init__(
        self,
        k_factor: float = 32.0,
        initial_rating: float = 1500.0,
    ):
        """Initialize Elo tracker.

        Args:
            k_factor: Maximum rating change per game. Higher = more volatile.
                      32 is standard for new players, 16 for established.
            initial_rating: Starting rating for new agents.
        """
        self.k_factor = k_factor
        self.initial_rating = initial_rating
        self.ratings: dict[str, AgentRating] = {}

    def _ensure_agent(self, name: str) -> None:
        """Ensure agent exists in ratings dict."""
        if name not in self.ratings:
            self.ratings[name] = AgentRating(rating=self.initial_rating)

    def expected_score(self, rating_a: float, rating_b: float) -> float:
        """Calculate expected score for player A against player B.

        Returns value in [0, 1] representing expected win probability.
        0.5 means equal strength, 0.76 means A wins 76% of the time.
        """
        return 1.0 / (1.0 + 10 ** ((rating_b - rating_a) / 400.0))

    def update(
        self,
        agent1: str,
        agent2: str,
        winner: int,
    ) -> tuple[float, float]:
        """Update ratings after a game.

        Args:
            agent1: Name of first agent
            agent2: Name of second agent
            winner: 0 if agent1 won, 1 if agent2 won, -1 for draw

        Returns:
            Tuple of (agent1_new_rating, agent2_new_rating)
        """
        self._ensure_agent(agent1)
        self._ensure_agent(agent2)

        r1 = self.ratings[agent1]
        r2 = self.ratings[agent2]

        # Calculate expected scores
        e1 = self.expected_score(r1.rating, r2.rating)
        e2 = 1.0 - e1

        # Determine actual scores
        if winner == 0:
            s1, s2 = 1.0, 0.0
            r1.wins += 1
            r2.losses += 1
        elif winner == 1:
            s1, s2 = 0.0, 1.0
            r1.losses += 1
            r2.wins += 1
        else:  # Draw
            s1, s2 = 0.5, 0.5
            r1.draws += 1
            r2.draws += 1

        # Update ratings
        r1.rating += self.k_factor * (s1 - e1)
        r2.rating += self.k_factor * (s2 - e2)

        # Update game counts
        r1.games_played += 1
        r2.games_played += 1

        return r1.rating, r2.rating

    def get_rating(self, agent: str) -> float:
        """Get current rating for an agent."""
        self._ensure_agent(agent)
        return self.ratings[agent].rating

    def get_agent_stats(self, agent: str) -> AgentRating:
        """Get full stats for an agent."""
        self._ensure_agent(agent)
        return self.ratings[agent]

    def leaderboard(self) -> list[tuple[str, float, int]]:
        """Get sorted leaderboard.

        Returns:
            List of (name, rating, games_played) tuples, sorted by rating descending.
        """
        return sorted(
            [(name, r.rating, r.games_played) for name, r in self.ratings.items()],
            key=lambda x: x[1],
            reverse=True,
        )

    def to_dict(self) -> dict[str, Any]:
        """Export ratings to dictionary."""
        return {
            "k_factor": self.k_factor,
            "initial_rating": self.initial_rating,
            "ratings": {
                name: {
                    "rating": r.rating,
                    "games_played": r.games_played,
                    "wins": r.wins,
                    "losses": r.losses,
                    "draws": r.draws,
                }
                for name, r in self.ratings.items()
            },
        }

    @classmethod
    def from_dict(cls, data: dict[str, Any]) -> EloTracker:
        """Load ratings from dictionary."""
        tracker = cls(
            k_factor=data.get("k_factor", 32.0),
            initial_rating=data.get("initial_rating", 1500.0),
        )
        for name, stats in data.get("ratings", {}).items():
            tracker.ratings[name] = AgentRating(
                rating=stats["rating"],
                games_played=stats["games_played"],
                wins=stats["wins"],
                losses=stats["losses"],
                draws=stats["draws"],
            )
        return tracker

    def save(self, path: str | Path) -> None:
        """Save ratings to JSON file."""
        with open(path, "w") as f:
            json.dump(self.to_dict(), f, indent=2)

    @classmethod
    def load(cls, path: str | Path) -> EloTracker:
        """Load ratings from JSON file."""
        with open(path) as f:
            return cls.from_dict(json.load(f))

    def __repr__(self) -> str:
        agents = len(self.ratings)
        total_games = sum(r.games_played for r in self.ratings.values()) // 2
        return f"EloTracker({agents} agents, {total_games} games)"
