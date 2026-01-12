#!/usr/bin/env python3
"""
Visualize MCTS Tuning Results - CMA-ES Optimization Analysis
Creates comprehensive plots for ML/AI researchers and hobbyists.
"""

import pandas as pd
import matplotlib.pyplot as plt
import seaborn as sns
import numpy as np
import sys
from pathlib import Path

# Set style for publication-quality plots
plt.style.use('seaborn-v0_8-darkgrid')
sns.set_palette("husl")

def load_data(csv_path: str) -> pd.DataFrame:
    """Load tuning data from CSV."""
    df = pd.read_csv(csv_path)
    print(f"✓ Loaded {len(df)} generations of tuning data")
    return df

def create_overview_plot(df: pd.DataFrame, output_dir: Path):
    """Create main overview with fitness and win rate progression."""
    fig, axes = plt.subplots(2, 2, figsize=(16, 10))
    fig.suptitle('CMA-ES Tuning Overview - MCTS Bot Optimization', 
                 fontsize=16, fontweight='bold')
    
    # Plot 1: Fitness over time
    ax1 = axes[0, 0]
    ax1.plot(df['generation'], df['fitness'], 'b-', linewidth=2, label='Best Fitness')
    ax1.fill_between(df['generation'], df['fitness'].min(), df['fitness'], 
                     alpha=0.3, color='blue')
    ax1.set_xlabel('Generation', fontsize=12)
    ax1.set_ylabel('Fitness Score', fontsize=12)
    ax1.set_title('Fitness Progression\n(Higher is Better)', fontsize=13, fontweight='bold')
    ax1.grid(True, alpha=0.3)
    ax1.legend()
    
    # Add improvement annotations
    max_idx = df['fitness'].idxmax()
    ax1.annotate(f"Best: {df.loc[max_idx, 'fitness']:.2f}",
                xy=(df.loc[max_idx, 'generation'], df.loc[max_idx, 'fitness']),
                xytext=(10, 10), textcoords='offset points',
                bbox=dict(boxstyle='round,pad=0.5', fc='yellow', alpha=0.7),
                arrowprops=dict(arrowstyle='->', connectionstyle='arc3,rad=0'))
    
    # Plot 2: Win Rate over time
    ax2 = axes[0, 1]
    ax2.plot(df['generation'], df['win_rate'], 'g-', linewidth=2, label='Win Rate %')
    ax2.fill_between(df['generation'], 50, df['win_rate'], 
                     alpha=0.3, color='green', where=(df['win_rate'] >= 50))
    ax2.axhline(y=50, color='red', linestyle='--', alpha=0.5, label='50% Baseline')
    ax2.set_xlabel('Generation', fontsize=12)
    ax2.set_ylabel('Win Rate (%)', fontsize=12)
    ax2.set_title('Win Rate Evolution\n(vs Previous Best)', fontsize=13, fontweight='bold')
    ax2.grid(True, alpha=0.3)
    ax2.legend()
    ax2.set_ylim([45, 100])
    
    # Plot 3: Sigma (exploration parameter)
    ax3 = axes[1, 0]
    ax3.plot(df['generation'], df['sigma'], 'r-', linewidth=2, label='Sigma')
    ax3.set_xlabel('Generation', fontsize=12)
    ax3.set_ylabel('Sigma (Step Size)', fontsize=12)
    ax3.set_title('CMA-ES Exploration Parameter\n(Lower = More Converged)', 
                  fontsize=13, fontweight='bold')
    ax3.grid(True, alpha=0.3)
    ax3.legend()
    
    # Add convergence indicator
    final_sigma = df['sigma'].iloc[-1]
    initial_sigma = df['sigma'].iloc[0]
    reduction = ((initial_sigma - final_sigma) / initial_sigma) * 100
    ax3.text(0.05, 0.95, f'Reduction: {reduction:.1f}%\nConvergence: {"High" if reduction > 30 else "Moderate"}',
            transform=ax3.transAxes, fontsize=10,
            verticalalignment='top',
            bbox=dict(boxstyle='round', facecolor='wheat', alpha=0.5))
    
    # Plot 4: Training efficiency
    ax4 = axes[1, 1]
    # Calculate rolling improvement rate
    window = 10
    df['fitness_improvement'] = df['fitness'].diff()
    df['rolling_improvement'] = df['fitness_improvement'].rolling(window=window).mean()
    
    ax4.plot(df['generation'], df['rolling_improvement'], 'purple', linewidth=2,
            label=f'{window}-Gen Moving Avg')
    ax4.axhline(y=0, color='black', linestyle='-', alpha=0.3)
    ax4.set_xlabel('Generation', fontsize=12)
    ax4.set_ylabel('Fitness Improvement per Gen', fontsize=12)
    ax4.set_title('Learning Rate\n(Smoothed Improvement)', fontsize=13, fontweight='bold')
    ax4.grid(True, alpha=0.3)
    ax4.legend()
    
    plt.tight_layout()
    output_path = output_dir / 'overview.png'
    plt.savefig(output_path, dpi=300, bbox_inches='tight')
    print(f"✓ Saved overview plot: {output_path}")
    plt.close()

