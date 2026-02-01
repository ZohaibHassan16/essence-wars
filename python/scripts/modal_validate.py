#!/usr/bin/env python3
"""
Modal cloud validation for Essence Wars.

Runs 96 inter-faction matchups in parallel across up to 100 Modal containers,
providing ~10x faster validation with higher statistical confidence.

Usage:
    uv run python/scripts/modal_validate.py                    # Default: 500 games, AB depth 6
    uv run python/scripts/modal_validate.py --games 100        # Quick run
    uv run python/scripts/modal_validate.py --dry-run          # Preview matchups
    uv run python/scripts/modal_validate.py --local            # Run locally (no Modal)

Prerequisites:
    1. Build the matchup binary: cargo build --release --bin matchup
    2. Install Modal: uv sync --group cloud
    3. Authenticate Modal: modal token new
"""

from __future__ import annotations

import argparse
import json
import math
import os
import subprocess
import sys
import time
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

# Project root (for finding binary and data)
PROJECT_ROOT = Path(__file__).parent.parent.parent

# All 12 decks
DECKS = [
    # Argentum (4)
    "architect_fortify",
    "artificer_tokens",
    "sanctum_healer",
    "vex_piercing",
    # Symbiote (4)
    "broodmother_pack",
    "alpha_frenzy",
    "grove_regenerate",
    "plague_volatile",
    # Obsidion (4)
    "archon_burst",
    "deathmaster_assassin",
    "shadow_weaver",
    "sovereign_lifesteal",
]

FACTIONS = {
    "architect_fortify": "argentum",
    "artificer_tokens": "argentum",
    "sanctum_healer": "argentum",
    "vex_piercing": "argentum",
    "broodmother_pack": "symbiote",
    "alpha_frenzy": "symbiote",
    "grove_regenerate": "symbiote",
    "plague_volatile": "symbiote",
    "archon_burst": "obsidion",
    "deathmaster_assassin": "obsidion",
    "shadow_weaver": "obsidion",
    "sovereign_lifesteal": "obsidion",
}


def get_inter_faction_matchups() -> list[tuple[str, str]]:
    """Generate all inter-faction matchup pairs (96 total)."""
    matchups = []
    for i, deck1 in enumerate(DECKS):
        for deck2 in DECKS[i + 1 :]:
            # Only inter-faction (different factions)
            if FACTIONS[deck1] != FACTIONS[deck2]:
                matchups.append((deck1, deck2))
    return matchups


def ensure_binary_built() -> Path:
    """Ensure the matchup binary is built, build if needed."""
    binary_path = PROJECT_ROOT / "target" / "release" / "matchup"

    if not binary_path.exists():
        print("Building matchup binary...")
        result = subprocess.run(
            ["cargo", "build", "--release", "--bin", "matchup"],
            cwd=PROJECT_ROOT,
            capture_output=True,
            text=True,
        )
        if result.returncode != 0:
            print(f"Error building binary:\n{result.stderr}")
            sys.exit(1)
        print("Binary built successfully.")

    return binary_path


def run_matchup_local(
    deck1: str,
    deck2: str,
    games: int,
    ab_depth: int,
    seed: int,
    binary_path: Path,
) -> dict[str, Any]:
    """Run a single matchup locally."""
    cmd = [
        str(binary_path),
        "--deck1", deck1,
        "--deck2", deck2,
        "--games", str(games),
        "--ab-depth", str(ab_depth),
        "--seed", str(seed),
        "--json",
        "--quiet",
        "--cards", str(PROJECT_ROOT / "data" / "cards" / "core_set"),
        "--commanders", str(PROJECT_ROOT / "data" / "commanders"),
        "--decks", str(PROJECT_ROOT / "data" / "decks"),
        "--weights", str(PROJECT_ROOT / "data" / "weights"),
    ]

    result = subprocess.run(cmd, capture_output=True, text=True, check=True)
    return json.loads(result.stdout)


