# MCTS Training Analysis Tool

Comprehensive analysis and visualization toolkit for MCTS bot tuning experiments in Essence Wars.

## Features

✅ **Auto-Discovery** - Automatically scans `experiments/mcts/` for all training runs  
✅ **Aggregated CSV** - Exports consolidated dataset with all generation-by-generation data  
✅ **Interactive Dashboard** - Beautiful HTML dashboard with Plotly charts  
✅ **Rich CLI** - Progress bars, colored output, and formatted tables  
✅ **Robust Parsing** - Handles malformed logs gracefully with detailed error reporting  
✅ **Flexible Filtering** - Filter by tag, mode, minimum generations  
✅ **Comparison Metrics** - Identifies best/worst performers, convergence speed, efficiency  

## Installation

Install the analysis dependencies:

```bash
# Install with uv (recommended)
uv sync --group analysis

# Or with pip
pip install -e ".[analysis]"
```

## Quick Start

```bash
# Analyze all experiments and generate dashboard
./scripts/analyze-mcts.sh

# Filter by tag
./scripts/analyze-mcts.sh --tag generalist

# Filter by training mode
./scripts/analyze-mcts.sh --mode multi-opponent

# Minimum generations filter
./scripts/analyze-mcts.sh --min-gens 50

# List available experiments
./scripts/analyze-mcts.sh --list-experiments
```

## Usage

### Basic Usage

```bash
# Analyze all experiments in experiments/mcts/
python python/scripts/mcts_analysis.py

# Or use the convenience wrapper
./scripts/analyze-mcts.sh
```

### Filtering Options

```bash
# Filter by experiment tag (substring match)
python python/scripts/mcts_analysis.py --tag generalist

# Filter by training mode
python python/scripts/mcts_analysis.py --mode multi-opponent

# Only experiments with 50+ generations
python python/scripts/mcts_analysis.py --min-gens 50

# Combine filters
python python/scripts/mcts_analysis.py --tag specialist --mode specialist --min-gens 100
```

### Output Options

```bash
# Custom output directory
python python/scripts/mcts_analysis.py --output results/custom_analysis/

# Skip HTML dashboard (CSV only)
python python/scripts/mcts_analysis.py --no-dashboard

# Skip CSV export (dashboard only)
python python/scripts/mcts_analysis.py --no-csv

# Verbose logging
python python/scripts/mcts_analysis.py --verbose
```

### Examples

```bash
# Analyze all generalist runs from the past week
python python/scripts/mcts_analysis.py --tag generalist

# Compare different training modes
python python/scripts/mcts_analysis.py --min-gens 100

# Quick check of latest experiments
python python/scripts/mcts_analysis.py --list-experiments

# Export raw data for custom analysis
python python/scripts/mcts_analysis.py --no-dashboard --output data/export/
```

## Output

The tool generates:

1. **aggregated_data.csv** - Full dataset with all generations
   - Columns: `experiment_id`, `timestamp`, `tag`, `mode`, `generation`, `best_fitness`, `best_winrate`, `sigma`, `time_seconds`, `cumulative_time_minutes`, `fitness_change`, etc.

2. **summary.csv** - Summary statistics per experiment
   - Columns: `experiment_id`, `tag`, `mode`, `num_generations`, `final_fitness`, `final_winrate`, `total_time_min`, `stop_reason`

3. **dashboard.html** - Interactive visualization dashboard
   - Fitness evolution across experiments
   - Win rate progression
   - CMA-ES sigma adaptation
   - Final performance comparison
   - Convergence speed analysis
   - Training efficiency metrics
   - 3D exploration trajectory

## Dashboard Features

### Interactive Charts

- **Fitness Evolution** - Line chart showing fitness progression for all experiments
- **Win Rate Evolution** - Win rate over generations with 50% baseline
- **Sigma Adaptation** - CMA-ES step size decay (exploration → exploitation)
- **Final Performance** - Bar chart ranking experiments by fitness
- **Convergence Speed** - How quickly each experiment reaches 90% of final fitness
- **Mode Comparison** - Box plots comparing training modes (if multiple exist)
- **Training Efficiency** - Fitness per hour for each experiment
- **Exploration Trajectory** - 3D visualization of generation/sigma/fitness space

### Summary Statistics

- Total experiments analyzed
- Best/average fitness scores
- Best win rate achieved
- Total training time
- Top 5 performers table

### Interactive Features

- **Hover tooltips** - Detailed info on mouse hover
- **Zoom/pan** - Explore data in detail
- **Legend filtering** - Click legend items to show/hide traces
- **Responsive design** - Works on desktop and mobile

## Architecture

The tool is built with a modular architecture:

```
python/
├── cardgame/
│   └── analysis/
│       ├── __init__.py
│       ├── aggregator.py      # Core data aggregation
│       ├── dashboard.py       # HTML dashboard generator
│       ├── parse_log.py       # Log parsing utilities
│       └── visualize.py       # Additional visualization tools
└── scripts/
    ├── mcts_analysis.py       # Main CLI application
    └── analyze_mcts_batch.py  # Original batch analysis (deprecated)
```

