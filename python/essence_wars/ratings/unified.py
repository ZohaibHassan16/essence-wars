"""Unified ratings view combining deck and agent ratings.

Provides a single interface to view and compare ratings across both
deck performance and agent performance.

Example:
    >>> from essence_wars.ratings import UnifiedRatings
    >>> ratings = UnifiedRatings.load()
    >>> for entry in ratings.get_leaderboard()[:10]:
    ...     print(f"{entry.rank}. {entry.display_name}: {entry.rating:.0f} ({entry.category})")
"""

from __future__ import annotations

from dataclasses import dataclass
from datetime import datetime
from pathlib import Path
from typing import Literal

from .agent_ratings import AgentRating, AgentRatings, agent_ratings_exist
from .base import BaseRating, RatingCategory
from .deck_ratings import DeckRating, DeckRatings, deck_ratings_exist


@dataclass
class LeaderboardEntry:
    """A single entry in the unified leaderboard.

    Combines deck and agent ratings into a single comparable format.
    """

    rank: int
    identifier: str
    display_name: str
    rating: float
    games: int
    wins: int
    losses: int
    draws: int
    win_rate: float
    confidence_interval: float
    category: RatingCategory
    # Category-specific fields
    faction: str | None = None  # For decks
    agent_type: str | None = None  # For agents

    @classmethod
    def from_rating(cls, rating: BaseRating, rank: int = 0) -> LeaderboardEntry:
        """Create a leaderboard entry from a rating object."""
        entry = cls(
            rank=rank,
            identifier=rating.identifier,
            display_name=rating.display_name,
            rating=rating.rating,
            games=rating.games,
            wins=rating.wins,
            losses=rating.losses,
            draws=rating.draws,
            win_rate=rating.win_rate,
            confidence_interval=rating.confidence_interval,
            category=rating.category,
        )

        # Add category-specific fields
        if isinstance(rating, DeckRating):
            entry.faction = rating.faction
        elif isinstance(rating, AgentRating):
            entry.agent_type = rating.agent_type

        return entry


