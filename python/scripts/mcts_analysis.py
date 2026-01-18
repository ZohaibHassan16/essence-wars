#!/usr/bin/env python3
"""
MCTS Training Analysis Tool
Comprehensive analysis and visualization of MCTS tuning experiments.

Usage:
    python mcts_analysis.py                           # Analyze all experiments
    python mcts_analysis.py --tag generalist          # Filter by tag
    python mcts_analysis.py --mode multi-opponent     # Filter by mode
    python mcts_analysis.py --min-gens 50             # Minimum generations
    python mcts_analysis.py --output results/         # Custom output directory
"""

import argparse
import logging
import sys
from pathlib import Path
from typing import Optional

try:
    from rich.console import Console
    from rich.logging import RichHandler
    from rich.panel import Panel
    from rich.progress import BarColumn, Progress, SpinnerColumn, TextColumn
    from rich.table import Table

    RICH_AVAILABLE = True
except ImportError:
    RICH_AVAILABLE = False

try:
    from essence_wars.analysis.aggregator import ExperimentAggregator
    from essence_wars.analysis.dashboard import MCTSDashboard
except ImportError:
    # Fallback for running from scripts directory
    sys.path.insert(0, str(Path(__file__).parent.parent))
    from essence_wars.analysis.aggregator import ExperimentAggregator
    from essence_wars.analysis.dashboard import MCTSDashboard


def setup_logging(verbose: bool = False) -> logging.Logger:
    """Setup logging with optional rich formatting."""
    level = logging.DEBUG if verbose else logging.INFO

    if RICH_AVAILABLE:
        logging.basicConfig(
            level=level,
            format="%(message)s",
            handlers=[RichHandler(rich_tracebacks=True, show_path=False)],
        )
    else:
        logging.basicConfig(
            level=level,
            format="%(asctime)s - %(name)s - %(levelname)s - %(message)s",
        )

    return logging.getLogger(__name__)


