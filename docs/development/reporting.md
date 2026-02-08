# Report Generation System

*Complete guide to HTML report generation for Essence Wars experiments*

**Last Updated:** 2026-02-01

---

## Overview

The Essence Wars reporting system generates comprehensive HTML dashboards for analyzing experiments, validations, and ML training runs. All reports use a consistent dark-themed UI with interactive visualizations.

## Quick Start

```bash
# Generate report for latest run
essence-wars report generate --run-id latest --open

# Generate all reports
essence-wars report generate-all

# View unified leaderboard
essence-wars report leaderboard --format html --output leaderboard.html
```

---

## Report Structure

### Tab System

All reports use a tabbed interface with the following sections:

| Tab | Purpose | Data Source |
|-----|---------|-------------|
| **Overview** | Summary metrics, health score | `summary.json` |
| **Validation** | Deck performance, matchup matrix | `validation_results.json` |
| **Tuning** | CMA-ES training curves | `tuning_logs.json` |
| **Research** | Faction-level analysis | `faction_analysis.json` |
| **ELO** | Rating rankings and history | `data/ratings/*.json` |
| **Benchmark** | Agent evaluation results | `benchmark_results.json` |

### File Organization

```
experiments/reports/{run_id}/
├── index.html              # Main report file
├── assets/
│   ├── overview.png        # Overview charts
│   ├── validation.png      # Validation heatmaps
│   ├── tuning.png          # Tuning curves
│   ├── research.png        # Research charts
│   ├── elo.png             # ELO history
│   └── benchmark.png       # Benchmark results
└── data/
    ├── summary.json        # Summary statistics
    └── raw_data.json       # Raw experiment data
```

---

## Generating Reports

### Single Report

```bash
essence-wars report generate [OPTIONS]
```

**Options:**
| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `--run-id` | str | latest | Run identifier or "latest" |
| `--tabs` | list | all | Comma-separated tab list |
| `--open` | flag | False | Open report in browser |
| `--output` | path | Auto | Custom output directory |

**Examples:**
```bash
# Basic usage - latest run
essence-wars report generate --run-id latest --open

# Specific run with custom tabs
essence-wars report generate \
  --run-id 2026-01-31_1425_baseline \
  --tabs overview,elo,benchmark \
  --open

# Custom output location
essence-wars report generate \
  --run-id production_v1 \
  --output reports/production/
```

### Batch Generation

Generate reports for all runs:

```bash
essence-wars report generate-all [OPTIONS]
```

**Options:**
| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `--experiments-dir` | path | experiments/ | Base experiments directory |
| `--force` | flag | False | Regenerate existing reports |

**Example:**
```bash
# Generate reports for all experiments
essence-wars report generate-all --force
```

### Programmatic Generation

```python
from essence_wars.analysis.report import ReportGenerator

# Initialize generator
generator = ReportGenerator()

# Generate report
report_path = generator.generate_validation_report(
    run_id="latest",
    tabs=["overview", "validation", "elo"],
    open_browser=True,
)

print(f"Report generated: {report_path}")
```

---

## Tab Details

### Overview Tab

**Purpose:** High-level summary of experiment results

**Visualizations:**
- **Health Score Gauge** - Overall experiment quality (0-100)
- **Key Metrics Cards** - Win rates, avg turns, balance metrics
- **Faction Summary Bar Chart** - Win rates by faction
- **P1/P2 Asymmetry Chart** - First player advantage analysis

**Metrics Included:**
- Overall win rate
- Average game length
- Faction balance (variance)
- P1/P2 win rate difference
- Health score components

**Example Data:**
```json
{
  "health_score": 87.5,
  "overall_win_rate": 0.652,
  "avg_game_length": 12.4,
  "faction_balance": 0.032,
  "p1_advantage": 0.048,
  "metrics": {
    "total_games": 1320,
    "total_matchups": 132,
    "games_per_matchup": 10
  }
}
```

---

### Validation Tab

**Purpose:** Detailed deck performance analysis

**Visualizations:**
- **Matchup Heatmap** - Win rates for all deck pairs
- **Deck Performance Table** - Sortable win rates, ELO, games
- **Win Rate Distribution** - Histogram of deck win rates
- **ELO vs Win Rate Scatter** - Correlation analysis

**Features:**
- Hoverable cells with exact win rates
- Sortable columns (win rate, ELO, games played)
- Color-coded performance (green = strong, red = weak)
- Matchup details on cell click

**Example Data:**
```json
{
  "decks": [
    {
      "deck_id": "broodmother_pack",
      "display_name": "Symbiote: Broodmother's Swarm",
      "win_rate": 0.658,
      "elo": 1615.3,
      "games_played": 450,
      "matchups": {
        "celestial_vanguard": {"wins": 18, "losses": 12},
        "void_queen": {"wins": 15, "losses": 15}
      }
    }
  ]
}
```

