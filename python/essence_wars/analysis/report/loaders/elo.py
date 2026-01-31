"""ELO ratings data loader for HTML report generation."""

from __future__ import annotations

import json
from dataclasses import dataclass, field
from datetime import datetime
from pathlib import Path
from typing import Any


@dataclass
class RatingChange:
    """A single rating change event."""

    date: str
    opponent: str
    delta: int
    result: str  # "win", "loss", "draw"
    games: int


@dataclass
class DeckRating:
    """ELO rating data for a single deck."""

    deck_id: str
    rating: float
    games: int
    wins: int
    losses: int
    draws: int
    history: list[RatingChange] = field(default_factory=list)
    faction: str | None = None
    commander_name: str | None = None

    @property
    def win_rate(self) -> float:
        """Calculate win rate from record."""
        if self.games == 0:
            return 0.5
        return self.wins / self.games

    def get_rating_timeline(self) -> list[tuple[str, float]]:
        """Build rating timeline from history.

        Returns list of (date, rating) tuples showing rating progression.
        """
        timeline = []
        current_rating = 1500.0  # Starting rating

        for change in self.history:
            current_rating += change.delta
            timeline.append((change.date, current_rating))

        return timeline


@dataclass
class EloData:
    """Complete ELO ratings data."""

    version: int
    last_updated: datetime
    ratings: dict[str, DeckRating]

    def get_ranked_decks(self) -> list[DeckRating]:
        """Get all decks sorted by rating (descending)."""
        return sorted(self.ratings.values(), key=lambda d: d.rating, reverse=True)

    def get_decks_by_faction(self, faction: str) -> list[DeckRating]:
        """Get decks for a specific faction, sorted by rating."""
        return sorted(
            [d for d in self.ratings.values() if d.faction == faction],
            key=lambda d: d.rating,
            reverse=True,
        )

    def calculate_expected_win_rate(self, deck1_id: str, deck2_id: str) -> float:
        """Calculate expected win rate for deck1 vs deck2 using ELO formula.

        Returns expected probability that deck1 wins.
        """
        r1 = self.ratings.get(deck1_id)
        r2 = self.ratings.get(deck2_id)

        if not r1 or not r2:
            return 0.5

        # ELO expected score formula
        return 1.0 / (1.0 + 10 ** ((r2.rating - r1.rating) / 400))

    def get_matchup_predictions(self) -> dict[str, dict[str, float]]:
        """Build a matrix of expected win rates for all matchups."""
        deck_ids = list(self.ratings.keys())
        matrix = {}

        for d1 in deck_ids:
            matrix[d1] = {}
            for d2 in deck_ids:
                if d1 == d2:
                    matrix[d1][d2] = 0.5
                else:
                    matrix[d1][d2] = self.calculate_expected_win_rate(d1, d2)

        return matrix

    def get_faction_standings(self) -> dict[str, dict[str, Any]]:
        """Calculate aggregate standings per faction."""
        faction_data: dict[str, dict[str, Any]] = {}

        for deck in self.ratings.values():
            faction = deck.faction or "unknown"
            if faction not in faction_data:
                faction_data[faction] = {
                    "total_rating": 0,
                    "count": 0,
                    "total_wins": 0,
                    "total_losses": 0,
                    "total_games": 0,
                }

            faction_data[faction]["total_rating"] += deck.rating
            faction_data[faction]["count"] += 1
            faction_data[faction]["total_wins"] += deck.wins
            faction_data[faction]["total_losses"] += deck.losses
            faction_data[faction]["total_games"] += deck.games

        # Calculate averages
        for faction, data in faction_data.items():
            if data["count"] > 0:
                data["avg_rating"] = data["total_rating"] / data["count"]
            else:
                data["avg_rating"] = 1500.0

            if data["total_games"] > 0:
                data["win_rate"] = data["total_wins"] / data["total_games"]
            else:
                data["win_rate"] = 0.5

        return faction_data


def load_elo_data(
    elo_file: Path | str | None = None,
    decks_dir: Path | str | None = None,
) -> EloData:
    """Load ELO ratings data.

    Args:
        elo_file: Path to deck_elo.json file. Defaults to data/ratings/deck_elo.json.
        decks_dir: Path to decks directory for faction/commander lookup.
                   Defaults to data/decks.

    Returns:
        EloData object with parsed ratings.

    Raises:
        FileNotFoundError: If ELO file not found.
    """
    if elo_file is None:
        elo_file = Path("data/ratings/deck_elo.json")
    else:
        elo_file = Path(elo_file)

    if decks_dir is None:
        decks_dir = Path("data/decks")
    else:
        decks_dir = Path(decks_dir)

    if not elo_file.exists():
        raise FileNotFoundError(f"ELO ratings file not found: {elo_file}")

    with open(elo_file) as f:
        data = json.load(f)

    # Build deck-to-faction mapping from directory structure
    deck_factions = _build_deck_faction_map(decks_dir)

    # Parse last_updated
    last_updated_str = data.get("last_updated", "")
    try:
        # Handle ISO format with timezone
        last_updated = datetime.fromisoformat(last_updated_str.replace("Z", "+00:00"))
    except (ValueError, AttributeError):
        last_updated = datetime.now()

    # Parse ratings
    ratings = {}
    for deck_id, rating_data in data.get("ratings", {}).items():
        history = [
            RatingChange(
                date=h.get("date", ""),
                opponent=h.get("opponent", ""),
                delta=h.get("delta", 0),
                result=h.get("result", ""),
                games=h.get("games", 0),
            )
            for h in rating_data.get("history", [])
        ]

        # Derive commander name from deck_id (convert snake_case to Title Case)
        commander_name = _deck_id_to_commander_name(deck_id)

        ratings[deck_id] = DeckRating(
            deck_id=deck_id,
            rating=rating_data.get("rating", 1500.0),
            games=rating_data.get("games", 0),
            wins=rating_data.get("wins", 0),
            losses=rating_data.get("losses", 0),
            draws=rating_data.get("draws", 0),
            history=history,
            faction=deck_factions.get(deck_id),
            commander_name=commander_name,
        )

    return EloData(
        version=data.get("version", 1),
        last_updated=last_updated,
        ratings=ratings,
    )


def _build_deck_faction_map(decks_dir: Path) -> dict[str, str]:
    """Build a mapping of deck_id to faction from directory structure."""
    mapping = {}

    if not decks_dir.exists():
        return mapping

    for faction_dir in decks_dir.iterdir():
        if not faction_dir.is_dir():
            continue

        faction = faction_dir.name
        for deck_file in faction_dir.glob("*.toml"):
            deck_id = deck_file.stem
            mapping[deck_id] = faction

    return mapping


def _deck_id_to_commander_name(deck_id: str) -> str:
    """Convert deck_id to human-readable commander name.

    Examples:
        artificer_tokens -> Artificer Tokens
        broodmother_pack -> Broodmother Pack
        vex_piercing -> Vex Piercing
    """
    return " ".join(word.title() for word in deck_id.split("_"))


def elo_file_exists(elo_file: Path | str | None = None) -> bool:
    """Check if ELO ratings file exists."""
    if elo_file is None:
        elo_file = Path("data/ratings/deck_elo.json")
    else:
        elo_file = Path(elo_file)
    return elo_file.exists()