def create_efficiency_plot(df: pd.DataFrame, output_dir: Path):
    """Create time efficiency analysis."""
    fig, axes = plt.subplots(1, 2, figsize=(16, 6))
    fig.suptitle('Training Efficiency Analysis', fontsize=16, fontweight='bold')
    
    # Plot 1: Fitness vs Time
    ax1 = axes[0]
    scatter = ax1.scatter(df['cumulative_minutes'], df['fitness'], 
                         c=df['generation'], cmap='viridis', s=100, alpha=0.6)
    ax1.plot(df['cumulative_minutes'], df['fitness'], 'k-', alpha=0.3, linewidth=1)
    ax1.set_xlabel('Cumulative Time (minutes)', fontsize=12)
    ax1.set_ylabel('Best Fitness', fontsize=12)
    ax1.set_title('Fitness vs Training Time\n(Color = Generation)', fontsize=13, fontweight='bold')
    ax1.grid(True, alpha=0.3)
    
    cbar = plt.colorbar(scatter, ax=ax1)
    cbar.set_label('Generation', fontsize=11)
    
    # Add efficiency metrics
    total_time = df['cumulative_minutes'].iloc[-1]
    total_improvement = df['fitness'].iloc[-1] - df['fitness'].iloc[0]
    ax1.text(0.05, 0.95, 
            f'Total Time: {total_time:.1f} min\n'
            f'Total Improvement: {total_improvement:.2f}\n'
            f'Efficiency: {total_improvement/total_time:.3f} pts/min',
            transform=ax1.transAxes, fontsize=10,
            verticalalignment='top',
            bbox=dict(boxstyle='round', facecolor='lightblue', alpha=0.7))
    
    # Plot 2: Generation time distribution
    ax2 = axes[1]
    ax2.hist(df['gen_time'], bins=30, color='coral', alpha=0.7, edgecolor='black')
    ax2.axvline(df['gen_time'].mean(), color='red', linestyle='--', 
               linewidth=2, label=f"Mean: {df['gen_time'].mean():.1f}s")
    ax2.axvline(df['gen_time'].median(), color='blue', linestyle='--', 
               linewidth=2, label=f"Median: {df['gen_time'].median():.1f}s")
    ax2.set_xlabel('Generation Time (seconds)', fontsize=12)
    ax2.set_ylabel('Frequency', fontsize=12)
    ax2.set_title('Time per Generation Distribution', fontsize=13, fontweight='bold')
    ax2.legend(fontsize=11)
    ax2.grid(True, alpha=0.3, axis='y')
    
    plt.tight_layout()
    output_path = output_dir / 'efficiency.png'
    plt.savefig(output_path, dpi=300, bbox_inches='tight')
    print(f"✓ Saved efficiency plot: {output_path}")
    plt.close()