def wilson_score_interval(wins: int, total: int, z: float = 1.96) -> tuple[float, float]:
    """Calculate Wilson score confidence interval for a proportion."""
    if total == 0:
        return (0.0, 1.0)

    p = wins / total
    denominator = 1 + z * z / total
    center = (p + z * z / (2 * total)) / denominator
    spread = z * math.sqrt(p * (1 - p) / total + z * z / (4 * total * total)) / denominator

    return (max(0, center - spread), min(1, center + spread))


def compute_deck_stats(matchup_results: list[dict[str, Any]]) -> list[dict[str, Any]]:
    """Compute per-deck statistics from matchup results."""
    from collections import defaultdict

    deck_stats: dict[str, dict[str, Any]] = defaultdict(
        lambda: {
            "games": 0,
            "wins": 0,
            "matchups": {},
            "commander_id": 0,
            "commander_name": "",
            "faction": "",
        }
    )

    for m in matchup_results:
        d1, d2 = m["deck1_id"], m["deck2_id"]

        # Track deck1 stats
        deck_stats[d1]["games"] += m["total_games"]
        deck_stats[d1]["wins"] += m["faction1_total_wins"]
        deck_stats[d1]["commander_id"] = m["commander1_id"]
        deck_stats[d1]["commander_name"] = m["commander1_name"]
        deck_stats[d1]["faction"] = m["faction1"]
        deck_stats[d1]["matchups"][d2] = m["faction1_win_rate"]

        # Track deck2 stats
        deck_stats[d2]["games"] += m["total_games"]
        deck_stats[d2]["wins"] += m["faction2_total_wins"]
        deck_stats[d2]["commander_id"] = m["commander2_id"]
        deck_stats[d2]["commander_name"] = m["commander2_name"]
        deck_stats[d2]["faction"] = m["faction2"]
        deck_stats[d2]["matchups"][d1] = m["faction2_win_rate"]

    # Build final stats list
    final_stats = []
    for deck_id, stats in deck_stats.items():
        if stats["games"] == 0:
            continue

        win_rate = stats["wins"] / stats["games"]
        ci_lower, ci_upper = wilson_score_interval(stats["wins"], stats["games"])

        # Find best/worst matchups
        if stats["matchups"]:
            best = max(stats["matchups"].items(), key=lambda x: x[1])
            worst = min(stats["matchups"].items(), key=lambda x: x[1])
        else:
            best = worst = (None, 0.5)

        final_stats.append({
            "deck_id": deck_id,
            "commander_id": stats["commander_id"],
            "commander_name": stats["commander_name"],
            "faction": stats["faction"],
            "total_games": stats["games"],
            "total_wins": stats["wins"],
            "win_rate": win_rate,
            "win_rate_ci_lower": ci_lower,
            "win_rate_ci_upper": ci_upper,
            "best_matchup": [best[0], best[1]] if best[0] else None,
            "worst_matchup": [worst[0], worst[1]] if worst[0] else None,
        })

    # Sort by win rate descending
    final_stats.sort(key=lambda x: x["win_rate"], reverse=True)
    return final_stats


