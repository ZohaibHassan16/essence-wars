"""Base rating classes for the unified ratings system.

Provides common interfaces and utilities shared by deck and agent ratings.
"""

from __future__ import annotations

import math
from abc import ABC, abstractmethod
from dataclasses import dataclass
from enum import Enum
from typing import Any


class RatingCategory(Enum):
    """Category of rating."""

    DECK = "deck"
    AGENT = "agent"


@dataclass
class BaseRating(ABC):
    """Base class for all rating entries.

    Provides common properties and methods shared by deck and agent ratings.
    """

    rating: float
    games: int
    wins: int
    losses: int
    draws: int

    @property
    def win_rate(self) -> float:
        """Calculate win rate from record."""
        if self.games == 0:
            return 0.5
        return self.wins / self.games

    @property
    def confidence_interval(self) -> float:
        """Approximate 95% confidence interval width.

        Based on standard error decreasing with sqrt(n).
        Returns rating uncertainty in ELO points.
        """
        if self.games < 10:
            return 400.0  # High uncertainty for few games
        return 400.0 / math.sqrt(self.games)

    @property
    @abstractmethod
    def display_name(self) -> str:
        """Human-readable name for display."""
        ...

    @property
    @abstractmethod
    def category(self) -> RatingCategory:
        """Rating category (deck or agent)."""
        ...

    @property
    @abstractmethod
    def identifier(self) -> str:
        """Unique identifier for this rating entry."""
        ...

    def to_dict(self) -> dict[str, Any]:
        """Export rating to dictionary."""
        return {
            "rating": self.rating,
            "games": self.games,
            "wins": self.wins,
            "losses": self.losses,
            "draws": self.draws,
            "win_rate": self.win_rate,
            "confidence_interval": self.confidence_interval,
            "display_name": self.display_name,
            "category": self.category.value,
            "identifier": self.identifier,
        }


def expected_score(rating_a: float, rating_b: float) -> float:
    """Calculate expected score for player A against player B.

    Uses the standard ELO formula:
        E_A = 1 / (1 + 10^((R_B - R_A) / 400))

    Args:
        rating_a: ELO rating of player A
        rating_b: ELO rating of player B

    Returns:
        Expected win probability for player A (0 to 1).
        0.5 means equal strength.
    """
    return 1.0 / (1.0 + 10 ** ((rating_b - rating_a) / 400.0))


def calculate_rating_change(
    rating: float,
    opponent_rating: float,
    score: float,
    k_factor: float = 32.0,
) -> float:
    """Calculate rating change after a game.

    Args:
        rating: Current ELO rating
        opponent_rating: Opponent's ELO rating
        score: Actual score (1.0 = win, 0.5 = draw, 0.0 = loss)
        k_factor: Maximum rating change per game

    Returns:
        Rating change (can be positive or negative).
    """
    expected = expected_score(rating, opponent_rating)
    return k_factor * (score - expected)
