#!/usr/bin/env python3
"""
Analyze validation results from results.json.

Usage:
    python scripts/analyze_validation.py experiments/validation/2026-01-16_2137/results.json
    python scripts/analyze_validation.py --latest
"""

import argparse
import json
from collections import defaultdict
from dataclasses import dataclass, field
from pathlib import Path


@dataclass
class FactionStats:
    """Aggregated stats for a faction."""
    name: str
    total_wins: int = 0
    total_games: int = 0
    total_draws: int = 0

    # Aggregated diagnostics
    face_damage_dealt: list = field(default_factory=list)
    face_damage_taken: list = field(default_factory=list)
    trade_ratios: list = field(default_factory=list)
    board_advantages: list = field(default_factory=list)
    essence_avgs: list = field(default_factory=list)
    game_lengths: list = field(default_factory=list)

    # Per-matchup data
    matchup_results: dict = field(default_factory=dict)

    @property
    def win_rate(self) -> float:
        if self.total_games == 0:
            return 0.0
        return self.total_wins / (self.total_games - self.total_draws)

    def avg(self, values: list) -> float:
        return sum(values) / len(values) if values else 0.0


@dataclass
class DeckStats:
    """Stats for a specific deck."""
    deck_id: str
    faction: str
    wins: int = 0
    games: int = 0
    draws: int = 0

    @property
    def win_rate(self) -> float:
        effective_games = self.games - self.draws
        if effective_games == 0:
            return 0.0
        return self.wins / effective_games


def load_results(path: Path) -> dict:
    """Load results.json file."""
    with open(path) as f:
        return json.load(f)


def analyze_factions(data: dict) -> dict[str, FactionStats]:
    """Aggregate stats by faction."""
    factions = {}

    for matchup in data["matchups"]:
        f1, f2 = matchup["faction1"], matchup["faction2"]
        d1, d2 = matchup["deck1_id"], matchup["deck2_id"]
        diag = matchup["diagnostics"]

        # Initialize factions if needed
        for faction in [f1, f2]:
            if faction not in factions:
                factions[faction] = FactionStats(name=faction)

        # Faction 1 stats
        factions[f1].total_wins += matchup["faction1_total_wins"]
        factions[f1].total_games += matchup["total_games"]
        factions[f1].total_draws += matchup["draws"]

        # Faction 2 stats
        factions[f2].total_wins += matchup["faction2_total_wins"]
        factions[f2].total_games += matchup["total_games"]
        factions[f2].total_draws += matchup["draws"]

        # Store matchup result for faction 1
        matchup_key = f"{f1}_vs_{f2}"
        if matchup_key not in factions[f1].matchup_results:
            factions[f1].matchup_results[matchup_key] = {"wins": 0, "games": 0, "draws": 0, "decks": []}
        factions[f1].matchup_results[matchup_key]["wins"] += matchup["faction1_total_wins"]
        factions[f1].matchup_results[matchup_key]["games"] += matchup["total_games"]
        factions[f1].matchup_results[matchup_key]["draws"] += matchup["draws"]
        factions[f1].matchup_results[matchup_key]["decks"].append({
            "deck": d1, "vs": d2,
            "win_rate": matchup["faction1_win_rate"],
            "avg_turns": matchup["avg_turns"]
        })

        # Store matchup result for faction 2 (reverse)
        matchup_key_rev = f"{f2}_vs_{f1}"
        if matchup_key_rev not in factions[f2].matchup_results:
            factions[f2].matchup_results[matchup_key_rev] = {"wins": 0, "games": 0, "draws": 0, "decks": []}
        factions[f2].matchup_results[matchup_key_rev]["wins"] += matchup["faction2_total_wins"]
        factions[f2].matchup_results[matchup_key_rev]["games"] += matchup["total_games"]
        factions[f2].matchup_results[matchup_key_rev]["draws"] += matchup["draws"]
        factions[f2].matchup_results[matchup_key_rev]["decks"].append({
            "deck": d2, "vs": d1,
            "win_rate": matchup["faction2_win_rate"],
            "avg_turns": matchup["avg_turns"]
        })

        # Aggregate diagnostics (faction1 is always P1 in first half, P2 in second half)
        # We'll use the average of both positions
        factions[f1].trade_ratios.append(diag["p1_trade_ratio"])
        factions[f2].trade_ratios.append(diag["p2_trade_ratio"])
        factions[f1].face_damage_dealt.append(diag["p1_face_damage_avg"])
        factions[f2].face_damage_dealt.append(diag["p2_face_damage_avg"])
        factions[f1].face_damage_taken.append(diag["p2_face_damage_avg"])
        factions[f2].face_damage_taken.append(diag["p1_face_damage_avg"])
        factions[f1].essence_avgs.append(diag["p1_essence_avg"])
        factions[f2].essence_avgs.append(diag["p2_essence_avg"])
        factions[f1].game_lengths.append(diag["game_length_p50"])
        factions[f2].game_lengths.append(diag["game_length_p50"])
        factions[f1].board_advantages.append(diag["avg_board_advantage"])
        factions[f2].board_advantages.append(-diag["avg_board_advantage"])

    return factions