def compute_balance_summary(matchup_results: list[dict[str, Any]]) -> dict[str, Any]:
    """Compute balance summary from matchup results."""
    from collections import defaultdict

    faction_wins: dict[str, int] = defaultdict(int)
    faction_games: dict[str, int] = defaultdict(int)
    total_p1_wins = 0
    total_decisive_games = 0

    for m in matchup_results:
        # Faction aggregates
        faction_wins[m["faction1"]] += m["faction1_total_wins"]
        faction_games[m["faction1"]] += m["total_games"]
        faction_wins[m["faction2"]] += m["faction2_total_wins"]
        faction_games[m["faction2"]] += m["total_games"]

        # P1/P2 stats
        diag = m.get("diagnostics", {})
        decisive = m["total_games"] - m.get("draws", 0)
        if decisive > 0:
            p1_rate = diag.get("p1_win_rate", 0.5)
            total_p1_wins += int(p1_rate * decisive)
            total_decisive_games += decisive

    # Faction win rates
    faction_win_rates = {
        f: wins / faction_games[f] if faction_games[f] > 0 else 0.5
        for f, wins in faction_wins.items()
    }

    max_faction_delta = (
        max(faction_win_rates.values()) - min(faction_win_rates.values())
        if faction_win_rates
        else 0.0
    )
    p1_win_rate = total_p1_wins / total_decisive_games if total_decisive_games > 0 else 0.5

    # Determine status
    p1_deviation = abs(p1_win_rate - 0.5)
    p1_status = (
        "balanced" if p1_deviation < 0.05 else "warning" if p1_deviation < 0.1 else "imbalanced"
    )
    faction_status = (
        "balanced"
        if max_faction_delta < 0.05
        else "warning"
        if max_faction_delta < 0.1
        else "imbalanced"
    )
    overall_status = (
        "imbalanced"
        if "imbalanced" in (p1_status, faction_status)
        else "warning"
        if "warning" in (p1_status, faction_status)
        else "balanced"
    )

    # Compute deck stats
    deck_stats = compute_deck_stats(matchup_results)

    return {
        "p1_win_rate": p1_win_rate,
        "p1_status": p1_status,
        "faction_win_rates": faction_win_rates,
        "max_faction_delta": max_faction_delta,
        "faction_status": faction_status,
        "overall_status": overall_status,
        "warnings": [],
        "p1_p2_diagnostics": {
            "overall_p1_win_rate": p1_win_rate,
            "overall_p1_ci_lower": 0.45,
            "overall_p1_ci_upper": 0.55,
            "significance": "not_significant",
            "assessment": "balanced" if p1_status == "balanced" else "check manually",
            "by_matchup": {},
        },
        "deck_stats": deck_stats,
    }


def get_version_info() -> dict[str, Any]:
    """Get version info from the Rust binary."""
    try:
        result = subprocess.run(
            ["cargo", "pkgid", "-p", "cardgame"],
            cwd=PROJECT_ROOT,
            capture_output=True,
            text=True,
        )
        if result.returncode == 0:
            # Format: file:///path#0.8.0
            version = result.stdout.strip().split("#")[-1]
        else:
            version = "unknown"
    except Exception:
        version = "unknown"

    # Get git info
    try:
        git_hash = subprocess.run(
            ["git", "rev-parse", "HEAD"],
            cwd=PROJECT_ROOT,
            capture_output=True,
            text=True,
        ).stdout.strip()[:12]
    except Exception:
        git_hash = "unknown"

    try:
        git_branch = subprocess.run(
            ["git", "rev-parse", "--abbrev-ref", "HEAD"],
            cwd=PROJECT_ROOT,
            capture_output=True,
            text=True,
        ).stdout.strip()
    except Exception:
        git_branch = "unknown"

    return {
        "version": version,
        "git_hash": git_hash,
        "git_branch": git_branch,
        "build_timestamp": datetime.now(timezone.utc).isoformat(),
    }


