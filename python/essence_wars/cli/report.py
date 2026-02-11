"""Report generation commands for Essence Wars."""

from __future__ import annotations

import json
import sys
from pathlib import Path
from typing import TYPE_CHECKING, Any

if TYPE_CHECKING:
    import click

    from essence_wars.ratings import LeaderboardEntry


def create_report_group() -> click.Group:
    """Create the report command group."""
    import click

    @click.group()
    def report() -> None:
        """Generate HTML reports and dashboards.

        \b
        Report types:
          generate      - Single validation run report
          generate-all  - Batch generate all missing reports
          aggregate     - Aggregated dashboard across runs
          leaderboard   - Agent/deck leaderboard
        """

    @report.command("generate")
    @click.option(
        "--run-id",
        "-r",
        default="latest",
        help="Validation run ID ('latest' for most recent)",
    )
    @click.option(
        "--validation-dir",
        type=click.Path(exists=True),
        default="experiments/validation",
        help="Validation results directory",
    )
    @click.option(
        "--tuning-dir",
        type=click.Path(),
        default="experiments/mcts",
        help="Tuning experiments directory",
    )
    @click.option("--elo-file", type=click.Path(), default=None, help="ELO ratings file")
    @click.option(
        "--output", "-o", type=click.Path(), default=None, help="Output directory"
    )
    @click.option(
        "--tabs",
        multiple=True,
        default=None,
        type=click.Choice(["overview", "validation", "tuning", "elo"]),
        help="Tabs to include (default: all available)",
    )
    @click.option(
        "--theme",
        type=click.Choice(["dark", "light"]),
        default="dark",
        help="Color theme",
    )
    @click.option(
        "--open",
        "open_browser",
        is_flag=True,
        help="Open report in browser after generation",
    )
    @click.option("--verbose", "-v", is_flag=True, help="Verbose output")
    def report_generate(**kwargs: Any) -> None:
        """Generate an HTML report for a validation run.

        Creates a multi-tab report with overview metrics, validation results,
        tuning experiment data, and ELO ratings.

        \b
        Examples:
          essence-wars report generate --run-id latest --open
          essence-wars report generate --run-id 2026-01-31_1542
          essence-wars report generate --tabs overview validation --theme light
        """
        _run_report_generate(kwargs)

    @report.command("generate-all")
    @click.option(
        "--validation-dir",
        type=click.Path(exists=True),
        default="experiments/validation",
        help="Validation results directory",
    )
    @click.option(
        "--tuning-dir",
        type=click.Path(),
        default="experiments/mcts",
        help="Tuning experiments directory",
    )
    @click.option(
        "--since",
        type=str,
        default=None,
        help="Only include runs after this date (YYYY-MM-DD)",
    )
    @click.option(
        "--limit", type=int, default=None, help="Only process N most recent runs"
    )
    @click.option(
        "--force", is_flag=True, help="Regenerate reports even if they exist"
    )
    @click.option(
        "--theme",
        type=click.Choice(["dark", "light"]),
        default="dark",
        help="Color theme",
    )
    @click.option("--verbose", "-v", is_flag=True, help="Verbose output")
    def report_generate_all(**kwargs: Any) -> None:
        """Generate reports for all validation runs.

        Skips runs that already have reports unless --force is specified.
        Also regenerates the aggregated dashboard.

        \b
        Examples:
          essence-wars report generate-all
          essence-wars report generate-all --since 2026-01-25
          essence-wars report generate-all --limit 10 --force
        """
        _run_report_generate_all(kwargs)

    @report.command("aggregate")
    @click.option(
        "--validation-dir",
        type=click.Path(exists=True),
        default="experiments/validation",
        help="Validation results directory",
    )
    @click.option(
        "--tuning-dir",
        type=click.Path(),
        default="experiments/mcts",
        help="Tuning experiments directory",
    )
    @click.option("--limit", type=int, default=20, help="Maximum runs to include")
    @click.option(
        "--output", "-o", type=click.Path(), default=None, help="Output directory"
    )
    @click.option(
        "--theme",
        type=click.Choice(["dark", "light"]),
        default="dark",
        help="Color theme",
    )
    @click.option(
        "--open", "open_browser", is_flag=True, help="Open dashboard in browser"
    )
    def report_aggregate(**kwargs: Any) -> None:
        """Generate an aggregated dashboard across all runs.

        Shows health score timeline, persistent outliers, and
        summary tables for validation and tuning experiments.

        \b
        Examples:
          essence-wars report aggregate --open
          essence-wars report aggregate --limit 50
        """
        _run_report_aggregate(kwargs)

    @report.command("leaderboard")
    @click.option(
        "--elo-file",
        type=click.Path(exists=True),
        default="data/ratings/deck_elo.json",
        help="ELO ratings file",
    )
    @click.option(
        "--output", "-o", type=click.Path(), default=None, help="Output HTML file"
    )
    @click.option(
        "--format",
        "output_format",
        type=click.Choice(["html", "json", "table"]),
        default="table",
        help="Output format",
    )
    def report_leaderboard(**kwargs: Any) -> None:
        """Display deck/agent leaderboard from ELO ratings.

        \b
        Examples:
          essence-wars report leaderboard
          essence-wars report leaderboard --format html --output leaderboard.html
          essence-wars report leaderboard --format json
        """
        _run_leaderboard(kwargs)

    return report