---

### Tuning Tab

**Purpose:** CMA-ES training progress visualization

**Visualizations:**
- **Fitness Curve** - Best/mean/worst fitness over generations
- **Weight Evolution** - Selected weight parameters over time
- **Convergence Analysis** - Standard deviation of population
- **Generation Statistics Table** - Detailed gen-by-gen data

**Features:**
- Smoothed fitness curves (moving average)
- Interactive legend (toggle series visibility)
- Confidence bands (mean ± std)
- Convergence detection

**Example Data:**
```json
{
  "generations": [
    {
      "generation": 0,
      "best_fitness": 0.523,
      "mean_fitness": 0.487,
      "std_fitness": 0.042,
      "weights": {
        "board_advantage": 1.2,
        "hand_advantage": 0.8,
        "tempo": 1.5
      }
    }
  ]
}
```

---

### Research Tab

**Purpose:** Faction-level strategic analysis

**Visualizations:**
- **Faction Matchup Heatmap** - 3×3 matrix (Argentum, Symbiote, Obsidion)
- **Faction Win Rates Bar Chart** - Overall faction performance
- **Game Length by Faction** - Distribution histograms
- **Combat Efficiency** - Trade ratios and face damage

**Sub-Tabs:**
- **Overview** - High-level faction stats
- **Argentum** - Argentum-specific matchups
- **Symbiote** - Symbiote-specific matchups
- **Obsidion** - Obsidion-specific matchups

**Features:**
- Faction-specific matchup tables
- Win rate confidence intervals
- Average game length by matchup
- Combat efficiency metrics

**Example Data:**
```json
{
  "factions": {
    "argentum": {
      "win_rate": 0.521,
      "avg_game_length": 13.2,
      "matchups": {
        "vs_argentum": 0.498,
        "vs_symbiote": 0.534,
        "vs_obsidion": 0.531
      }
    }
  }
}
```

---

### ELO Tab

**Purpose:** Rating rankings and history

**Visualizations:**
- **Combined Leaderboard** - Decks and agents together
- **Rating History Chart** - Line plot of top competitors
- **Win Rate Comparison** - Bar chart sorted by win rate
- **Rating Distribution** - Histogram with statistics

**Features:**
- Category badges (DECK/AGENT)
- Sortable columns (rating, win rate, games)
- Rating trend indicators (↑↓)
- Historical rating tooltips

**Example Data:**
```json
{
  "leaderboard": [
    {
      "rank": 1,
      "name": "Obsidion: Void Queen (Ruin)",
      "category": "DECK",
      "rating": 1687.2,
      "win_rate": 0.694,
      "games_played": 528
    }
  ]
}
```

---

### Benchmark Tab

**Purpose:** ML agent evaluation results

**Visualizations:**
- **Win Rate by Opponent** - Bar chart
- **Performance Matrix** - Agent vs baseline heatmap
- **Rating Progression** - Line chart over evaluation runs
- **Detailed Results Table** - Win/loss records

**Features:**
- Per-opponent breakdown
- Statistical significance indicators
- Baseline comparison
- Evaluation timestamp tracking

**Example Data:**
```json
{
  "agent_name": "bc_mcts_10k_best",
  "evaluation_timestamp": "2026-01-31T14:25:00Z",
  "results": {
    "random": {"wins": 98, "losses": 2, "win_rate": 0.98},
    "greedy": {"wins": 67, "losses": 33, "win_rate": 0.67},
    "mcts_50": {"wins": 52, "losses": 48, "win_rate": 0.52}
  },
  "overall_elo": 1654.2
}
```

---

## Customization

### Custom Tab Selection

Generate reports with specific tabs only:

```bash
essence-wars report generate \
  --run-id my_experiment \
  --tabs overview,elo \
  --open
```

### Custom Styling

Override default styles by modifying CSS in the report template:

```python
from essence_wars.analysis.report import ReportGenerator

generator = ReportGenerator()
generator.custom_css = """
.tab-content {
    background-color: #1e1e1e;
}
.metric-card {
    border-color: #00ff00;
}
"""
```

### Custom Charts

Add custom visualizations:

```python
from essence_wars.analysis.report.charts import create_custom_chart

# Create custom chart
fig = create_custom_chart(data, title="My Custom Chart")

# Add to report
generator.add_custom_chart(
    tab="overview",
    chart=fig,
    filename="custom_chart.png"
)
```

---

## Report Aggregation

### Aggregate Multiple Experiments

Combine metrics from multiple experiments:

```bash
essence-wars report aggregate [OPTIONS]
```

**Options:**
| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `--experiments-dir` | path | experiments/ | Base experiments directory |
| `--output` | path | aggregated.json | Output JSON file |
| `--format` | choice | json | Output format: json, csv |

