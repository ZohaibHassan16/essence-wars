#!/usr/bin/env python3
"""
Comprehensive analysis script for MCTS bot tuning training logs.
Generates multiple visualizations comparing training runs.
"""

import re
import sys
from pathlib import Path
from dataclasses import dataclass
from typing import List, Dict, Optional
import matplotlib.pyplot as plt
import matplotlib.gridspec as gridspec
import numpy as np
import seaborn as sns

# Set style for publication-quality plots
sns.set_style("whitegrid")
plt.rcParams['figure.figsize'] = (14, 10)
plt.rcParams['font.size'] = 10
plt.rcParams['axes.labelsize'] = 11
plt.rcParams['axes.titlesize'] = 12
plt.rcParams['legend.fontsize'] = 9


@dataclass
class GenerationData:
    """Data for a single generation."""
    generation: int
    best_fitness: float
    best_winrate: float
    sigma: float
    time: float


@dataclass
class TrainingRun:
    """Complete training run data."""
    filename: str
    experiment_id: str
    mode: str
    generations: int
    population: int
    games_per_eval: int
    initial_sigma: float
    seed: int
    data: List[GenerationData]
    final_fitness: float
    final_winrate: float
    total_time: float
    stop_reason: str
    
    @property
    def label(self) -> str:
        """Generate a readable label for plots."""
        # Extract meaningful part from experiment ID
        parts = self.experiment_id.split('_')
        if len(parts) >= 3:
            # Format: YYYY-MM-DD_HHMM_name
            name = '_'.join(parts[2:])
            return f"{name} ({self.mode})"
        return self.experiment_id
    
    @property
    def short_label(self) -> str:
        """Generate a short label for legends."""
        parts = self.experiment_id.split('_')
        if len(parts) >= 3:
            name = '_'.join(parts[2:])
            # Abbreviate common terms
            name = name.replace('generalist', 'gen')
            name = name.replace('specialist', 'spec')
            name = name.replace('argentum', 'arg')
            name = name.replace('symbiote', 'sym')
            name = name.replace('obsidion', 'obs')
            return name
        return self.experiment_id[:20]


def parse_log_file(filepath: Path) -> Optional[TrainingRun]:
    """Parse a training log file into a TrainingRun object."""
    with open(filepath, 'r') as f:
        content = f.read()
    
    # Parse configuration
    config_pattern = r"Experiment ID: ([^\n]+)\s+Mode: ([^\n]+)"
    config_match = re.search(config_pattern, content)
    if not config_match:
        print(f"Warning: Could not parse config from {filepath.name}")
        return None
    
    experiment_id = config_match.group(1).strip()
    mode = config_match.group(2).strip()
    
    # Parse parameters
    def extract_int(pattern: str) -> int:
        match = re.search(pattern, content)
        return int(match.group(1)) if match else 0
    
    def extract_float(pattern: str) -> float:
        match = re.search(pattern, content)
        return float(match.group(1)) if match else 0.0
    
    generations = extract_int(r"Generations: (\d+)")
    population = extract_int(r"Population: (\d+)")
    games_per_eval = extract_int(r"Games/eval: (\d+)")
    initial_sigma = extract_float(r"Initial sigma: ([\d.]+)")
    seed = extract_int(r"Seed: (\d+)")
    
    # Parse generation data
    gen_pattern = r"Gen\s+(\d+):\s+best_fit=\s*([\d.]+),\s+best_wr\s*=\s*([\d.]+)%,\s+sigma=([\d.]+),\s+time=([\d.]+)s"
    generations_data = []
    
    for match in re.finditer(gen_pattern, content):
        gen = int(match.group(1))
        fitness = float(match.group(2))
        winrate = float(match.group(3))
        sigma = float(match.group(4))
        time = float(match.group(5))
        generations_data.append(GenerationData(gen, fitness, winrate, sigma, time))
    
    if not generations_data:
        print(f"Warning: No generation data found in {filepath.name}")
        return None
    
    # Parse final results
    final_fitness = extract_float(r"Best fitness: ([\d.]+)")
    final_winrate = extract_float(r"Best win rate: ([\d.]+)%")
    total_time = extract_float(r"Total time: ([\d.]+)s")
    
    stop_match = re.search(r"Stop reason: ([^\n]+)", content)
    stop_reason = stop_match.group(1).strip() if stop_match else "unknown"
    
    return TrainingRun(
        filename=filepath.name,
        experiment_id=experiment_id,
        mode=mode,
        generations=generations,
        population=population,
        games_per_eval=games_per_eval,
        initial_sigma=initial_sigma,
        seed=seed,
        data=generations_data,
        final_fitness=final_fitness,
        final_winrate=final_winrate,
        total_time=total_time,
        stop_reason=stop_reason
    )