def _run_report_generate(kwargs: dict[str, Any]) -> None:
    """Run report generation."""
    try:
        from essence_wars.analysis.report import ReportGenerator
    except ImportError as e:
        print(f"Error: Missing dependencies for report generation: {e}")
        print("Install with: pip install essence-wars[analysis]")
        print("Or: uv sync --group analysis")
        sys.exit(1)

    import webbrowser

    generator = ReportGenerator(
        output_dir=kwargs.get("output"),
        theme=kwargs["theme"],
    )

    tabs = list(kwargs["tabs"]) if kwargs["tabs"] else None

    if kwargs["verbose"]:
        print(f"Generating report for: {kwargs['run_id']}")

    output_path = generator.generate_validation_report(
        run_id=kwargs["run_id"],
        validation_dir=Path(kwargs["validation_dir"]),
        tuning_dir=Path(kwargs["tuning_dir"]),
        elo_file=kwargs.get("elo_file"),
        tabs=tabs,
    )

    print(f"Report generated: {output_path}")

    if kwargs["open_browser"]:
        webbrowser.open(output_path.absolute().as_uri())


def _run_report_generate_all(kwargs: dict[str, Any]) -> None:
    """Run batch report generation."""
    try:
        from essence_wars.analysis.report import ReportGenerator
    except ImportError as e:
        print(f"Error: Missing dependencies for report generation: {e}")
        print("Install with: pip install essence-wars[analysis]")
        sys.exit(1)

    from datetime import datetime

    # Parse since date
    since = None
    if kwargs["since"]:
        try:
            since = datetime.strptime(kwargs["since"], "%Y-%m-%d")
        except ValueError:
            print("Error: Invalid date format. Use YYYY-MM-DD.")
            sys.exit(1)

    generator = ReportGenerator(theme=kwargs["theme"])

    if kwargs["verbose"]:
        print(f"Scanning for validation runs in: {kwargs['validation_dir']}")

    generated = generator.generate_all_reports(
        validation_dir=Path(kwargs["validation_dir"]),
        tuning_dir=Path(kwargs["tuning_dir"]),
        since=since,
        limit=kwargs.get("limit"),
        force=kwargs["force"],
    )

    if generated:
        print(f"Generated {len(generated)} report(s)")
        for path in generated[:5]:
            print(f"  {path}")
        if len(generated) > 5:
            print(f"  ... and {len(generated) - 5} more")
    else:
        print("No new reports to generate (all up to date)")

    # Regenerate dashboard
    if kwargs["verbose"]:
        print("Regenerating aggregated dashboard...")
    dashboard_path = generator.generate_aggregated_dashboard(
        validation_dir=Path(kwargs["validation_dir"]),
        tuning_dir=Path(kwargs["tuning_dir"]),
        limit=kwargs.get("limit") or 20,
    )
    print(f"Dashboard: {dashboard_path}")


def _run_report_aggregate(kwargs: dict[str, Any]) -> None:
    """Run aggregated dashboard generation."""
    try:
        from essence_wars.analysis.report import ReportGenerator
    except ImportError as e:
        print(f"Error: Missing dependencies for report generation: {e}")
        print("Install with: pip install essence-wars[analysis]")
        sys.exit(1)

    import webbrowser

    generator = ReportGenerator(
        output_dir=kwargs.get("output"),
        theme=kwargs["theme"],
    )

    output_path = generator.generate_aggregated_dashboard(
        validation_dir=Path(kwargs["validation_dir"]),
        tuning_dir=Path(kwargs["tuning_dir"]),
        limit=kwargs["limit"],
    )

    print(f"Dashboard generated: {output_path}")

    if kwargs["open_browser"]:
        webbrowser.open(output_path.absolute().as_uri())


