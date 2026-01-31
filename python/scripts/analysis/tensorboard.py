#!/usr/bin/env python3
"""
Analyze TensorBoard logs from training runs.

Extracts scalar metrics and generates plots showing training progress.

Usage:
    uv run python python/scripts/analyze_tensorboard.py experiments/alphazero/20260118_095729
    uv run python python/scripts/analyze_tensorboard.py --latest
"""

import argparse
import sys
from pathlib import Path

import matplotlib.pyplot as plt
import numpy as np
from tensorboard.backend.event_processing import event_accumulator


def load_tensorboard_data(log_dir: Path) -> dict:
    """Load scalar data from TensorBoard logs."""
    # Find tensorboard subdirectory if it exists
    tb_dir = log_dir / "tensorboard" if (log_dir / "tensorboard").exists() else log_dir
    
    print(f"Loading TensorBoard data from: {tb_dir}")
    
    ea = event_accumulator.EventAccumulator(str(tb_dir))
    ea.Reload()
    
    available_tags = ea.Tags()
    print(f"Available metrics: {available_tags['scalars']}")
    
    data = {}
    for tag in available_tags['scalars']:
        events = ea.Scalars(tag)
        steps = [e.step for e in events]
        values = [e.value for e in events]
        data[tag] = {"steps": steps, "values": values}
    
    return data


