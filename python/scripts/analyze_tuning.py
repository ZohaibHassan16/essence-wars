#!/usr/bin/env python3
"""
Analyze MCTS tuning experiments: parse logs, create visualizations, and generate reports.

This script orchestrates the complete analysis pipeline for tuning experiments,
following the data strategy documented in docs/data-strategy.md
"""

import argparse
import sys
from pathlib import Path

from cardgame.analysis.parse_log import parse_tuning_log, print_summary, export_csv
from cardgame.analysis.visualize import (
    load_data,
    create_overview_plot,
    create_efficiency_plot,
    create_convergence_analysis,
    create_comparison_plot,
    create_summary_report,
)
from cardgame.infra.experiment import Experiment


def analyze_experiment(exp_dir: Path, force: bool = False) -> None:
    """
    Analyze a single tuning experiment.
    
    Args:
        exp_dir: Path to experiment directory
        force: Force re-analysis even if outputs exist
    """
    print(f"\n{'='*80}")
    print(f"Analyzing Experiment: {exp_dir.name}")
    print(f"{'='*80}\n")
    
    # Check for required files
    log_file = exp_dir / "train.log"
    run_log = exp_dir / "run.log"
    
    # Try both possible log filenames
    if log_file.exists():
        log_path = log_file
    elif run_log.exists():
        log_path = run_log
    else:
        print(f"❌ No log file found in {exp_dir}")
        print(f"   Expected: train.log or run.log")
        return
    
    # Output paths
    stats_csv = exp_dir / "stats.csv"
    plots_dir = exp_dir / "plots"
    plots_dir.mkdir(exist_ok=True)
    
    # Step 1: Parse log file
    print(f"📄 Parsing log file: {log_path.name}")
    data = parse_tuning_log(str(log_path))
    
    if not data:
        print("❌ No tuning data found in log file")
        return
    
    # Step 2: Export to CSV
    print(f"\n💾 Exporting data to CSV...")
    export_csv(data, str(stats_csv))
    
    # Step 3: Print summary
    print_summary(data)
    
    # Step 4: Create visualizations
    if not (plots_dir / "overview.png").exists() or force:
        print(f"\n🎨 Creating visualizations in {plots_dir}/...")
        
        # Load data as DataFrame
        df = load_data(str(stats_csv))
        
        # Generate all plots
        create_overview_plot(df, plots_dir)
        create_efficiency_plot(df, plots_dir)
        create_convergence_analysis(df, plots_dir)
        create_comparison_plot(df, plots_dir)
        create_summary_report(df, plots_dir)
        
        print(f"\n✅ Analysis complete!")
    else:
        print(f"\n✓ Visualizations already exist (use --force to regenerate)")
    
    # Summary
    print(f"\n{'='*80}")
    print(f"📊 Results saved in: {exp_dir}")
    print(f"  - stats.csv           : Parsed metrics")
    print(f"  - plots/overview.png  : Main dashboard")
    print(f"  - plots/efficiency.png: Training efficiency")
    print(f"  - plots/convergence.png: Convergence analysis")
    print(f"  - plots/comparison.png: Dual-axis comparison")
    print(f"  - plots/summary_report.txt: Text summary")
    print(f"{'='*80}\n")


def find_experiments(base_dir: Path, pattern: str = "*") -> list[Path]:
    """Find experiment directories matching a pattern."""
    experiments = []
    
    # Search in mcts subdirectory
    mcts_dir = base_dir / "mcts"
    if mcts_dir.exists():
        for exp_dir in mcts_dir.glob(pattern):
            if exp_dir.is_dir():
                experiments.append(exp_dir)
    
    return sorted(experiments)


def main():
    parser = argparse.ArgumentParser(
        description="Analyze MCTS tuning experiments",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  # Analyze the latest experiment
  %(prog)s --latest
  
  # Analyze a specific experiment
  %(prog)s experiments/mcts/2026-01-12_1430_baseline
  
  # Analyze all experiments
  %(prog)s --all
  
  # Force regeneration of plots
  %(prog)s --latest --force
        """
    )
    
    parser.add_argument(
        "experiment",
        nargs="?",
        type=Path,
        help="Path to specific experiment directory"
    )
    
    parser.add_argument(
        "--latest",
        action="store_true",
        help="Analyze the most recent experiment"
    )
    
    parser.add_argument(
        "--all",
        action="store_true",
        help="Analyze all experiments in experiments/mcts/"
    )
    
    parser.add_argument(
        "--base-dir",
        type=Path,
        default=Path("experiments"),
        help="Base experiments directory (default: experiments)"
    )
    
    parser.add_argument(
        "--force",
        action="store_true",
        help="Force regeneration of plots even if they exist"
    )
    
    args = parser.parse_args()
    
    # Determine which experiments to analyze
    experiments_to_analyze = []
    
    if args.experiment:
        # Specific experiment provided
        if not args.experiment.exists():
            print(f"❌ Experiment directory not found: {args.experiment}")
            sys.exit(1)
        experiments_to_analyze = [args.experiment]
    
    elif args.latest:
        # Find latest experiment
        experiments = find_experiments(args.base_dir)
        if not experiments:
            print(f"❌ No experiments found in {args.base_dir / 'mcts'}")
            sys.exit(1)
        experiments_to_analyze = [experiments[-1]]
        print(f"🔍 Found latest experiment: {experiments[-1].name}")
    
    elif args.all:
        # Find all experiments
        experiments_to_analyze = find_experiments(args.base_dir)
        if not experiments_to_analyze:
            print(f"❌ No experiments found in {args.base_dir / 'mcts'}")
            sys.exit(1)
        print(f"🔍 Found {len(experiments_to_analyze)} experiments")
    
    else:
        # No arguments - try to find latest
        experiments = find_experiments(args.base_dir)
        if experiments:
            experiments_to_analyze = [experiments[-1]]
            print(f"🔍 No experiment specified, analyzing latest: {experiments[-1].name}")
        else:
            parser.print_help()
            print(f"\n❌ No experiments found in {args.base_dir / 'mcts'}")
            sys.exit(1)
    
    # Analyze each experiment
    for exp_dir in experiments_to_analyze:
        try:
            analyze_experiment(exp_dir, force=args.force)
        except Exception as e:
            print(f"❌ Error analyzing {exp_dir}: {e}")
            if args.all:
                continue  # Continue with other experiments
            else:
                sys.exit(1)


if __name__ == "__main__":
    main()