def _run_leaderboard(kwargs: dict[str, Any]) -> None:
    """Display leaderboard from ELO ratings using unified ratings system."""
    try:
        from essence_wars.ratings import UnifiedRatings
    except ImportError:
        print("Error: ratings module not available")
        sys.exit(1)

    # Load unified ratings (handles missing files gracefully)
    elo_file = kwargs.get("elo_file")
    ratings = UnifiedRatings.load(deck_file=elo_file)

    entries = ratings.get_leaderboard()
    if not entries:
        print("No ratings data found.")
        return

    output_format = kwargs["output_format"]

    if output_format == "json":
        print(
            json.dumps(
                {
                    "leaderboard": [
                        {
                            "rank": e.rank,
                            "identifier": e.identifier,
                            "display_name": e.display_name,
                            "rating": e.rating,
                            "games": e.games,
                            "wins": e.wins,
                            "losses": e.losses,
                            "draws": e.draws,
                            "win_rate": e.win_rate,
                            "category": e.category.value,
                            "faction": e.faction,
                            "agent_type": e.agent_type,
                        }
                        for e in entries
                    ]
                },
                indent=2,
            )
        )

    elif output_format == "table":
        print("\n" + "=" * 75)
        print("ESSENCE WARS LEADERBOARD")
        print("=" * 75)
        print(
            f"{'#':>3}  {'Name':<25}  {'Type':>6}  {'ELO':>6}  {'W-L-D':>10}  {'Win%':>6}"
        )
        print("-" * 75)
        for e in entries:
            wld = f"{e.wins}-{e.losses}-{e.draws}"
            cat = e.category.value[:5].upper()
            print(
                f"{e.rank:>3}  {e.display_name:<25}  {cat:>6}  {e.rating:>6.0f}  "
                f"{wld:>10}  {e.win_rate * 100:>5.1f}%"
            )
        print("=" * 75)

        # Show summary
        summary = ratings.get_summary()
        print(
            f"\nTotal: {summary.get('total_entries', 0)} entries "
            f"({summary.get('total_decks', 0)} decks, "
            f"{summary.get('total_agents', 0)} agents)"
        )

    else:  # HTML
        output = kwargs.get("output") or "leaderboard.html"
        # Generate HTML using unified entries
        html = _generate_leaderboard_html_unified(entries)
        Path(output).write_text(html)
        print(f"Leaderboard saved to: {output}")


def _generate_leaderboard_html(sorted_ratings: list[tuple[str, dict[str, Any]]]) -> str:
    """Generate a simple HTML leaderboard."""
    rows = []
    for i, (deck_id, stats) in enumerate(sorted_ratings, 1):
        rating = stats.get("rating", 1500)
        wins = stats.get("wins", 0)
        losses = stats.get("losses", 0)
        draws = stats.get("draws", 0)
        games = stats.get("games", wins + losses + draws)
        win_rate = (wins / games * 100) if games > 0 else 0
        rows.append(f"""
        <tr>
            <td>{i}</td>
            <td>{deck_id}</td>
            <td>{rating:.0f}</td>
            <td>{wins}-{losses}-{draws}</td>
            <td>{win_rate:.1f}%</td>
        </tr>
        """)

    return f"""<!DOCTYPE html>
<html>
<head>
    <title>Essence Wars Leaderboard</title>
    <style>
        body {{ font-family: sans-serif; max-width: 800px; margin: 0 auto; padding: 20px; }}
        table {{ width: 100%; border-collapse: collapse; }}
        th, td {{ padding: 10px; text-align: left; border-bottom: 1px solid #ddd; }}
        th {{ background: #333; color: white; }}
        tr:hover {{ background: #f5f5f5; }}
    </style>
</head>
<body>
    <h1>Essence Wars Leaderboard</h1>
    <table>
        <thead><tr><th>#</th><th>Deck</th><th>ELO</th><th>W-L-D</th><th>Win%</th></tr></thead>
        <tbody>{"".join(rows)}</tbody>
    </table>
</body>
</html>"""


def _generate_leaderboard_html_unified(entries: list[LeaderboardEntry]) -> str:
    """Generate HTML leaderboard from unified rating entries."""
    rows = []
    for e in entries:
        category_badge = "deck" if e.category.value == "deck" else "agent"
        rows.append(f"""
        <tr>
            <td>{e.rank}</td>
            <td>{e.display_name}</td>
            <td><span class="badge {category_badge}">{e.category.value.upper()}</span></td>
            <td>{e.rating:.0f}</td>
            <td>{e.wins}-{e.losses}-{e.draws}</td>
            <td>{e.win_rate * 100:.1f}%</td>
        </tr>
        """)

    return f"""<!DOCTYPE html>
<html>
<head>
    <title>Essence Wars Leaderboard</title>
    <style>
        body {{ font-family: sans-serif; max-width: 900px; margin: 0 auto; padding: 20px; background: #1a1a2e; color: #eaeaea; }}
        h1 {{ color: #e94560; }}
        table {{ width: 100%; border-collapse: collapse; }}
        th, td {{ padding: 12px; text-align: left; border-bottom: 1px solid #333; }}
        th {{ background: #16213e; color: #D4AF37; }}
        tr:hover {{ background: #0f3460; }}
        .badge {{ padding: 2px 8px; border-radius: 4px; font-size: 0.8em; }}
        .badge.deck {{ background: #D4AF37; color: #000; }}
        .badge.agent {{ background: #00FFFF; color: #000; }}
    </style>
</head>
<body>
    <h1>Essence Wars Leaderboard</h1>
    <table>
        <thead><tr><th>#</th><th>Name</th><th>Type</th><th>ELO</th><th>W-L-D</th><th>Win%</th></tr></thead>
        <tbody>{"".join(rows)}</tbody>
    </table>
</body>
</html>"""