def create_convergence_analysis(df: pd.DataFrame, output_dir: Path):
    """Create detailed convergence analysis."""
    fig, axes = plt.subplots(2, 1, figsize=(14, 10))
    fig.suptitle('Convergence & Optimization Dynamics', fontsize=16, fontweight='bold')
    
    # Plot 1: Combined fitness, win rate, and sigma
    ax1 = axes[0]
    
    # Normalize all metrics to 0-1 for comparison
    fitness_norm = (df['fitness'] - df['fitness'].min()) / (df['fitness'].max() - df['fitness'].min())
    winrate_norm = (df['win_rate'] - df['win_rate'].min()) / (df['win_rate'].max() - df['win_rate'].min())
    sigma_norm = (df['sigma'] - df['sigma'].min()) / (df['sigma'].max() - df['sigma'].min())
    
    ax1.plot(df['generation'], fitness_norm, 'b-', linewidth=2, label='Fitness (normalized)', alpha=0.8)
    ax1.plot(df['generation'], winrate_norm, 'g-', linewidth=2, label='Win Rate (normalized)', alpha=0.8)
    ax1.plot(df['generation'], sigma_norm, 'r-', linewidth=2, label='Sigma (normalized)', alpha=0.8)
    
    ax1.set_xlabel('Generation', fontsize=12)
    ax1.set_ylabel('Normalized Value (0-1)', fontsize=12)
    ax1.set_title('Multi-Metric Convergence Analysis\n(All metrics scaled 0-1 for comparison)', 
                 fontsize=13, fontweight='bold')
    ax1.legend(loc='right', fontsize=11)
    ax1.grid(True, alpha=0.3)
    ax1.set_ylim([-0.05, 1.05])
    
    # Plot 2: Phase analysis
    ax2 = axes[1]
    
    # Calculate improvement phases
    df['fitness_change'] = df['fitness'].diff()
    df['phase'] = 'Stable'
    df.loc[df['fitness_change'] > df['fitness_change'].quantile(0.75), 'phase'] = 'High Growth'
    df.loc[df['fitness_change'] < df['fitness_change'].quantile(0.25), 'phase'] = 'Plateau'
    
    # Color-coded scatter
    colors = {'High Growth': 'green', 'Stable': 'orange', 'Plateau': 'red'}
    for phase, color in colors.items():
        mask = df['phase'] == phase
        ax2.scatter(df.loc[mask, 'generation'], df.loc[mask, 'fitness'], 
                   c=color, label=phase, s=80, alpha=0.6, edgecolors='black', linewidth=0.5)
    
    ax2.plot(df['generation'], df['fitness'], 'k-', alpha=0.2, linewidth=1)
    ax2.set_xlabel('Generation', fontsize=12)
    ax2.set_ylabel('Fitness', fontsize=12)
    ax2.set_title('Training Phase Classification\n(Growth vs Plateau periods)', 
                 fontsize=13, fontweight='bold')
    ax2.legend(fontsize=11)
    ax2.grid(True, alpha=0.3)
    
    plt.tight_layout()
    output_path = output_dir / 'convergence.png'
    plt.savefig(output_path, dpi=300, bbox_inches='tight')
    print(f"✓ Saved convergence plot: {output_path}")
    plt.close()