def plot_training_metrics(data: dict, output_dir: Path):
    """Generate plots from training metrics."""
    output_dir.mkdir(parents=True, exist_ok=True)
    
    # Create 2x2 subplot
    fig, axes = plt.subplots(2, 2, figsize=(14, 10))
    fig.suptitle("AlphaZero Training Progress", fontsize=16, fontweight='bold')
    
    # Plot 1: Total Loss
    if 'loss/total' in data:
        ax = axes[0, 0]
        steps = data['loss/total']['steps']
        values = data['loss/total']['values']
        ax.plot(steps, values, 'b-', alpha=0.7, linewidth=1)
        
        # Add smoothed line
        if len(values) > 10:
            window = min(20, len(values) // 10)
            smoothed = np.convolve(values, np.ones(window)/window, mode='valid')
            smooth_steps = steps[window-1:]
            ax.plot(smooth_steps, smoothed, 'r-', linewidth=2, label='Smoothed')
            ax.legend()
        
        ax.set_xlabel('Iteration')
        ax.set_ylabel('Loss')
        ax.set_title('Total Loss')
        ax.grid(True, alpha=0.3)
    
    # Plot 2: Policy vs Value Loss
    ax = axes[0, 1]
    if 'loss/policy' in data:
        ax.plot(data['loss/policy']['steps'], data['loss/policy']['values'], 
                'g-', label='Policy Loss', alpha=0.7)
    if 'loss/value' in data:
        ax.plot(data['loss/value']['steps'], data['loss/value']['values'], 
                'm-', label='Value Loss', alpha=0.7)
    ax.set_xlabel('Iteration')
    ax.set_ylabel('Loss')
    ax.set_title('Policy & Value Loss')
    ax.legend()
    ax.grid(True, alpha=0.3)
    
    # Plot 3: Win Rate vs Greedy
    if 'eval/win_rate_vs_greedy' in data:
        ax = axes[1, 0]
        steps = data['eval/win_rate_vs_greedy']['steps']
        values = [v * 100 for v in data['eval/win_rate_vs_greedy']['values']]
        ax.plot(steps, values, 'o-', color='darkblue', markersize=6, linewidth=2)
        ax.axhline(y=60, color='r', linestyle='--', label='Target (60%)', linewidth=2)
        ax.set_xlabel('Iteration')
        ax.set_ylabel('Win Rate (%)')
        ax.set_title('Win Rate vs GreedyBot')
        ax.legend()
        ax.grid(True, alpha=0.3)
        ax.set_ylim([0, 105])
    
    # Plot 4: Win Rate vs Random
    if 'eval/win_rate_vs_random' in data:
        ax = axes[1, 1]
        steps = data['eval/win_rate_vs_random']['steps']
        values = [v * 100 for v in data['eval/win_rate_vs_random']['values']]
        ax.plot(steps, values, 'o-', color='darkgreen', markersize=6, linewidth=2)
        ax.set_xlabel('Iteration')
        ax.set_ylabel('Win Rate (%)')
        ax.set_title('Win Rate vs RandomBot')
        ax.grid(True, alpha=0.3)
        ax.set_ylim([0, 105])
    
    plt.tight_layout()
    
    # Save plot
    plot_path = output_dir / "training_progress.png"
    plt.savefig(plot_path, dpi=150, bbox_inches='tight')
    print(f"\n✓ Saved plot: {plot_path}")
    
    # Show plot
    plt.show()


def print_summary(data: dict):
    """Print summary statistics."""
    print("\n" + "="*60)
    print("Training Summary")
    print("="*60)
    
    if 'loss/total' in data:
        losses = data['loss/total']['values']
        print(f"\nLoss Statistics:")
        print(f"  Final loss:     {losses[-1]:.4f}")
        print(f"  Min loss:       {min(losses):.4f}")
        print(f"  Mean loss:      {np.mean(losses):.4f}")
        if len(losses) > 20:
            print(f"  Last 20 mean:   {np.mean(losses[-20:]):.4f}")
    
    if 'eval/win_rate_vs_greedy' in data:
        win_rates = data['eval/win_rate_vs_greedy']['values']
        steps = data['eval/win_rate_vs_greedy']['steps']
        print(f"\nWin Rate vs GreedyBot:")
        print(f"  Final:          {win_rates[-1]*100:.1f}%")
        print(f"  Best:           {max(win_rates)*100:.1f}%")
        print(f"  Mean:           {np.mean(win_rates)*100:.1f}%")
        
        # Check if target reached
        target = 0.60
        reached = [i for i, v in enumerate(win_rates) if v >= target]
        if reached:
            first_reach = reached[0]
            print(f"  Target (60%):   ✓ Reached at iteration {steps[first_reach]}")
        else:
            print(f"  Target (60%):   ✗ Not reached")
    
    if 'eval/win_rate_vs_random' in data:
        win_rates = data['eval/win_rate_vs_random']['values']
        print(f"\nWin Rate vs RandomBot:")
        print(f"  Final:          {win_rates[-1]*100:.1f}%")
        print(f"  Best:           {max(win_rates)*100:.1f}%")
    
    # Training progress
    if 'loss/total' in data:
        total_iters = len(data['loss/total']['steps'])
        print(f"\nTraining Progress:")
        print(f"  Total iterations: {total_iters}")
        
        if 'eval/win_rate_vs_greedy' in data:
            eval_iters = len(data['eval/win_rate_vs_greedy']['steps'])
            print(f"  Evaluations:      {eval_iters}")


def find_latest_experiment(base_dir: Path = Path("experiments/alphazero")) -> Path:
    """Find the most recent experiment directory."""
    if not base_dir.exists():
        raise FileNotFoundError(f"Directory not found: {base_dir}")
    
    # Find all experiment directories
    exp_dirs = [d for d in base_dir.iterdir() if d.is_dir()]
    if not exp_dirs:
        raise FileNotFoundError(f"No experiment directories found in {base_dir}")
    
    # Sort by modification time
    latest = max(exp_dirs, key=lambda d: d.stat().st_mtime)
    return latest


def main():
    parser = argparse.ArgumentParser(
        description="Analyze TensorBoard logs from training runs",
        formatter_class=argparse.ArgumentDefaultsHelpFormatter,
    )
    parser.add_argument(
        "log_dir",
        nargs="?",
        type=str,
        help="Path to experiment directory (contains tensorboard/ subdirectory)",
    )
    parser.add_argument(
        "--latest",
        action="store_true",
        help="Analyze the most recent experiment in experiments/alphazero/",
    )
    parser.add_argument(
        "--no-plot",
        action="store_true",
        help="Skip plotting (only print summary)",
    )
    
    args = parser.parse_args()
    
    # Determine log directory
    if args.latest:
        log_dir = find_latest_experiment()
        print(f"Using latest experiment: {log_dir}")
    elif args.log_dir:
        log_dir = Path(args.log_dir)
    else:
        parser.error("Must specify log_dir or use --latest")
    
    if not log_dir.exists():
        print(f"Error: Directory not found: {log_dir}")
        sys.exit(1)
    
    # Load data
    try:
        data = load_tensorboard_data(log_dir)
    except Exception as e:
        print(f"Error loading TensorBoard data: {e}")
        sys.exit(1)
    
    if not data:
        print("No scalar data found in TensorBoard logs")
        sys.exit(1)
    
    # Print summary
    print_summary(data)
    
    # Generate plots
    if not args.no_plot:
        plot_dir = log_dir / "plots"
        plot_training_metrics(data, plot_dir)
    
    print("\n" + "="*60)
    print("Analysis complete!")
    print("="*60)


if __name__ == "__main__":
    main()