def plot_training_curves(runs: List[TrainingRun], output_dir: Path):
    """Generate comprehensive training curve visualizations."""
    
    # Create figure with multiple subplots
    fig = plt.figure(figsize=(18, 12))
    gs = gridspec.GridSpec(3, 2, figure=fig, hspace=0.3, wspace=0.25)
    
    # 1. Fitness over generations (all runs)
    ax1 = fig.add_subplot(gs[0, :])
    for run in runs:
        gens = [d.generation for d in run.data]
        fitness = [d.best_fitness for d in run.data]
        ax1.plot(gens, fitness, label=run.short_label, linewidth=2, alpha=0.8)
    
    ax1.set_xlabel('Generation')
    ax1.set_ylabel('Best Fitness')
    ax1.set_title('Fitness Evolution Across All Training Runs', fontsize=14, fontweight='bold')
    ax1.legend(bbox_to_anchor=(1.05, 1), loc='upper left', ncol=2)
    ax1.grid(True, alpha=0.3)
    
    # 2. Win rate over generations
    ax2 = fig.add_subplot(gs[1, 0])
    for run in runs:
        gens = [d.generation for d in run.data]
        winrate = [d.best_winrate for d in run.data]
        ax2.plot(gens, winrate, label=run.short_label, linewidth=2, alpha=0.8)
    
    ax2.set_xlabel('Generation')
    ax2.set_ylabel('Win Rate (%)')
    ax2.set_title('Win Rate Evolution', fontsize=12, fontweight='bold')
    ax2.grid(True, alpha=0.3)
    ax2.legend(bbox_to_anchor=(1.05, 1), loc='upper left', fontsize=8)
    
    # 3. Sigma (exploration) over generations
    ax3 = fig.add_subplot(gs[1, 1])
    for run in runs:
        gens = [d.generation for d in run.data]
        sigma = [d.sigma for d in run.data]
        ax3.plot(gens, sigma, label=run.short_label, linewidth=2, alpha=0.8)
    
    ax3.set_xlabel('Generation')
    ax3.set_ylabel('Sigma (Step Size)')
    ax3.set_title('CMA-ES Adaptation (Exploration)', fontsize=12, fontweight='bold')
    ax3.grid(True, alpha=0.3)
    ax3.legend(bbox_to_anchor=(1.05, 1), loc='upper left', fontsize=8)
    
    # 4. Time per generation
    ax4 = fig.add_subplot(gs[2, 0])
    for run in runs:
        gens = [d.generation for d in run.data]
        times = [d.time for d in run.data]
        ax4.plot(gens, times, label=run.short_label, linewidth=1.5, alpha=0.7)
    
    ax4.set_xlabel('Generation')
    ax4.set_ylabel('Time (seconds)')
    ax4.set_title('Evaluation Time per Generation', fontsize=12, fontweight='bold')
    ax4.grid(True, alpha=0.3)
    ax4.legend(bbox_to_anchor=(1.05, 1), loc='upper left', fontsize=8)
    
    # 5. Cumulative time
    ax5 = fig.add_subplot(gs[2, 1])
    for run in runs:
        gens = [d.generation for d in run.data]
        times = [d.time for d in run.data]
        cumulative = np.cumsum(times)
        ax5.plot(gens, cumulative / 60, label=run.short_label, linewidth=2, alpha=0.8)
    
    ax5.set_xlabel('Generation')
    ax5.set_ylabel('Cumulative Time (minutes)')
    ax5.set_title('Total Training Time', fontsize=12, fontweight='bold')
    ax5.grid(True, alpha=0.3)
    ax5.legend(bbox_to_anchor=(1.05, 1), loc='upper left', fontsize=8)
    
    plt.savefig(output_dir / 'training_curves_comprehensive.png', dpi=300, bbox_inches='tight')
    print(f"✓ Saved: training_curves_comprehensive.png")
    plt.close()