class UnifiedRatings:
    """Combined view of deck and agent ratings.

    Provides unified leaderboards and comparison utilities across both
    rating categories.

    Example:
        >>> ratings = UnifiedRatings.load()
        >>>
        >>> # Get combined leaderboard
        >>> for entry in ratings.get_leaderboard()[:5]:
        ...     print(f"{entry.display_name}: {entry.rating:.0f}")
        >>>
        >>> # Get category-specific rankings
        >>> deck_leaders = ratings.deck_ratings.get_ranked()[:3]
        >>> agent_leaders = ratings.agent_ratings.get_ranked()[:3]
        >>>
        >>> # Compare specific entries
        >>> deck_rating = ratings.deck_ratings.get("artificer_tokens")
        >>> agent_rating = ratings.agent_ratings.get("ppo_v1")
    """

    def __init__(
        self,
        deck_ratings: DeckRatings | None = None,
        agent_ratings: AgentRatings | None = None,
    ):
        """Initialize unified ratings.

        Args:
            deck_ratings: Deck ratings instance (or None for empty)
            agent_ratings: Agent ratings instance (or None for empty)
        """
        self.deck_ratings = deck_ratings or DeckRatings.empty()
        self.agent_ratings = agent_ratings or AgentRatings.empty()

    @classmethod
    def load(
        cls,
        deck_file: Path | str | None = None,
        agent_file: Path | str | None = None,
        decks_dir: Path | str | None = None,
    ) -> UnifiedRatings:
        """Load unified ratings from both sources.

        Missing files are handled gracefully - empty ratings are used.

        Args:
            deck_file: Path to deck_elo.json
            agent_file: Path to agent_elo.json
            decks_dir: Path to decks directory for faction lookup

        Returns:
            UnifiedRatings instance with available data.
        """
        # Load deck ratings if available
        deck_ratings = None
        if deck_ratings_exist(deck_file):
            try:
                deck_ratings = DeckRatings.load(deck_file, decks_dir)
            except Exception:
                pass

        # Load agent ratings if available
        agent_ratings = None
        if agent_ratings_exist(agent_file):
            try:
                agent_ratings = AgentRatings.load(agent_file)
            except Exception:
                pass

        return cls(deck_ratings=deck_ratings, agent_ratings=agent_ratings)

    def get_leaderboard(
        self,
        category: Literal["all", "deck", "agent"] = "all",
        limit: int | None = None,
    ) -> list[LeaderboardEntry]:
        """Get unified leaderboard sorted by rating.

        Args:
            category: Filter by category ("all", "deck", or "agent")
            limit: Maximum number of entries to return

        Returns:
            List of LeaderboardEntry sorted by rating descending.
        """
        entries: list[LeaderboardEntry] = []

        # Collect deck ratings
        if category in ("all", "deck"):
            for rating in self.deck_ratings.get_ranked():
                entries.append(LeaderboardEntry.from_rating(rating))

        # Collect agent ratings
        if category in ("all", "agent"):
            for rating in self.agent_ratings.get_ranked():
                entries.append(LeaderboardEntry.from_rating(rating))

        # Sort by rating
        entries.sort(key=lambda e: e.rating, reverse=True)

        # Assign ranks
        for i, entry in enumerate(entries, 1):
            entry.rank = i

        # Apply limit
        if limit:
            entries = entries[:limit]

        return entries

    def get_by_rating_range(
        self,
        min_rating: float = 0,
        max_rating: float = float("inf"),
    ) -> list[LeaderboardEntry]:
        """Get entries within a rating range.

        Args:
            min_rating: Minimum rating (inclusive)
            max_rating: Maximum rating (inclusive)

        Returns:
            List of entries in the rating range, sorted by rating.
        """
        entries = self.get_leaderboard()
        return [e for e in entries if min_rating <= e.rating <= max_rating]

    def get_summary(self) -> dict:
        """Get summary statistics for all ratings.

        Returns:
            Dictionary with summary statistics.
        """
        deck_ranked = self.deck_ratings.get_ranked()
        agent_ranked = self.agent_ratings.get_ranked()

        all_ratings = [d.rating for d in deck_ranked] + [a.rating for a in agent_ranked]

        summary = {
            "total_decks": len(deck_ranked),
            "total_agents": len(agent_ranked),
            "total_entries": len(all_ratings),
        }

        if deck_ranked:
            summary["deck_top"] = deck_ranked[0].display_name if deck_ranked else None
            summary["deck_top_rating"] = deck_ranked[0].rating if deck_ranked else None
            summary["deck_avg_rating"] = sum(d.rating for d in deck_ranked) / len(deck_ranked)

        if agent_ranked:
            summary["agent_top"] = agent_ranked[0].display_name if agent_ranked else None
            summary["agent_top_rating"] = agent_ranked[0].rating if agent_ranked else None
            summary["agent_avg_rating"] = sum(a.rating for a in agent_ranked) / len(agent_ranked)

        if all_ratings:
            summary["overall_avg_rating"] = sum(all_ratings) / len(all_ratings)
            summary["overall_min_rating"] = min(all_ratings)
            summary["overall_max_rating"] = max(all_ratings)

        return summary

    def format_leaderboard(
        self,
        category: Literal["all", "deck", "agent"] = "all",
        limit: int = 10,
    ) -> str:
        """Format leaderboard as a human-readable string.

        Args:
            category: Filter by category
            limit: Maximum entries to show

        Returns:
            Formatted leaderboard string.
        """
        entries = self.get_leaderboard(category=category, limit=limit)

        if not entries:
            return "No ratings available."

        lines = ["Rank  Rating  Games  Win%   Category  Name"]
        lines.append("-" * 60)

        for entry in entries:
            cat_str = entry.category.value[:5].upper()
            lines.append(
                f"{entry.rank:4d}  {entry.rating:6.0f}  {entry.games:5d}  "
                f"{entry.win_rate * 100:4.1f}%  {cat_str:8s}  {entry.display_name}"
            )

        return "\n".join(lines)

    @property
    def last_updated(self) -> datetime:
        """Get the most recent update timestamp."""
        timestamps = []
        if self.deck_ratings.ratings:
            timestamps.append(self.deck_ratings.last_updated)
        if self.agent_ratings.ratings:
            timestamps.append(self.agent_ratings.last_updated)

        if timestamps:
            return max(timestamps)
        return datetime.now()

    def __repr__(self) -> str:
        return (
            f"UnifiedRatings("
            f"{len(self.deck_ratings)} decks, "
            f"{len(self.agent_ratings)} agents)"
        )