def parse_args() -> argparse.Namespace:
    """Parse command line arguments."""
    parser = argparse.ArgumentParser(
        description="Analyze MCTS training experiments and generate interactive dashboard",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  %(prog)s                                    # Analyze all experiments
  %(prog)s --tag generalist                   # Filter by tag
  %(prog)s --mode multi-opponent              # Filter by training mode
  %(prog)s --min-gens 50                      # Only experiments with 50+ generations
  %(prog)s --output results/custom            # Custom output directory
  %(prog)s --no-dashboard                     # Skip HTML generation (CSV only)
  %(prog)s --verbose                          # Detailed logging
        """,
    )

    parser.add_argument(
        "--experiments",
        type=Path,
        default=Path("experiments"),
        help="Root experiments directory (default: experiments/)",
    )

    parser.add_argument(
        "--output",
        "-o",
        type=Path,
        default=None,
        help="Output directory (default: experiments/mcts/analysis_TIMESTAMP/)",
    )

    parser.add_argument(
        "--tag",
        type=str,
        default=None,
        help="Filter experiments by tag (substring match, e.g., 'generalist')",
    )

    parser.add_argument(
        "--mode",
        type=str,
        default=None,
        help="Filter by training mode (e.g., 'multi-opponent', 'generalist')",
    )

    parser.add_argument(
        "--min-gens",
        type=int,
        default=0,
        help="Minimum number of generations required (default: 0)",
    )

    parser.add_argument(
        "--no-dashboard",
        action="store_true",
        help="Skip HTML dashboard generation (only export CSV)",
    )

    parser.add_argument(
        "--no-csv",
        action="store_true",
        help="Skip CSV export (only generate dashboard)",
    )

    parser.add_argument(
        "--verbose",
        "-v",
        action="store_true",
        help="Enable verbose logging",
    )

    parser.add_argument(
        "--list-experiments",
        action="store_true",
        help="List available experiments and exit",
    )

    parser.add_argument(
        "--no-cache",
        action="store_true",
        help="Disable caching (re-parse all experiments)",
    )

    return parser.parse_args()


def print_summary_table(summary_df, console: Optional["Console"] = None):
    """Print summary table of experiments."""
    if console and RICH_AVAILABLE:
        table = Table(title="📊 Experiment Summary", show_header=True, header_style="bold magenta")
        table.add_column("Tag", style="cyan")
        table.add_column("Mode", style="green")
        table.add_column("Gens", justify="right")
        table.add_column("Fitness", justify="right", style="yellow")
        table.add_column("Win Rate", justify="right", style="blue")
        table.add_column("Time", justify="right")

        for _, row in summary_df.iterrows():
            table.add_row(
                row["tag"][:30],
                row["mode"],
                str(row["num_generations"]),
                f"{row['final_fitness']:.2f}",
                f"{row['final_winrate']:.1f}%",
                f"{row['total_time_min']:.1f}m",
            )

        console.print(table)
    else:
        # Fallback to simple print
        print("\n" + "=" * 100)
        print(
            f"{'Tag':<35} {'Mode':<20} {'Gens':>6} {'Fitness':>10} {'WinRate':>10} {'Time':>10}"
        )
        print("=" * 100)

        for _, row in summary_df.iterrows():
            print(
                f"{row['tag'][:35]:<35} {row['mode']:<20} {row['num_generations']:>6} "
                f"{row['final_fitness']:>10.2f} {row['final_winrate']:>9.1f}% "
                f"{row['total_time_min']:>9.1f}m"
            )

        print("=" * 100 + "\n")


def main():
    """Main entry point."""
    args = parse_args()
    logger = setup_logging(args.verbose)

    # Setup console
    console = Console() if RICH_AVAILABLE else None

    if console:
        console.print(
            Panel.fit(
                "[bold cyan]MCTS Training Analysis Tool[/bold cyan]\n"
                "[dim]Aggregate, analyze, and visualize training experiments[/dim]",
                border_style="cyan",
            )
        )

    # Validate experiments directory
    if not args.experiments.exists():
        logger.error(f"Experiments directory not found: {args.experiments}")
        sys.exit(1)

    # Initialize aggregator
    logger.info(f"Scanning experiments in: {args.experiments}")
    aggregator = ExperimentAggregator(args.experiments)

    # List experiments mode
    if args.list_experiments:
        exp_dirs = aggregator.scan_experiments(
            min_generations=args.min_gens, mode_filter=args.mode, tag_filter=args.tag
        )

        if console:
            console.print(f"\n[bold]Found {len(exp_dirs)} experiments:[/bold]\n")
            for exp_dir in exp_dirs:
                console.print(f"  • {exp_dir.name}")
        else:
            print(f"\nFound {len(exp_dirs)} experiments:")
            for exp_dir in exp_dirs:
                print(f"  - {exp_dir.name}")

        sys.exit(0)

    # Aggregate all experiments
    use_cache = not args.no_cache
    if RICH_AVAILABLE:
        with Progress(
            SpinnerColumn(),
            TextColumn("[progress.description]{task.description}"),
            BarColumn(),
            transient=True,
        ) as progress:
            cache_text = " (cached)" if use_cache else ""
            task = progress.add_task(f"Aggregating experiments{cache_text}...", total=None)

            df = aggregator.aggregate_all(
                min_generations=args.min_gens, mode_filter=args.mode, tag_filter=args.tag, use_cache=use_cache
            )

            progress.update(task, completed=True)
    else:
        print("Aggregating experiments...")
        df = aggregator.aggregate_all(
            min_generations=args.min_gens, mode_filter=args.mode, tag_filter=args.tag, use_cache=use_cache
        )

    if df.empty:
        logger.error("No experiments found matching criteria")
        sys.exit(1)

    logger.info(f"Successfully aggregated {len(aggregator.runs)} experiments")

    # Get summary statistics
    summary_df = aggregator.get_summary_stats()

    # Print summary
    if console:
        console.print()
    print_summary_table(summary_df, console)

    # Determine output directory
    if args.output:
        output_dir = args.output
    else:
        from datetime import datetime

        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        output_dir = args.experiments / "mcts" / f"analysis_{timestamp}"

    output_dir.mkdir(parents=True, exist_ok=True)
    logger.info(f"Output directory: {output_dir}")

    # Export CSV
    if not args.no_csv:
        csv_path = output_dir / "aggregated_data.csv"
        summary_csv_path = output_dir / "summary.csv"

        if console:
            with Progress(
                SpinnerColumn(),
                TextColumn("[progress.description]{task.description}"),
                transient=True,
            ) as progress:
                task = progress.add_task("Exporting CSV files...", total=None)
                df.to_csv(csv_path, index=False)
                summary_df.to_csv(summary_csv_path, index=False)
                progress.update(task, completed=True)
        else:
            print("Exporting CSV files...")
            df.to_csv(csv_path, index=False)
            summary_df.to_csv(summary_csv_path, index=False)

        logger.info(f"✓ Saved aggregated data: {csv_path}")
        logger.info(f"✓ Saved summary: {summary_csv_path}")

    # Generate dashboard
    if not args.no_dashboard:
        dashboard_path = output_dir / "dashboard.html"

        if console:
            with Progress(
                SpinnerColumn(),
                TextColumn("[progress.description]{task.description}"),
                transient=True,
            ) as progress:
                task = progress.add_task("Generating interactive dashboard...", total=None)

                dashboard = MCTSDashboard(df, summary_df)
                dashboard.generate_html(dashboard_path)

                progress.update(task, completed=True)
        else:
            print("Generating interactive dashboard...")
            dashboard = MCTSDashboard(df, summary_df)
            dashboard.generate_html(dashboard_path)

        logger.info(f"✓ Saved dashboard: {dashboard_path}")

        if console:
            console.print(
                f"\n[bold green]✓ Analysis complete![/bold green]\n"
                f"[dim]Open dashboard:[/dim] [cyan]{dashboard_path}[/cyan]\n"
            )
        else:
            print("\n✓ Analysis complete!")
            print(f"Open dashboard: {dashboard_path}\n")

    # Print statistics
    if console:
        stats_panel = Panel(
            f"[bold]Total Experiments:[/bold] {len(summary_df)}\n"
            f"[bold]Best Fitness:[/bold] {summary_df['final_fitness'].max():.2f}\n"
            f"[bold]Avg Fitness:[/bold] {summary_df['final_fitness'].mean():.2f}\n"
            f"[bold]Best Win Rate:[/bold] {summary_df['final_winrate'].max():.1f}%\n"
            f"[bold]Total Training Time:[/bold] {summary_df['total_time_min'].sum()/60:.1f} hours",
            title="📈 Key Statistics",
            border_style="green",
        )
        console.print(stats_panel)


if __name__ == "__main__":
    try:
        main()
    except KeyboardInterrupt:
        print("\n\nOperation cancelled by user")
        sys.exit(130)
    except Exception as e:
        logging.exception(f"Fatal error: {e}")
        sys.exit(1)