### Key Components

**ExperimentAggregator** (`aggregator.py`)
- Scans `experiments/mcts/` directory
- Parses `.log` files with robust error handling
- Extracts metadata and generation metrics
- Builds consolidated pandas DataFrames
- Filters by tag, mode, minimum generations

**MCTSDashboard** (`dashboard.py`)
- Generates interactive Plotly charts
- Creates beautiful HTML dashboard with Jinja2 templates
- Responsive design with gradient styling
- Summary statistics cards and tables

**CLI Application** (`mcts_analysis.py`)
- Rich terminal interface with progress bars
- Argparse for flexible command-line options
- Graceful error handling and logging
- Auto-discovery of experiments

## Programmatic Usage

You can also use the modules directly in Python:

```python
from pathlib import Path
from cardgame.analysis import ExperimentAggregator, MCTSDashboard

# Initialize aggregator
aggregator = ExperimentAggregator(Path("experiments"))

# Aggregate with filters
df = aggregator.aggregate_all(
    min_generations=50,
    mode_filter="multi-opponent",
    tag_filter="generalist"
)

# Get summary stats
summary = aggregator.get_summary_stats()

# Generate dashboard
dashboard = MCTSDashboard(df, summary)
dashboard.generate_html(Path("output/dashboard.html"))

# Access individual runs
for run in aggregator.runs:
    print(f"{run.metadata.tag}: {run.metadata.final_fitness:.2f}")
```

## Data Schema

### Aggregated Data Columns

| Column | Type | Description |
|--------|------|-------------|
| `experiment_id` | str | Full experiment identifier |
| `timestamp` | str | Timestamp (YYYY-MM-DD_HHMM) |
| `tag` | str | Experiment tag (e.g., "generalist-v0.4") |
| `mode` | str | Training mode (e.g., "multi-opponent") |
| `generation` | int | Generation number (0-indexed) |
| `best_fitness` | float | Best fitness in this generation |
| `best_winrate` | float | Win rate percentage |
| `sigma` | float | CMA-ES step size |
| `time_seconds` | float | Time for this generation |
| `cumulative_time_seconds` | float | Total time up to this generation |
| `cumulative_time_minutes` | float | Cumulative time in minutes |
| `fitness_change` | float | Change from previous generation |
| `winrate_change` | float | Win rate delta |
| `seed` | int | Random seed used |
| `final_fitness` | float | Final fitness of this experiment |
| `final_winrate` | float | Final win rate |

## Performance

- **Fast scanning** - Processes 50+ experiments in seconds
- **Efficient parsing** - Regex-based log parsing
- **Parallel-ready** - Can be extended for parallel processing
- **Memory efficient** - Streams data for large datasets

## Troubleshooting

### No experiments found

```bash
# Check directory structure
ls -la experiments/mcts/

# Verify log files exist
find experiments/mcts/ -name "*.log" | head -n 5

# Use verbose mode to debug
python python/scripts/mcts_analysis.py --verbose
```

### Parsing errors

The tool handles malformed logs gracefully:
- Skips experiments with missing log files
- Warns about unparseable data
- Continues processing remaining experiments
- Reports failed parses at the end

### Missing dependencies

```bash
# Reinstall analysis dependencies
uv sync --group analysis

# Or manually
pip install pandas matplotlib seaborn plotly jinja2 rich
```

## Comparison with Old Script

| Feature | Old (`analyze_mcts_batch.py`) | New (`mcts_analysis.py`) |
|---------|-------------------------------|--------------------------|
| Auto-discovery | ❌ Manual directory | ✅ Auto-scans experiments/ |
| CSV export | ❌ None | ✅ Aggregated + Summary |
| HTML dashboard | ❌ Static PNGs only | ✅ Interactive Plotly |
| Error handling | ❌ Crashes on bad logs | ✅ Robust, continues on errors |
| Filtering | ❌ None | ✅ Tag, mode, min-gens |
| Progress bars | ❌ None | ✅ Rich progress indicators |
| CLI interface | ❌ Basic positional args | ✅ Full argparse with help |
| Modular code | ❌ Single 628-line file | ✅ Clean modules |

## Future Enhancements

Potential improvements:
- [ ] Statistical significance testing (t-tests, ANOVA)
- [ ] Hyperparameter correlation analysis
- [ ] Automated experiment recommendations
- [ ] Integration with W&B or MLflow
- [ ] Real-time monitoring dashboard
- [ ] Parallel log parsing for 100+ experiments
- [ ] Export to LaTeX tables for papers

## Contributing

To add new visualizations:

1. Add plot method to `MCTSDashboard` class
2. Call it in `create_all_plots()`
3. The HTML template auto-includes all generated plots

To add new metrics:

1. Add column calculation in `ExperimentRun.to_dataframe()`
2. Use the new column in dashboard plots

## License

Part of Essence Wars project - see root LICENSE file.