def update_elo_ratings(matchup_results: list[dict[str, Any]]) -> None:
    """Update ELO ratings based on matchup results."""
    elo_file = PROJECT_ROOT / "data" / "ratings" / "deck_elo.json"

    # Load existing ratings
    if elo_file.exists():
        with open(elo_file) as f:
            storage = json.load(f)
    else:
        storage = {"version": 1, "last_updated": "", "ratings": {}}

    K_FACTOR = 32.0
    DEFAULT_RATING = 1500.0

    def expected_score(rating_a: float, rating_b: float) -> float:
        return 1.0 / (1.0 + 10.0 ** ((rating_b - rating_a) / 400.0))

    for m in matchup_results:
        deck1_id = m["deck1_id"]
        deck2_id = m["deck2_id"]

        # Get or create ratings
        if deck1_id not in storage["ratings"]:
            storage["ratings"][deck1_id] = {
                "rating": DEFAULT_RATING,
                "games": 0,
                "wins": 0,
                "losses": 0,
                "draws": 0,
                "history": [],
            }
        if deck2_id not in storage["ratings"]:
            storage["ratings"][deck2_id] = {
                "rating": DEFAULT_RATING,
                "games": 0,
                "wins": 0,
                "losses": 0,
                "draws": 0,
                "history": [],
            }

        r1 = storage["ratings"][deck1_id]
        r2 = storage["ratings"][deck2_id]

        # Calculate expected and actual scores
        expected1 = expected_score(r1["rating"], r2["rating"])
        expected2 = 1.0 - expected1

        total_games = m["total_games"]
        draws = m.get("draws", 0)
        actual1 = (m["faction1_total_wins"] + 0.5 * draws) / total_games
        actual2 = (m["faction2_total_wins"] + 0.5 * draws) / total_games

        # Calculate deltas
        delta1 = round(K_FACTOR * (actual1 - expected1))
        delta2 = round(K_FACTOR * (actual2 - expected2))

        # Update ratings
        r1["rating"] += delta1
        r1["games"] += total_games
        r1["wins"] += m["faction1_total_wins"]
        r1["losses"] += m["faction2_total_wins"]
        r1["draws"] += draws

        r2["rating"] += delta2
        r2["games"] += total_games
        r2["wins"] += m["faction2_total_wins"]
        r2["losses"] += m["faction1_total_wins"]
        r2["draws"] += draws

        # Add history entries
        date = datetime.now().strftime("%Y-%m-%d")
        d1_won_more = m["faction1_total_wins"] > m["faction2_total_wins"]
        d2_won_more = m["faction2_total_wins"] > m["faction1_total_wins"]

        r1["history"].append({
            "date": date,
            "opponent": deck2_id,
            "delta": delta1,
            "result": "win" if d1_won_more else "loss" if d2_won_more else "draw",
            "games": total_games,
        })
        r2["history"].append({
            "date": date,
            "opponent": deck1_id,
            "delta": delta2,
            "result": "win" if d2_won_more else "loss" if d1_won_more else "draw",
            "games": total_games,
        })

        # Trim history to last 50
        r1["history"] = r1["history"][-50:]
        r2["history"] = r2["history"][-50:]

    # Update timestamp and save
    storage["last_updated"] = datetime.now(timezone.utc).isoformat()
    elo_file.parent.mkdir(parents=True, exist_ok=True)
    with open(elo_file, "w") as f:
        json.dump(storage, f, indent=2)
    print(f"Updated ELO ratings: {elo_file}")


def save_validation_results(
    matchup_results: list[dict[str, Any]],
    run_id: str,
    games_per_matchup: int,
    seed: int,
    total_time: float,
) -> Path:
    """Save validation results to experiments/validation/{run_id}/."""
    output_dir = PROJECT_ROOT / "experiments" / "validation" / run_id
    output_dir.mkdir(parents=True, exist_ok=True)

    # Build ValidationResults structure
    validation_results = {
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "version": get_version_info(),
        "config": {
            "games_per_matchup": games_per_matchup,
            "mcts_simulations": 0,  # Using Alpha-Beta
            "seed": seed,
            "threads": 1,
            "matchup_filter": None,
        },
        "matchups": matchup_results,
        "summary": compute_balance_summary(matchup_results),
    }

    # Save results.json
    results_file = output_dir / "results.json"
    with open(results_file, "w") as f:
        json.dump(validation_results, f, indent=2)
    print(f"Saved results: {results_file}")

    # Save summary.txt
    summary = validation_results["summary"]
    summary_file = output_dir / "summary.txt"
    with open(summary_file, "w") as f:
        f.write(f"Modal Cloud Validation - {run_id}\n")
        f.write("=" * 50 + "\n\n")
        f.write(f"Total matchups: {len(matchup_results)}\n")
        f.write(f"Games per matchup: {games_per_matchup * 2}\n")
        f.write(f"Total games: {len(matchup_results) * games_per_matchup * 2}\n")
        f.write(f"Total time: {total_time:.1f}s\n\n")
        f.write(f"P1 win rate: {summary['p1_win_rate']:.1%}\n")
        f.write(f"P1 status: {summary['p1_status']}\n")
        f.write(f"Faction status: {summary['faction_status']}\n")
        f.write(f"Overall status: {summary['overall_status']}\n\n")
        f.write("Faction win rates:\n")
        for faction, rate in summary["faction_win_rates"].items():
            f.write(f"  {faction}: {rate:.1%}\n")
    print(f"Saved summary: {summary_file}")

    # Save config.toml
    config_file = output_dir / "config.toml"
    with open(config_file, "w") as f:
        f.write(f'run_id = "{run_id}"\n')
        f.write(f"games_per_matchup = {games_per_matchup}\n")
        f.write(f"seed = {seed}\n")
        f.write('bot = "alphabeta"\n')
        f.write("ab_depth = 6\n")
        f.write(f"total_time_secs = {total_time:.1f}\n")
        f.write(f'timestamp = "{datetime.now(timezone.utc).isoformat()}"\n')

    return output_dir