def plot_convergence_analysis(runs: List[TrainingRun], output_dir: Path):
    """Analyze convergence patterns and plateaus."""
    
    fig, axes = plt.subplots(2, 2, figsize=(16, 10))
    fig.suptitle('Convergence Analysis', fontsize=16, fontweight='bold')
    
    # 1. Final performance comparison
    ax1 = axes[0, 0]
    modes = [run.mode for run in runs]
    labels = [run.short_label for run in runs]
    fitness = [run.final_fitness for run in runs]
    
    colors = plt.cm.viridis(np.linspace(0, 1, len(runs)))
    bars = ax1.barh(labels, fitness, color=colors)
    ax1.set_xlabel('Final Fitness')
    ax1.set_title('Final Performance Comparison', fontweight='bold')
    ax1.grid(axis='x', alpha=0.3)
    
    # Add value labels
    for i, (bar, val) in enumerate(zip(bars, fitness)):
        ax1.text(val, i, f' {val:.1f}', va='center', fontsize=9)
    
    # 2. Win rate comparison
    ax2 = axes[0, 1]
    winrates = [run.final_winrate for run in runs]
    bars = ax2.barh(labels, winrates, color=colors)
    ax2.set_xlabel('Final Win Rate (%)')
    ax2.set_title('Win Rate Comparison', fontweight='bold')
    ax2.grid(axis='x', alpha=0.3)
    
    for i, (bar, val) in enumerate(zip(bars, winrates)):
        ax2.text(val, i, f' {val:.1f}%', va='center', fontsize=9)
    
    # 3. Improvement over time (first 20% vs last 20%)
    ax3 = axes[1, 0]
    improvements = []
    for run in runs:
        n = len(run.data)
        early_avg = np.mean([d.best_fitness for d in run.data[:n//5]])
        late_avg = np.mean([d.best_fitness for d in run.data[-n//5:]])
        improvement = late_avg - early_avg
        improvements.append(improvement)
    
    bars = ax3.barh(labels, improvements, color=colors)
    ax3.set_xlabel('Fitness Improvement (Late - Early)')
    ax3.set_title('Learning Progress (First 20% → Last 20%)', fontweight='bold')
    ax3.grid(axis='x', alpha=0.3)
    
    for i, (bar, val) in enumerate(zip(bars, improvements)):
        ax3.text(val, i, f' {val:+.1f}', va='center', fontsize=9)
    
    # 4. Training efficiency (fitness per hour)
    ax4 = axes[1, 1]
    efficiencies = []
    for run in runs:
        hours = run.total_time / 3600
        efficiency = run.final_fitness / hours if hours > 0 else 0
        efficiencies.append(efficiency)
    
    bars = ax4.barh(labels, efficiencies, color=colors)
    ax4.set_xlabel('Fitness per Hour')
    ax4.set_title('Training Efficiency', fontweight='bold')
    ax4.grid(axis='x', alpha=0.3)
    
    for i, (bar, val) in enumerate(zip(bars, efficiencies)):
        ax4.text(val, i, f' {val:.1f}', va='center', fontsize=9)
    
    plt.tight_layout()
    plt.savefig(output_dir / 'convergence_analysis.png', dpi=300, bbox_inches='tight')
    print(f"✓ Saved: convergence_analysis.png")
    plt.close()


def plot_mode_comparison(runs: List[TrainingRun], output_dir: Path):
    """Compare performance by training mode (generalist vs specialist)."""
    
    # Group runs by mode
    mode_groups: Dict[str, List[TrainingRun]] = {}
    for run in runs:
        mode = run.mode
        if mode not in mode_groups:
            mode_groups[mode] = []
        mode_groups[mode].append(run)
    
    if len(mode_groups) < 2:
        print("Skipping mode comparison (only one mode found)")
        return
    
    fig, axes = plt.subplots(2, 2, figsize=(16, 10))
    fig.suptitle('Training Mode Comparison', fontsize=16, fontweight='bold')
    
    # 1. Average fitness curves by mode
    ax1 = axes[0, 0]
    for mode, mode_runs in mode_groups.items():
        # Find max generation count
        max_gen = max(len(run.data) for run in mode_runs)
        
        # Aggregate fitness values
        all_fitness = []
        for run in mode_runs:
            fitness = [d.best_fitness for d in run.data]
            # Pad if necessary
            if len(fitness) < max_gen:
                fitness.extend([fitness[-1]] * (max_gen - len(fitness)))
            all_fitness.append(fitness)
        
        # Calculate mean and std
        all_fitness = np.array(all_fitness)
        mean_fitness = np.mean(all_fitness, axis=0)
        std_fitness = np.std(all_fitness, axis=0)
        gens = np.arange(len(mean_fitness))
        
        ax1.plot(gens, mean_fitness, label=f"{mode} (n={len(mode_runs)})", linewidth=2.5)
        ax1.fill_between(gens, mean_fitness - std_fitness, mean_fitness + std_fitness, alpha=0.2)
    
    ax1.set_xlabel('Generation')
    ax1.set_ylabel('Best Fitness')
    ax1.set_title('Average Fitness by Mode', fontweight='bold')
    ax1.legend()
    ax1.grid(True, alpha=0.3)
    
    # 2. Final performance distribution
    ax2 = axes[0, 1]
    mode_names = list(mode_groups.keys())
    final_fitness_by_mode = [[run.final_fitness for run in mode_groups[mode]] for mode in mode_names]
    
    bp = ax2.boxplot(final_fitness_by_mode, labels=mode_names, patch_artist=True)
    for patch, color in zip(bp['boxes'], plt.cm.Set2.colors):
        patch.set_facecolor(color)
    
    ax2.set_ylabel('Final Fitness')
    ax2.set_title('Final Fitness Distribution by Mode', fontweight='bold')
    ax2.grid(axis='y', alpha=0.3)
    
    # 3. Win rate distribution
    ax3 = axes[1, 0]
    final_winrate_by_mode = [[run.final_winrate for run in mode_groups[mode]] for mode in mode_names]
    
    bp = ax3.boxplot(final_winrate_by_mode, labels=mode_names, patch_artist=True)
    for patch, color in zip(bp['boxes'], plt.cm.Set2.colors):
        patch.set_facecolor(color)
    
    ax3.set_ylabel('Win Rate (%)')
    ax3.set_title('Win Rate Distribution by Mode', fontweight='bold')
    ax3.grid(axis='y', alpha=0.3)
    
    # 4. Training time comparison
    ax4 = axes[1, 1]
    time_by_mode = [[run.total_time / 60 for run in mode_groups[mode]] for mode in mode_names]
    
    bp = ax4.boxplot(time_by_mode, labels=mode_names, patch_artist=True)
    for patch, color in zip(bp['boxes'], plt.cm.Set2.colors):
        patch.set_facecolor(color)
    
    ax4.set_ylabel('Total Time (minutes)')
    ax4.set_title('Training Time by Mode', fontweight='bold')
    ax4.grid(axis='y', alpha=0.3)
    
    plt.tight_layout()
    plt.savefig(output_dir / 'mode_comparison.png', dpi=300, bbox_inches='tight')
    print(f"✓ Saved: mode_comparison.png")
    plt.close()


def plot_exploration_dynamics(runs: List[TrainingRun], output_dir: Path):
    """Analyze CMA-ES exploration/exploitation dynamics."""
    
    fig, axes = plt.subplots(2, 2, figsize=(16, 10))
    fig.suptitle('CMA-ES Exploration Dynamics', fontsize=16, fontweight='bold')
    
    # 1. Sigma decay comparison
    ax1 = axes[0, 0]
    for run in runs:
        gens = [d.generation for d in run.data]
        sigma = [d.sigma for d in run.data]
        normalized_sigma = [s / run.initial_sigma for s in sigma]
        ax1.plot(gens, normalized_sigma, label=run.short_label, linewidth=2, alpha=0.8)
    
    ax1.set_xlabel('Generation')
    ax1.set_ylabel('Normalized Sigma (σ/σ₀)')
    ax1.set_title('Step Size Adaptation (Normalized)', fontweight='bold')
    ax1.legend(bbox_to_anchor=(1.05, 1), loc='upper left', fontsize=8)
    ax1.grid(True, alpha=0.3)
    ax1.axhline(y=0.5, color='red', linestyle='--', alpha=0.5, label='50% decay')
    
    # 2. Sigma vs Fitness improvement
    ax2 = axes[0, 1]
    for run in runs:
        sigma = [d.sigma for d in run.data]
        fitness = [d.best_fitness for d in run.data]
        ax2.scatter(sigma, fitness, label=run.short_label, alpha=0.6, s=30)
    
    ax2.set_xlabel('Sigma')
    ax2.set_ylabel('Best Fitness')
    ax2.set_title('Exploration vs Performance', fontweight='bold')
    ax2.legend(bbox_to_anchor=(1.05, 1), loc='upper left', fontsize=8)
    ax2.grid(True, alpha=0.3)
    
    # 3. Fitness variance (as proxy for exploration)
    ax3 = axes[1, 0]
    for run in runs:
        gens = [d.generation for d in run.data]
        fitness = [d.best_fitness for d in run.data]
        
        # Calculate rolling variance (window size 5)
        window = 5
        variances = []
        for i in range(len(fitness)):
            start = max(0, i - window + 1)
            window_data = fitness[start:i+1]
            variances.append(np.var(window_data))
        
        ax3.plot(gens, variances, label=run.short_label, linewidth=2, alpha=0.8)
    
    ax3.set_xlabel('Generation')
    ax3.set_ylabel('Fitness Variance (rolling window=5)')
    ax3.set_title('Search Diversity Over Time', fontweight='bold')
    ax3.legend(bbox_to_anchor=(1.05, 1), loc='upper left', fontsize=8)
    ax3.grid(True, alpha=0.3)
    
    # 4. Sigma trajectory in 2D (sigma vs time, colored by fitness)
    ax4 = axes[1, 1]
    for i, run in enumerate(runs[:6]):  # Limit to 6 for clarity
        gens = [d.generation for d in run.data]
        sigma = [d.sigma for d in run.data]
        fitness = [d.best_fitness for d in run.data]
        
        scatter = ax4.scatter(gens, sigma, c=fitness, cmap='viridis', 
                            alpha=0.6, s=20, label=run.short_label)
    
    ax4.set_xlabel('Generation')
    ax4.set_ylabel('Sigma')
    ax4.set_title('Exploration Trajectory (colored by fitness)', fontweight='bold')
    if len(runs) <= 6:
        ax4.legend(bbox_to_anchor=(1.05, 1), loc='upper left', fontsize=8)
    plt.colorbar(scatter, ax=ax4, label='Fitness')
    ax4.grid(True, alpha=0.3)
    
    plt.tight_layout()
    plt.savefig(output_dir / 'exploration_dynamics.png', dpi=300, bbox_inches='tight')
    print(f"✓ Saved: exploration_dynamics.png")
    plt.close()


def generate_summary_report(runs: List[TrainingRun], output_dir: Path):
    """Generate a markdown summary report."""
    
    report = []
    report.append("# MCTS Bot Tuning Analysis Report\n")
    report.append(f"**Generated:** {Path.cwd()}\n")
    report.append(f"**Total Runs:** {len(runs)}\n\n")
    
    report.append("## Overview\n")
    report.append("| Run | Mode | Generations | Final Fitness | Final WR | Total Time |\n")
    report.append("|-----|------|-------------|---------------|----------|------------|\n")
    
    for run in runs:
        time_str = f"{run.total_time/60:.1f}m"
        report.append(
            f"| {run.short_label} | {run.mode} | {run.generations} | "
            f"{run.final_fitness:.2f} | {run.final_winrate:.1f}% | {time_str} |\n"
        )
    
    # Statistics by mode
    report.append("\n## Performance by Mode\n")
    mode_groups: Dict[str, List[TrainingRun]] = {}
    for run in runs:
        if run.mode not in mode_groups:
            mode_groups[run.mode] = []
        mode_groups[run.mode].append(run)
    
    for mode, mode_runs in mode_groups.items():
        report.append(f"\n### {mode.upper()}\n")
        fitness_values = [r.final_fitness for r in mode_runs]
        winrate_values = [r.final_winrate for r in mode_runs]
        
        report.append(f"- **Count:** {len(mode_runs)}\n")
        report.append(f"- **Fitness:** {np.mean(fitness_values):.2f} ± {np.std(fitness_values):.2f} "
                     f"(range: {min(fitness_values):.2f}-{max(fitness_values):.2f})\n")
        report.append(f"- **Win Rate:** {np.mean(winrate_values):.1f}% ± {np.std(winrate_values):.1f}% "
                     f"(range: {min(winrate_values):.1f}%-{max(winrate_values):.1f}%)\n")
    
    # Top performers
    report.append("\n## Top Performers\n")
    sorted_by_fitness = sorted(runs, key=lambda r: r.final_fitness, reverse=True)
    report.append("\n### By Fitness\n")
    for i, run in enumerate(sorted_by_fitness[:5], 1):
        report.append(f"{i}. **{run.short_label}** - {run.final_fitness:.2f} ({run.final_winrate:.1f}% WR)\n")
    
    sorted_by_winrate = sorted(runs, key=lambda r: r.final_winrate, reverse=True)
    report.append("\n### By Win Rate\n")
    for i, run in enumerate(sorted_by_winrate[:5], 1):
        report.append(f"{i}. **{run.short_label}** - {run.final_winrate:.1f}% ({run.final_fitness:.2f} fitness)\n")
    
    # Key insights
    report.append("\n## Key Insights\n")
    
    # Convergence speed
    fast_learners = []
    for run in runs:
        if len(run.data) >= 20:
            early_fitness = run.data[0].best_fitness
            gen20_fitness = run.data[19].best_fitness
            improvement = gen20_fitness - early_fitness
            fast_learners.append((run, improvement))
    
    fast_learners.sort(key=lambda x: x[1], reverse=True)
    report.append("\n### Fastest Learners (First 20 Generations)\n")
    for run, improvement in fast_learners[:3]:
        report.append(f"- **{run.short_label}**: +{improvement:.1f} fitness\n")
    
    # Training efficiency
    efficiencies = [(run, run.final_fitness / (run.total_time / 3600)) for run in runs]
    efficiencies.sort(key=lambda x: x[1], reverse=True)
    report.append("\n### Most Efficient Training\n")
    for run, eff in efficiencies[:3]:
        report.append(f"- **{run.short_label}**: {eff:.1f} fitness/hour\n")
    
    report_text = ''.join(report)
    
    with open(output_dir / 'analysis_report.md', 'w') as f:
        f.write(report_text)
    
    print(f"✓ Saved: analysis_report.md")
    return report_text


def main():
    if len(sys.argv) < 2:
        print("Usage: python analyze_mcts_batch.py <logs_directory>")
        sys.exit(1)
    
    logs_dir = Path(sys.argv[1])
    if not logs_dir.exists():
        print(f"Error: Directory not found: {logs_dir}")
        sys.exit(1)
    
    # Find all log files
    log_files = list(logs_dir.glob("*.log"))
    if not log_files:
        print(f"Error: No .log files found in {logs_dir}")
        sys.exit(1)
    
    print(f"\n{'='*60}")
    print(f"MCTS Training Analysis")
    print(f"{'='*60}")
    print(f"Directory: {logs_dir}")
    print(f"Found {len(log_files)} log files\n")
    
    # Parse all logs
    runs = []
    for log_file in sorted(log_files):
        print(f"Parsing: {log_file.name}...", end=' ')
        run = parse_log_file(log_file)
        if run:
            runs.append(run)
            print(f"✓ ({run.mode}, {len(run.data)} gens)")
        else:
            print("✗ Failed")
    
    if not runs:
        print("\nError: No valid training runs parsed")
        sys.exit(1)
    
    print(f"\nSuccessfully parsed {len(runs)} training runs")
    
    # Create output directory
    output_dir = logs_dir / 'analysis'
    output_dir.mkdir(exist_ok=True)
    print(f"Output directory: {output_dir}\n")
    
    # Generate plots
    print("Generating visualizations...")
    print("-" * 60)
    
    plot_training_curves(runs, output_dir)
    plot_convergence_analysis(runs, output_dir)
    plot_mode_comparison(runs, output_dir)
    plot_exploration_dynamics(runs, output_dir)
    
    print("-" * 60)
    print("\nGenerating summary report...")
    generate_summary_report(runs, output_dir)
    
    print(f"\n{'='*60}")
    print("✓ Analysis complete!")
    print(f"{'='*60}")
    print(f"\nAll outputs saved to: {output_dir}")
    print("\nGenerated files:")
    print("  - training_curves_comprehensive.png")
    print("  - convergence_analysis.png")
    print("  - mode_comparison.png")
    print("  - exploration_dynamics.png")
    print("  - analysis_report.md")
    print()


if __name__ == '__main__':
    main()