def create_summary_report(df: pd.DataFrame, output_dir: Path):
    """Generate text summary report."""
    report = []
    report.append("=" * 80)
    report.append("CMA-ES TUNING SUMMARY REPORT")
    report.append("=" * 80)
    report.append("")
    
    # Basic stats
    report.append("📊 OVERALL STATISTICS")
    report.append("-" * 80)
    report.append(f"Total Generations:      {len(df)}")
    report.append(f"Total Training Time:    {df['cumulative_minutes'].iloc[-1]:.1f} minutes ({df['cumulative_minutes'].iloc[-1]/60:.2f} hours)")
    report.append(f"Avg Time per Gen:       {df['gen_time'].mean():.2f} seconds")
    report.append("")
    
    # Fitness improvement
    report.append("📈 FITNESS IMPROVEMENT")
    report.append("-" * 80)
    initial_fitness = df['fitness'].iloc[0]
    final_fitness = df['fitness'].iloc[-1]
    best_fitness = df['fitness'].max()
    improvement = final_fitness - initial_fitness
    improvement_pct = (improvement / initial_fitness) * 100
    
    report.append(f"Initial Fitness:        {initial_fitness:.2f}")
    report.append(f"Final Fitness:          {final_fitness:.2f}")
    report.append(f"Best Fitness:           {best_fitness:.2f} (Gen {df['fitness'].idxmax()})")
    report.append(f"Total Improvement:      +{improvement:.2f} ({improvement_pct:+.1f}%)")
    report.append("")
    
    # Win rate analysis
    report.append("🏆 WIN RATE ANALYSIS")
    report.append("-" * 80)
    initial_wr = df['win_rate'].iloc[0]
    final_wr = df['win_rate'].iloc[-1]
    best_wr = df['win_rate'].max()
    wr_improvement = final_wr - initial_wr
    
    report.append(f"Initial Win Rate:       {initial_wr:.1f}%")
    report.append(f"Final Win Rate:         {final_wr:.1f}%")
    report.append(f"Best Win Rate:          {best_wr:.1f}% (Gen {df['win_rate'].idxmax()})")
    report.append(f"Improvement:            +{wr_improvement:.1f} percentage points")
    report.append("")
    
    # Convergence analysis
    report.append("🎯 CONVERGENCE STATUS")
    report.append("-" * 80)
    initial_sigma = df['sigma'].iloc[0]
    final_sigma = df['sigma'].iloc[-1]
    sigma_reduction = ((initial_sigma - final_sigma) / initial_sigma) * 100
    
    report.append(f"Initial Sigma:          {initial_sigma:.4f}")
    report.append(f"Final Sigma:            {final_sigma:.4f}")
    report.append(f"Sigma Reduction:        {sigma_reduction:.1f}%")
    
    if sigma_reduction > 50:
        status = "HIGHLY CONVERGED ✓"
        recommendation = "Good stopping point. Algorithm has converged well."
    elif sigma_reduction > 30:
        status = "MODERATELY CONVERGED"
        recommendation = "Consider running 50-100 more generations for full convergence."
    else:
        status = "STILL EXPLORING"
        recommendation = "Algorithm still exploring. Recommend continuing tuning."
    
    report.append(f"Status:                 {status}")
    report.append(f"Recommendation:         {recommendation}")
    report.append("")
    
    # Efficiency metrics
    report.append("⚡ EFFICIENCY METRICS")
    report.append("-" * 80)
    total_time_mins = df['cumulative_minutes'].iloc[-1]
    fitness_per_min = improvement / total_time_mins
    
    report.append(f"Fitness per Minute:     {fitness_per_min:.4f}")
    report.append(f"Time to 80% WR:         {df[df['win_rate'] >= 80]['cumulative_minutes'].iloc[0] if any(df['win_rate'] >= 80) else 'Not reached':.1f} minutes" if any(df['win_rate'] >= 80) else "Time to 80% WR:         Not reached yet")
    report.append("")
    
    # Milestones
    report.append("🎖️  KEY MILESTONES")
    report.append("-" * 80)
    milestones = [25, 50, 75, 90, 95]
    for wr in milestones:
        matching = df[df['win_rate'] >= wr]
        if not matching.empty:
            first = matching.iloc[0]
            report.append(f"{wr}% Win Rate:          Gen {first['generation']} ({first['cumulative_minutes']:.1f} min)")
    report.append("")
    
    # Learning phases
    report.append("📚 LEARNING PHASES")
    report.append("-" * 80)
    
    # Early phase (first 20%)
    early_end = len(df) // 5
    early_improvement = df['fitness'].iloc[early_end] - df['fitness'].iloc[0]
    
    # Mid phase (20-80%)
    mid_start = early_end
    mid_end = (len(df) * 4) // 5
    mid_improvement = df['fitness'].iloc[mid_end] - df['fitness'].iloc[mid_start]
    
    # Late phase (last 20%)
    late_improvement = df['fitness'].iloc[-1] - df['fitness'].iloc[mid_end]
    
    report.append(f"Early Phase (0-20%):    +{early_improvement:.2f} fitness")
    report.append(f"Mid Phase (20-80%):     +{mid_improvement:.2f} fitness")
    report.append(f"Late Phase (80-100%):   +{late_improvement:.2f} fitness")
    report.append("")
    
    report.append("=" * 80)
    report.append("END OF REPORT")
    report.append("=" * 80)
    
    # Save report
    report_text = "\n".join(report)
    output_path = output_dir / 'summary_report.txt'
    with open(output_path, 'w') as f:
        f.write(report_text)
    
    print("\n" + report_text)
    print(f"\n✓ Saved summary report: {output_path}")