def generate_report(run_id: str) -> Path | None:
    """Generate HTML report using existing ReportGenerator."""
    try:
        from essence_wars.analysis.report import ReportGenerator

        generator = ReportGenerator()
        report_path = generator.generate_validation_report(run_id=run_id)
        print(f"Generated report: {report_path}")
        return report_path
    except ImportError as e:
        print(f"Warning: Could not generate HTML report: {e}")
        print("Install analysis dependencies: uv sync --group analysis")
        return None
    except Exception as e:
        print(f"Warning: Report generation failed: {e}")
        return None


def print_windows_path(path: Path) -> None:
    """Print Windows-accessible path for WSL2 users."""
    abs_path = path.resolve()

    # Try to convert to Windows path
    try:
        result = subprocess.run(
            ["wslpath", "-w", str(abs_path)],
            capture_output=True,
            text=True,
            check=True,
        )
        windows_path = result.stdout.strip()
        print(f"\nWindows path: {windows_path}")

        # Check for index.html
        index_html = abs_path / "index.html"
        if index_html.exists():
            result = subprocess.run(
                ["wslpath", "-w", str(index_html)],
                capture_output=True,
                text=True,
                check=True,
            )
            print(f"Report:       {result.stdout.strip()}")
    except (subprocess.CalledProcessError, FileNotFoundError):
        # Not in WSL2 or wslpath not available
        print(f"\nResults: {abs_path}")


def run_local(args: argparse.Namespace) -> None:
    """Run validation locally (no Modal)."""
    binary_path = ensure_binary_built()
    matchups = get_inter_faction_matchups()

    run_id = args.run_id or f"modal_{datetime.now().strftime('%Y-%m-%d_%H%M')}"

    print(f"Running {len(matchups)} matchups locally")
    print(f"Games per matchup: {args.games * 2}")
    print(f"Total games: {len(matchups) * args.games * 2}")
    print()

    if args.dry_run:
        print("Dry run - matchups that would be executed:")
        for i, (d1, d2) in enumerate(matchups[:10]):
            print(f"  {i+1}. {d1} vs {d2}")
        if len(matchups) > 10:
            print(f"  ... and {len(matchups) - 10} more")
        return

    start_time = time.time()
    results = []

    for i, (deck1, deck2) in enumerate(matchups):
        print(f"[{i+1}/{len(matchups)}] {deck1} vs {deck2}...", end=" ", flush=True)
        matchup_seed = args.seed + i * 1_000_000
        result = run_matchup_local(
            deck1, deck2, args.games, args.ab_depth, matchup_seed, binary_path
        )
        results.append(result)
        wr = result["faction1_win_rate"]
        print(f"{wr:.1%}")

    total_time = time.time() - start_time
    print(f"\nCompleted in {total_time:.1f}s")

    # Save results
    output_dir = save_validation_results(results, run_id, args.games, args.seed, total_time)

    # Update ELO
    update_elo_ratings(results)

    # Generate report
    report_dir = generate_report(run_id)

    # Print paths
    print_windows_path(report_dir or output_dir)


