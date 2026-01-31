#!/usr/bin/env python3
"""Generate HTML reports for Essence Wars validation and tuning results.

Usage:
    # Generate report for latest validation run
    python generate_report.py --run-id latest

    # Generate report for specific run
    python generate_report.py --run-id 2026-01-31_1542

    # Custom output directory and tabs
    python generate_report.py --run-id latest --output ./my_reports --tabs overview validation

    # Open in browser after generation
    python generate_report.py --run-id latest --open
"""

from __future__ import annotations

import argparse
import subprocess
import sys
import webbrowser
from pathlib import Path


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Generate HTML reports for Essence Wars experiments",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  %(prog)s --run-id latest              # Generate report for most recent validation
  %(prog)s --run-id 2026-01-31_1542     # Specific validation run
  %(prog)s --run-id latest --open       # Generate and open in browser
  %(prog)s --all                        # Aggregated dashboard (future)
        """,
    )

    # Input options
    input_group = parser.add_mutually_exclusive_group(required=True)
    input_group.add_argument(
        "--run-id",
        type=str,
        help="Validation run ID ('latest' for most recent, or specific ID like '2026-01-31_1542')",
    )
    input_group.add_argument(
        "--all",
        action="store_true",
        help="Generate aggregated dashboard across all runs (future)",
    )

    # Paths
    parser.add_argument(
        "--validation-dir",
        type=Path,
        default=Path("experiments/validation"),
        help="Validation results directory (default: experiments/validation)",
    )
    parser.add_argument(
        "-o", "--output",
        type=Path,
        default=Path("reports"),
        help="Output directory (default: reports/)",
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

    # Handle aggregated mode (future)
    if args.all:
        print("Aggregated dashboard mode is not yet implemented.", file=sys.stderr)
        print("Use --run-id to generate a single-run report.", file=sys.stderr)
        return 1

    # Generate single-run report
    try:
        generator = ReportGenerator(
            output_dir=args.output,
            theme=args.theme,
        )

        if args.verbose:
            print(f"Loading validation data for: {args.run_id}")

        output_path = generator.generate_validation_report(
            run_id=args.run_id,
            validation_dir=args.validation_dir,
            tabs=args.tabs,
        )

        print(f"Report generated: {output_path}")

        if args.open:
            # Try to open in browser
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
