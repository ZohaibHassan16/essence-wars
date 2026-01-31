#!/usr/bin/env python3
"""Generate HTML reports for Essence Wars validation and tuning results.

Usage:
    # Generate report for latest validation run
    uv run python python/scripts/generate_report.py --run-id latest

    # Generate report for specific run
    uv run python python/scripts/generate_report.py --run-id 2026-01-31_1542

    # Generate reports for all validation runs (skip existing)
    uv run python python/scripts/generate_report.py --generate-all

    # Generate all reports since a date
    uv run python python/scripts/generate_report.py --generate-all --since 2026-01-25

    # Generate aggregated dashboard
    uv run python python/scripts/generate_report.py --aggregate

    # Open report in browser after generation
    uv run python python/scripts/generate_report.py --run-id latest --open
"""

from __future__ import annotations

import argparse
import subprocess
import sys
import webbrowser
from datetime import datetime
from pathlib import Path


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Generate HTML reports for Essence Wars experiments",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  %(prog)s --run-id latest              # Generate report for most recent validation
  %(prog)s --run-id 2026-01-31_1542     # Specific validation run
  %(prog)s --generate-all               # Generate all missing reports
  %(prog)s --generate-all --limit 10    # Only 10 most recent runs
  %(prog)s --generate-all --force       # Regenerate all (overwrite existing)
  %(prog)s --aggregate                  # Generate aggregated dashboard
  %(prog)s --run-id latest --open       # Generate and open in browser
        """,
    )

    # Input options (mutually exclusive)
    input_group = parser.add_mutually_exclusive_group(required=True)
    input_group.add_argument(
        "--run-id",
        type=str,
        help="Validation run ID ('latest' for most recent, or specific ID like '2026-01-31_1542')",
    )
    input_group.add_argument(
        "--generate-all",
        action="store_true",
        help="Generate reports for all validation runs (skips existing unless --force)",
    )
    input_group.add_argument(
        "--aggregate",
        action="store_true",
        help="Generate aggregated dashboard across all runs",
    )

    # Batch generation options
    parser.add_argument(
        "--since",
        type=str,
        default=None,
        metavar="DATE",
        help="Only include runs after this date (YYYY-MM-DD format, for --generate-all)",
    )
    parser.add_argument(
        "--limit",
        type=int,
        default=None,
        metavar="N",
        help="Only process N most recent runs (for --generate-all and --aggregate)",
    )
    parser.add_argument(
        "--force",
        action="store_true",
        help="Regenerate reports even if they already exist",
    )

    # Paths
    parser.add_argument(
        "--validation-dir",
        type=Path,
        default=Path("experiments/validation"),
        help="Validation results directory (default: experiments/validation)",
    )
    parser.add_argument(
        "--tuning-dir",
        type=Path,
        default=Path("experiments/mcts"),
        help="Tuning experiments directory (default: experiments/mcts)",
    )
    parser.add_argument(
        "--elo-file",
        type=Path,
        default=None,
        help="ELO ratings file (default: data/ratings/deck_elo.json)",
    )
    parser.add_argument(
        "-o", "--output",
        type=Path,
        default=None,
        help="Output directory (default: experiments/reports)",
    )

    # Content options
    parser.add_argument(
        "--tabs",
        nargs="+",
        choices=["overview", "validation", "tuning", "elo"],
        default=None,
        help="Tabs to include (default: all available)",
    )
    parser.add_argument(
        "--theme",
        choices=["dark", "light"],
        default="dark",
        help="Color theme (default: dark)",
    )

    # Actions
    parser.add_argument(
        "--open",
        action="store_true",
        help="Open report in browser after generation",
    )
    parser.add_argument(
        "-v", "--verbose",
        action="store_true",
        help="Verbose output",
    )

    args = parser.parse_args()

    # Parse --since date if provided
    since_date = None
    if args.since:
        try:
            since_date = datetime.strptime(args.since, "%Y-%m-%d")
        except ValueError:
            print(f"Error: Invalid date format '{args.since}'. Use YYYY-MM-DD.", file=sys.stderr)
            return 1

    # Import here to avoid slow imports when just showing help
    try:
        from essence_wars.analysis.report import ReportGenerator
    except ImportError as e:
        print(f"Error: Failed to import report module: {e}", file=sys.stderr)
        print("Make sure you have installed the analysis dependencies:", file=sys.stderr)
        print("  pip install essence-wars[analysis]", file=sys.stderr)
        print("Or:", file=sys.stderr)
        print("  uv sync --group analysis", file=sys.stderr)
        return 1

    try:
        generator = ReportGenerator(
            output_dir=args.output,
            theme=args.theme,
        )

        output_path = None

        if args.generate_all:
            # Batch generate all reports
            if args.verbose:
                print(f"Scanning for validation runs in: {args.validation_dir}")
                if since_date:
                    print(f"  Filtering runs since: {since_date.strftime('%Y-%m-%d')}")
                if args.limit:
                    print(f"  Limiting to {args.limit} most recent runs")
                if args.force:
                    print("  Force mode: will overwrite existing reports")

            generated = generator.generate_all_reports(
                validation_dir=args.validation_dir,
                tuning_dir=args.tuning_dir,
                elo_file=args.elo_file,
                since=since_date,
                limit=args.limit,
                force=args.force,
                tabs=args.tabs,
            )

            if generated:
                print(f"Generated {len(generated)} report(s):")
                for path in generated:
                    print(f"  {path}")
                output_path = generated[0]  # Open first one if --open
            else:
                print("No new reports to generate (all up to date)")

            # Also regenerate aggregated dashboard after batch generation
            if args.verbose:
                print("Regenerating aggregated dashboard...")
            dashboard_path = generator.generate_aggregated_dashboard(
                validation_dir=args.validation_dir,
                tuning_dir=args.tuning_dir,
                limit=args.limit or 20,
            )
            print(f"Dashboard: {dashboard_path}")
            if not generated:
                output_path = dashboard_path

        elif args.aggregate:
            # Generate aggregated dashboard only
            if args.verbose:
                print("Generating aggregated dashboard...")
                print(f"  Validation dir: {args.validation_dir}")
                print(f"  Tuning dir: {args.tuning_dir}")

            output_path = generator.generate_aggregated_dashboard(
                validation_dir=args.validation_dir,
                tuning_dir=args.tuning_dir,
                limit=args.limit or 20,
            )
            print(f"Dashboard generated: {output_path}")

        else:
            # Single validation report
            if args.verbose:
                print(f"Loading validation data for: {args.run_id}")

            output_path = generator.generate_validation_report(
                run_id=args.run_id,
                validation_dir=args.validation_dir,
                tuning_dir=args.tuning_dir,
                elo_file=args.elo_file,
                tabs=args.tabs,
            )
            print(f"Report generated: {output_path}")

        # Open in browser if requested
        if args.open and output_path:
            url = output_path.absolute().as_uri()
            if args.verbose:
                print(f"Opening in browser: {url}")

            # Try xdg-open first (Linux), then webbrowser module
            try:
                subprocess.run(["xdg-open", str(output_path)], check=True, capture_output=True)
            except (subprocess.CalledProcessError, FileNotFoundError):
                webbrowser.open(url)

        return 0

    except FileNotFoundError as e:
        print(f"Error: {e}", file=sys.stderr)
        return 1
    except Exception as e:
        print(f"Error generating report: {e}", file=sys.stderr)
        if args.verbose:
            import traceback
            traceback.print_exc()
        return 1


if __name__ == "__main__":
    sys.exit(main())