def analyze_decks(data: dict) -> dict[str, DeckStats]:
    """Aggregate stats by deck."""
    decks = {}

    for matchup in data["matchups"]:
        d1, d2 = matchup["deck1_id"], matchup["deck2_id"]
        f1, f2 = matchup["faction1"], matchup["faction2"]

        # Initialize decks if needed
        if d1 not in decks:
            decks[d1] = DeckStats(deck_id=d1, faction=f1)
        if d2 not in decks:
            decks[d2] = DeckStats(deck_id=d2, faction=f2)

        # Deck 1 stats (as both P1 and P2)
        decks[d1].wins += matchup["faction1_total_wins"]
        decks[d1].games += matchup["total_games"]
        decks[d1].draws += matchup["draws"]

        # Deck 2 stats
        decks[d2].wins += matchup["faction2_total_wins"]
        decks[d2].games += matchup["total_games"]
        decks[d2].draws += matchup["draws"]

    return decks


def find_worst_matchups(data: dict, n: int = 10) -> list:
    """Find the n most lopsided matchups."""
    matchups = []
    for m in data["matchups"]:
        # Calculate how far from 50% the matchup is
        delta = abs(m["faction1_win_rate"] - 0.5)
        matchups.append({
            "deck1": m["deck1_id"],
            "deck2": m["deck2_id"],
            "faction1": m["faction1"],
            "faction2": m["faction2"],
            "f1_win_rate": m["faction1_win_rate"],
            "f2_win_rate": m["faction2_win_rate"],
            "delta": delta,
            "avg_turns": m["avg_turns"],
            "draws": m["draws"],
            "diag": m["diagnostics"]
        })

    return sorted(matchups, key=lambda x: -x["delta"])[:n]


def print_faction_summary(factions: dict[str, FactionStats]):
    """Print faction summary."""
    print("\n" + "=" * 70)
    print("FACTION SUMMARY")
    print("=" * 70)

    # Sort by win rate
    sorted_factions = sorted(factions.values(), key=lambda f: -f.win_rate)

    for f in sorted_factions:
        print(f"\n{f.name.upper()}")
        print("-" * 40)
        print(f"  Win Rate: {f.win_rate*100:.1f}%")
        print(f"  Games: {f.total_games} (draws: {f.total_draws})")
        print(f"  Avg Face Damage Dealt: {f.avg(f.face_damage_dealt):.2f}")
        print(f"  Avg Face Damage Taken: {f.avg(f.face_damage_taken):.2f}")
        print(f"  Avg Trade Ratio: {f.avg(f.trade_ratios):.3f}")
        print(f"  Avg Board Advantage: {f.avg(f.board_advantages):+.3f}")
        print(f"  Avg Essence at End: {f.avg(f.essence_avgs):.2f}")
        print(f"  Median Game Length: {f.avg(f.game_lengths):.1f}")


def print_matchup_matrix(factions: dict[str, FactionStats]):
    """Print faction vs faction win rate matrix."""
    print("\n" + "=" * 70)
    print("FACTION MATCHUP MATRIX (row vs column win rate)")
    print("=" * 70)

    faction_names = sorted(factions.keys())

    # Header
    print(f"{'':>12}", end="")
    for name in faction_names:
        print(f"{name[:8]:>10}", end="")
    print()

    # Rows
    for f1 in faction_names:
        print(f"{f1:>12}", end="")
        for f2 in faction_names:
            if f1 == f2:
                print(f"{'---':>10}", end="")
            else:
                key = f"{f1}_vs_{f2}"
                if key in factions[f1].matchup_results:
                    mr = factions[f1].matchup_results[key]
                    effective = mr["games"] - mr["draws"]
                    wr = mr["wins"] / effective if effective > 0 else 0
                    print(f"{wr*100:>9.1f}%", end="")
                else:
                    print(f"{'N/A':>10}", end="")
        print()


def print_deck_rankings(decks: dict[str, DeckStats]):
    """Print deck rankings by win rate."""
    print("\n" + "=" * 70)
    print("DECK RANKINGS")
    print("=" * 70)

    sorted_decks = sorted(decks.values(), key=lambda d: -d.win_rate)

    print(f"\n{'Rank':<6}{'Deck':<25}{'Faction':<12}{'Win Rate':>10}{'Games':>8}")
    print("-" * 65)

    for i, d in enumerate(sorted_decks, 1):
        print(f"{i:<6}{d.deck_id:<25}{d.faction:<12}{d.win_rate*100:>9.1f}%{d.games:>8}")