**Example:**
```bash
# Aggregate all PPO experiments
essence-wars report aggregate \
  --experiments-dir experiments/ppo/ \
  --output ppo_summary.json

# Export to CSV
essence-wars report aggregate \
  --experiments-dir experiments/ \
  --format csv \
  --output all_experiments.csv
```

### Programmatic Aggregation

```python
from essence_wars.analysis.aggregator import ExperimentAggregator

# Initialize aggregator
aggregator = ExperimentAggregator(base_dir="experiments/")

# Aggregate experiments
summary = aggregator.aggregate_all()

# Access metrics
for exp_id, metrics in summary.items():
    print(f"{exp_id}: Win Rate = {metrics['win_rate']:.2%}")

# Export
aggregator.export_csv("summary.csv")
```

---

## Automation

### Auto-Report After Training

Enable automatic report generation after training:

```bash
essence-wars train ppo \
  --tag my_experiment \
  --auto-report  # Generate report when training completes
```

### Auto-Report in Callbacks

Use callbacks for custom report generation:

```python
from essence_wars.training.callbacks import AutoReportCallback

callback = AutoReportCallback(
    experiment_dir="experiments/my_run",
    tabs=["overview", "elo"],
    open_browser=True,
)

# Use in training
trainer.train(callbacks=[callback])
```

---

## Troubleshooting

### Missing Data Files

**Problem:** Report generation fails with "File not found"

**Solution:** Ensure required data files exist:
```bash
# Check for validation results
ls experiments/validation/latest/validation_results.json

# Check for ratings data
ls data/ratings/deck_elo.json
ls data/ratings/agent_elo.json
```

### Visualization Errors

**Problem:** Charts not rendering in HTML

**Solution:** Check Matplotlib backend:
```python
import matplotlib
matplotlib.use('Agg')  # Non-interactive backend
```

### Browser Not Opening

**Problem:** `--open` flag doesn't open browser

**Solution:** Manually open the report:
```bash
# Get report path
essence-wars report generate --run-id latest

# Open manually
xdg-open experiments/reports/latest/index.html
```

---

## API Reference

### ReportGenerator

```python
from essence_wars.analysis.report import ReportGenerator

class ReportGenerator:
    """Generate HTML reports from experiment data."""

    def __init__(self, base_dir: Path = Path("experiments/")):
        """Initialize generator with base directory."""
        ...

    def generate_validation_report(
        self,
        run_id: str = "latest",
        tabs: list[str] = None,
        open_browser: bool = False,
    ) -> Path:
        """Generate validation report for a specific run.

        Args:
            run_id: Run identifier or "latest"
            tabs: List of tab names to include (default: all)
            open_browser: Whether to open report in browser

        Returns:
            Path to generated report HTML file
        """
        ...

    def generate_all_reports(self, force: bool = False) -> list[Path]:
        """Generate reports for all experiments."""
        ...
```

### Chart Functions

```python
from essence_wars.analysis.report.charts import (
    create_matchup_heatmap,
    create_fitness_curve,
    create_faction_winrate_bar,
    create_elo_history,
)

# Matchup heatmap
fig = create_matchup_heatmap(
    matchup_data: dict[tuple[str, str], float],
    deck_names: list[str],
)

# Fitness curve
fig = create_fitness_curve(
    generations: list[int],
    best_fitness: list[float],
    mean_fitness: list[float],
)

# Faction win rates
fig = create_faction_winrate_bar(
    faction_data: dict[str, float]
)

# ELO history
fig = create_elo_history(
    history_data: dict[str, list[dict]]
)
```

---

## Best Practices

### 1. Generate Reports Regularly

```bash
# After validation runs
cargo run --release --bin validate -- --games 100
essence-wars report generate --run-id latest --open

# After training
essence-wars train ppo --tag exp1 --auto-report
```

### 2. Use Descriptive Run IDs

```bash
# Good: Descriptive run ID
essence-wars report generate --run-id 2026-02-01_baseline_v1

# Bad: Generic "latest"
essence-wars report generate --run-id latest
```

### 3. Select Relevant Tabs

```bash
# Don't generate all tabs if not needed
essence-wars report generate --tabs overview,elo
```

### 4. Archive Important Reports

```bash
# Copy report to permanent location
cp -r experiments/reports/production_v1/ reports/archive/
```

### 5. Aggregate for Meta-Analysis

```bash
# Periodically aggregate experiments
essence-wars report aggregate --output weekly_summary.json
```

---

## See Also

- [CLI Guide](cli-guide.md) - Command-line interface documentation
- [Ratings System](ratings-system.md) - ELO tracking details
- [Training Pipeline](training-pipeline.md) - Automated callbacks
- [Bots & Tuning Pipeline](bots-tuning-pipeline.md) - Rust validation configuration

---

*For issues or feature requests, see the project's GitHub repository.*