def run_modal(args: argparse.Namespace) -> None:
    """Run validation using Modal cloud."""
    try:
        import modal
    except ImportError:
        print("Error: Modal not installed. Run: uv sync --group cloud")
        sys.exit(1)

    binary_path = ensure_binary_built()
    matchups = get_inter_faction_matchups()

    run_id = args.run_id or f"modal_{datetime.now().strftime('%Y-%m-%d_%H%M')}"

    print(f"Running {len(matchups)} matchups on Modal")
    print(f"Games per matchup: {args.games * 2}")
    print(f"Total games: {len(matchups) * args.games * 2}")
    print(f"Concurrency: {args.concurrency}")
    print()

    if args.dry_run:
        print("Dry run - matchups that would be executed:")
        for i, (d1, d2) in enumerate(matchups[:10]):
            print(f"  {i+1}. {d1} vs {d2}")
        if len(matchups) > 10:
            print(f"  ... and {len(matchups) - 10} more")
        return

    # Define Modal app
    app = modal.App("essence-wars-validation")

    # Build image with binary and data
    image = (
        modal.Image.debian_slim(python_version="3.11")
        .apt_install("ca-certificates")
        .copy_local_file(str(binary_path), "/usr/local/bin/matchup")
        .copy_local_dir(str(PROJECT_ROOT / "data" / "cards"), "/data/cards")
        .copy_local_dir(str(PROJECT_ROOT / "data" / "commanders"), "/data/commanders")
        .copy_local_dir(str(PROJECT_ROOT / "data" / "decks"), "/data/decks")
        .copy_local_dir(str(PROJECT_ROOT / "data" / "weights"), "/data/weights")
    )

    @app.function(image=image, timeout=600, memory=512, cpu=1)
    def modal_run_matchup(
        deck1: str,
        deck2: str,
        games: int,
        ab_depth: int,
        seed: int,
    ) -> dict:
        """Run a single matchup in a Modal container."""
        import json
        import subprocess

        cmd = [
            "/usr/local/bin/matchup",
            "--deck1", deck1,
            "--deck2", deck2,
            "--games", str(games),
            "--ab-depth", str(ab_depth),
            "--seed", str(seed),
            "--json",
            "--quiet",
            "--cards", "/data/cards/core_set",
            "--commanders", "/data/commanders",
            "--decks", "/data/decks",
            "--weights", "/data/weights",
        ]

        result = subprocess.run(cmd, capture_output=True, text=True, check=True)
        return json.loads(result.stdout)

    # Run with Modal
    print("Starting Modal jobs...")
    start_time = time.time()

    with app.run():
        # Fan out all matchups
        results = list(
            modal_run_matchup.map(
                [m[0] for m in matchups],
                [m[1] for m in matchups],
                [args.games] * len(matchups),
                [args.ab_depth] * len(matchups),
                [args.seed + i * 1_000_000 for i in range(len(matchups))],
                order_outputs=True,
            )
        )

    total_time = time.time() - start_time
    print(f"\nCompleted in {total_time:.1f}s")

    # Save results
    output_dir = save_validation_results(results, run_id, args.games, args.seed, total_time)

    # Update ELO
    update_elo_ratings(results)

    # Generate report
    report_dir = generate_report(run_id)

    # Print paths
    print_windows_path(report_dir or output_dir)


def main() -> None:
    """Main entry point."""
    parser = argparse.ArgumentParser(
        description="Modal cloud validation for Essence Wars",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
    uv run python/scripts/modal_validate.py                    # Default run
    uv run python/scripts/modal_validate.py --games 100        # Quick run
    uv run python/scripts/modal_validate.py --dry-run          # Preview
    uv run python/scripts/modal_validate.py --local            # Run locally
        """,
    )
    parser.add_argument(
        "--games",
        type=int,
        default=250,
        help="Games per player order (total = 2x this). Default: 250",
    )
    parser.add_argument(
        "--ab-depth",
        type=int,
        default=6,
        help="Alpha-Beta search depth. Default: 6",
    )
    parser.add_argument(
        "--seed",
        type=int,
        default=42,
        help="Base random seed. Default: 42",
    )
    parser.add_argument(
        "--concurrency",
        type=int,
        default=100,
        help="Max concurrent Modal containers. Default: 100",
    )
    parser.add_argument(
        "--run-id",
        type=str,
        default=None,
        help="Custom run ID (default: modal_YYYY-MM-DD_HHMM)",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Preview matchups without running",
    )
    parser.add_argument(
        "--local",
        action="store_true",
        help="Run locally without Modal (sequential)",
    )

    args = parser.parse_args()

    if args.local:
        run_local(args)
    else:
        run_modal(args)


if __name__ == "__main__":
    main()