def create_comparison_plot(df: pd.DataFrame, output_dir: Path):
    """Create generation-by-generation comparison plot."""
    fig, ax = plt.subplots(figsize=(14, 8))
    
    # Create dual y-axis plot
    ax1 = ax
    ax2 = ax1.twinx()
    
    # Plot fitness and win rate
    line1 = ax1.plot(df['generation'], df['fitness'], 'b-', linewidth=2.5, 
                     label='Fitness Score', marker='o', markersize=4, alpha=0.7)
    line2 = ax2.plot(df['generation'], df['win_rate'], 'g-', linewidth=2.5, 
                     label='Win Rate %', marker='s', markersize=4, alpha=0.7)
    
    ax1.set_xlabel('Generation', fontsize=13)
    ax1.set_ylabel('Fitness Score', fontsize=13, color='b')
    ax2.set_ylabel('Win Rate (%)', fontsize=13, color='g')
    
    ax1.tick_params(axis='y', labelcolor='b')
    ax2.tick_params(axis='y', labelcolor='g')
    
    # Add reference line for 50% win rate
    ax2.axhline(y=50, color='red', linestyle='--', alpha=0.3, linewidth=1)
    
    # Title and grid
    ax1.set_title('Fitness and Win Rate Progression\n(Dual Axis Comparison)', 
                  fontsize=15, fontweight='bold', pad=20)
    ax1.grid(True, alpha=0.3)
    
    # Combined legend
    lines = line1 + line2
    labels = [l.get_label() for l in lines]
    ax1.legend(lines, labels, loc='lower right', fontsize=12)
    
    plt.tight_layout()
    output_path = output_dir / 'comparison.png'
    plt.savefig(output_path, dpi=300, bbox_inches='tight')
    print(f"✓ Saved comparison plot: {output_path}")
    plt.close()

def main():
    """Main execution function."""
    # Parse arguments
    if len(sys.argv) < 2:
        print("Usage: python visualize_tuning.py <tuning_data.csv> [output_dir]")
        print("\nExample:")
        print("  python visualize_tuning.py tuning_scaling_data.csv")
        print("  python visualize_tuning.py tuning_scaling_data.csv ./plots")
        sys.exit(1)
    
    csv_path = sys.argv[1]
    output_dir = Path(sys.argv[2]) if len(sys.argv) > 2 else Path('./plots')
    
    # Create output directory
    output_dir.mkdir(parents=True, exist_ok=True)
    
    print("\n" + "="*80)
    print("🎨 MCTS TUNING VISUALIZATION SCRIPT")
    print("="*80 + "\n")
    
    # Load data
    df = load_data(csv_path)
    
    if df.empty:
        print("❌ Error: No data found in CSV file")
        sys.exit(1)
    
    print(f"📁 Output directory: {output_dir.absolute()}\n")
    
    # Generate all visualizations
    print("Generating visualizations...\n")
    
    create_overview_plot(df, output_dir)
    create_efficiency_plot(df, output_dir)
    create_convergence_analysis(df, output_dir)
    create_comparison_plot(df, output_dir)
    create_summary_report(df, output_dir)
    
    print("\n" + "="*80)
    print("✅ ALL VISUALIZATIONS COMPLETE!")
    print("="*80)
    print(f"\n📊 Generated files in: {output_dir.absolute()}")
    print("  - overview.png          : Main dashboard with 4 key metrics")
    print("  - efficiency.png        : Training time and efficiency analysis")
    print("  - convergence.png       : Convergence dynamics and phases")
    print("  - comparison.png        : Dual-axis fitness vs win rate")
    print("  - summary_report.txt    : Detailed text summary\n")

if __name__ == '__main__':
    main()