def print_worst_matchups(matchups: list):
    """Print the most imbalanced matchups."""
    print("\n" + "=" * 70)
    print("MOST IMBALANCED MATCHUPS")
    print("=" * 70)

    for i, m in enumerate(matchups, 1):
        winner = m["deck1"] if m["f1_win_rate"] > 0.5 else m["deck2"]
        loser = m["deck2"] if m["f1_win_rate"] > 0.5 else m["deck1"]
        winner_faction = m["faction1"] if m["f1_win_rate"] > 0.5 else m["faction2"]
        loser_faction = m["faction2"] if m["f1_win_rate"] > 0.5 else m["faction1"]
        win_rate = max(m["f1_win_rate"], m["f2_win_rate"])

        print(f"\n{i}. {winner} ({winner_faction}) vs {loser} ({loser_faction})")
        print(f"   Win Rate: {win_rate*100:.1f}% - {(1-win_rate)*100:.1f}%")
        print(f"   Avg Turns: {m['avg_turns']:.1f}, Draws: {m['draws']}")
        print(f"   Trade Ratio: {m['diag']['p1_trade_ratio']:.2f} vs {m['diag']['p2_trade_ratio']:.2f}")
        print(f"   Face Damage: {m['diag']['p1_face_damage_avg']:.1f} vs {m['diag']['p2_face_damage_avg']:.1f}")


def print_deck_matchup_details(data: dict, faction: str):
    """Print detailed matchup info for a specific faction's decks."""
    print("\n" + "=" * 70)
    print(f"DETAILED MATCHUPS FOR {faction.upper()}")
    print("=" * 70)

    # Group by deck
    deck_matchups = defaultdict(list)
    for m in data["matchups"]:
        if m["faction1"] == faction:
            deck_matchups[m["deck1_id"]].append({
                "vs_deck": m["deck2_id"],
                "vs_faction": m["faction2"],
                "win_rate": m["faction1_win_rate"],
                "avg_turns": m["avg_turns"]
            })
        if m["faction2"] == faction:
            deck_matchups[m["deck2_id"]].append({
                "vs_deck": m["deck1_id"],
                "vs_faction": m["faction1"],
                "win_rate": m["faction2_win_rate"],
                "avg_turns": m["avg_turns"]
            })

    for deck_id, matchups in sorted(deck_matchups.items()):
        avg_wr = sum(m["win_rate"] for m in matchups) / len(matchups)
        print(f"\n{deck_id} (avg: {avg_wr*100:.1f}%)")
        print("-" * 50)

        # Sort by win rate
        for m in sorted(matchups, key=lambda x: x["win_rate"]):
            indicator = "!!" if m["win_rate"] < 0.3 else "!" if m["win_rate"] < 0.4 else ""
            print(f"  vs {m['vs_deck']:<20} ({m['vs_faction']:<8}): {m['win_rate']*100:5.1f}% {indicator}")


def find_latest_results() -> Path | None:
    """Find the most recent validation results."""
    validation_dir = Path("experiments/validation")
    if not validation_dir.exists():
        return None

    dirs = sorted(validation_dir.iterdir(), reverse=True)
    for d in dirs:
        results_file = d / "results.json"
        if results_file.exists():
            return results_file
    return None


def main():
    parser = argparse.ArgumentParser(description="Analyze validation results")
    parser.add_argument("path", nargs="?", help="Path to results.json")
    parser.add_argument("--latest", action="store_true", help="Use latest results")
    parser.add_argument("--faction", help="Show detailed matchups for a faction")
    parser.add_argument("--top", type=int, default=10, help="Number of worst matchups to show")
    args = parser.parse_args()

    # Find results file
    if args.latest:
        path = find_latest_results()
        if not path:
            print("No validation results found in experiments/validation/")
            return
    elif args.path:
        path = Path(args.path)
    else:
        # Try latest by default
        path = find_latest_results()
        if not path:
            print("Usage: analyze_validation.py <results.json> or --latest")
            return

    print(f"Analyzing: {path}")

    # Load and analyze
    data = load_results(path)
    factions = analyze_factions(data)
    decks = analyze_decks(data)
    worst = find_worst_matchups(data, args.top)

    # Print reports
    print_faction_summary(factions)
    print_matchup_matrix(factions)
    print_deck_rankings(decks)
    print_worst_matchups(worst)

    if args.faction:
        print_deck_matchup_details(data, args.faction)
    else:
        # Show details for the weakest faction
        weakest = min(factions.values(), key=lambda f: f.win_rate)
        print_deck_matchup_details(data, weakest.name)


if __name__ == "__main__":
    main()
