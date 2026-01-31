# Python Analysis Suite Audit

*Comprehensive analysis and consolidation plan for Engineers, ML/AI Researchers, and Developers*

**Date:** 2026-01-31
**Scope:** `python/essence_wars/` (~26,900 LOC across 74 files)

---

## Executive Summary

The Essence Wars Python analysis suite has evolved organically into a capable but fragmented system. This audit identifies key integration opportunities to create a unified, well-architected experience for the target audience: Engineers, ML/AI Researchers, and Developers.

**Key Findings:**
1. **Three overlapping dashboard systems** with different styling and capabilities
2. **Two separate ELO implementations** that don't communicate
3. **21 standalone scripts** without a unified CLI interface
4. **No automatic pipeline** from training → evaluation → reporting
5. **Strong foundations** that can be consolidated with minimal disruption

---

## Current Architecture Overview

### Directory Structure

```
python/essence_wars/
├── __init__.py          # Core bindings (PyGame, PyParallelGames)
├── env.py               # Gymnasium environment
├── parallel_env.py      # PettingZoo multi-agent environment
├── hub.py               # HuggingFace Hub integration
│
├── agents/              # ML Agent Implementations (~3,100 LOC)
│   ├── ppo.py           # PPO with cleanrl-style implementation
│   ├── alphazero.py     # AlphaZero neural MCTS
│   ├── behavioral_cloning.py  # Supervised learning from MCTS
│   ├── card2vec.py      # Card embeddings
│   └── decision_transformer.py # Sequence modeling approach
│
├── analysis/            # Visualization & Analysis (~4,200 LOC)
│   ├── aggregator.py    # Experiment aggregation
│   ├── dashboard.py     # MCTSDashboard (training analysis)
│   ├── research_dashboard.py  # Balance validation dashboard
│   └── report/          # HTML report generator (tabbed reports)
│       ├── generator.py
│       ├── charts.py
│       ├── tabs/
│       └── loaders/
│
├── benchmark/           # Standardized Evaluation (~1,400 LOC)
│   ├── api.py           # EssenceWarsBenchmark main class
│   ├── agents.py        # Benchmark agent wrappers
│   ├── elo.py           # EloTracker (agent ratings)
│   └── metrics.py       # BenchmarkResults dataclass
│
├── data/                # Dataset Utilities (~1,100 LOC)
│   ├── datasets.py      # MCTSDataset, ChunkedMCTSDataset
│   └── utils.py         # Data loading helpers
│
├── infra/               # Experiment Management (~600 LOC)
│   ├── experiments.py   # Experiment tracking
│   └── logging.py       # Logging utilities
│
├── training/            # Training Utilities (~900 LOC)
│   └── utils.py         # Training helpers, schedulers
│
└── viz/                 # Visualization (~700 LOC)
    └── plots.py         # Matplotlib plotting utilities
```

### Scripts (`python/scripts/`)

| Category | Scripts | Purpose |
|----------|---------|---------|
| **Training** | `train_ppo.py`, `train_alphazero.py`, `train_behavioral_cloning.py`, `train_decision_transformer.py`, `train_card2vec.py`, `train_distilled_policy.py` | ML model training |
| **Evaluation** | `evaluate_neural_mcts.py`, `evaluate_decision_transformer.py`, `run_benchmark.py` | Model evaluation |
| **Data Generation** | `generate_distillation_data.py`, `generate_exit_data.py` | Training data creation |
| **Benchmarking** | `benchmark_env.py`, `benchmark_bc_vs_mcts.py`, `benchmark_batched_mcts.py` | Performance profiling |
| **Analysis** | `mcts_analysis.py`, `analyze_tensorboard.py`, `diagnose_ppo.py` | Training analysis |
| **Reporting** | `generate_report.py`, `generate_leaderboard.py` | HTML report generation |
| **Other** | `generate_card_art.py`, `test_experiment.py` | Utilities |

---

## Gap Analysis

### 1. Three Overlapping Dashboard Systems

| System | Location | Generates | Use Case |
|--------|----------|-----------|----------|
| **MCTSDashboard** | `analysis/dashboard.py` | HTML with Plotly | CMA-ES tuning analysis |
| **Research Dashboard** | `analysis/research_dashboard.py` | HTML with Plotly | Balance validation (faction matchups) |
| **Report Generator** | `analysis/report/generator.py` | HTML with tabs | Unified validation reports |

**Problems:**
- Different CSS themes and styling approaches
- MCTSDashboard loads from CSV, Report Generator from JSON
- Research Dashboard is standalone script, not integrated into module
- No way to combine all views in a single dashboard

**Recommendation:** Consolidate into Report Generator with additional tabs:
- Add "Research" tab with faction matchups from research_dashboard
- Add "Training" tab that subsumes MCTSDashboard functionality
- Deprecate standalone dashboard scripts

---

### 2. Dual ELO Implementations

| Implementation | Location | Purpose | Data Source |
|----------------|----------|---------|-------------|
| **benchmark/elo.py** | `EloTracker` class | Runtime agent rating during benchmarks | In-memory, saves to JSON |
| **report/loaders/elo.py** | `load_elo_data()` | Load deck ELO for reports | `data/ratings/deck_elo.json` (from Rust) |

**Problems:**
- Benchmark ELO tracks agent performance (Python agents)
- Report ELO reads deck performance (from Rust arena binary)
- No unified view of agent vs deck ratings
- Different data structures (`AgentRating` vs `DeckRating`)

**Recommendation:** Create unified ELO layer:
```
essence_wars/
└── ratings/
    ├── __init__.py
    ├── deck_ratings.py    # Reads Rust-generated deck_elo.json
    ├── agent_ratings.py   # Tracks Python agent performance
    └── combined.py        # Unified view with both
```

---

### 3. No Unified CLI

Currently, running different tasks requires:
```bash
# Training
python python/scripts/train_ppo.py --tag my_run ...
python python/scripts/train_alphazero.py ...

# Evaluation
python python/scripts/run_benchmark.py ...
python python/scripts/evaluate_neural_mcts.py ...

# Reports
uv run python python/scripts/generate_report.py --run-id latest
uv run python python/scripts/generate_leaderboard.py
```

**Problems:**
- No discoverability - users must know which script to run
- Inconsistent argument patterns across scripts
- No tab completion or help system
- 21 scripts to maintain separately

**Recommendation:** Create unified CLI using Click or Typer:
```bash
# Proposed interface
essence-wars train ppo --tag my_run --games 1000
essence-wars train alphazero --tag az_v1
essence-wars evaluate agent my_model.pt --vs greedy
essence-wars benchmark --agent my_model.pt
essence-wars report --run latest --open
essence-wars report generate-all --since 2026-01-25
essence-wars dashboard --aggregate
```

---

### 4. Missing Pipeline Integration

```
Current Flow (Manual):
┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│   Training   │ --> │  Evaluation  │ --> │   Reports    │
│  (manual)    │     │   (manual)   │     │   (manual)   │
└──────────────┘     └──────────────┘     └──────────────┘
     ^                     ^                     ^
     │                     │                     │
  User runs             User runs             User runs
  train_*.py          benchmark.py         generate_report.py
```

```
Proposed Flow (Automated):
┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│   Training   │ --> │  Evaluation  │ --> │   Reports    │
│  (hooks)     │     │   (auto)     │     │   (auto)     │
└──────────────┘     └──────────────┘     └──────────────┘
     │                     │                     │
     └─────────────────────┴─────────────────────┘
                    Unified Pipeline
```

**Recommendation:** Add training callbacks:
```python
# In training scripts
@on_training_complete
def auto_evaluate(checkpoint_path):
    benchmark = EssenceWarsBenchmark()
    results = benchmark.evaluate(checkpoint_path)
    save_results(results)

@on_evaluation_complete
def auto_report(results_path):
    generator = ReportGenerator()
    generator.generate_training_report(results_path)
```

---

### 5. Inconsistent Module Boundaries

**Well-organized modules:**
- `agents/` - Clear separation, each agent self-contained
- `data/` - Dataset utilities work well together
- `benchmark/` - Clean API with good abstractions

**Poorly organized:**
- `analysis/` - Mix of aggregators, dashboards, and report generators
- `scripts/` - Flat directory with no categorization
- `viz/` - Single file that could be merged elsewhere

---

## Proposed Consolidated Architecture

### Phase 1: Unified CLI (~200 LOC)

Create `python/essence_wars/cli.py`:

```python
import click

@click.group()
def main():
    """Essence Wars - ML Research Toolkit"""
    pass

@main.group()
def train():
    """Train ML agents"""
    pass

@train.command()
@click.option("--tag", required=True)
@click.option("--games", default=1000)
def ppo(tag, games):
    """Train PPO agent"""
    from .agents.ppo import train_ppo
    train_ppo(tag=tag, games=games)

@main.group()
def report():
    """Generate reports and dashboards"""
    pass

@report.command("generate")
@click.option("--run-id", default="latest")
@click.option("--open", is_flag=True)
def generate_report(run_id, open):
    """Generate HTML report for validation run"""
    from .analysis.report import ReportGenerator
    gen = ReportGenerator()
    path = gen.generate_validation_report(run_id=run_id)
    if open:
        webbrowser.open(path.as_uri())
```

Add entry point in `pyproject.toml`:
```toml
[project.scripts]
essence-wars = "essence_wars.cli:main"
```

---

### Phase 2: Dashboard Consolidation (~500 LOC refactor)

Merge `research_dashboard.py` into `report/`:

```
analysis/report/
├── tabs/
│   ├── overview.py      # Existing
│   ├── validation.py    # Existing
│   ├── tuning.py        # Existing (merge MCTSDashboard)
│   ├── elo.py           # Existing
│   ├── research.py      # NEW: Faction matchups from research_dashboard
│   └── benchmark.py     # NEW: Agent benchmark results
```

Update generator to support all tabs:
```python
AVAILABLE_TABS = {
    "overview": OverviewTab,
    "validation": ValidationTab,
    "tuning": TuningTab,
    "elo": EloTab,
    "research": ResearchTab,    # NEW
    "benchmark": BenchmarkTab,  # NEW
}
```

Deprecate standalone dashboards:
```python
# In research_dashboard.py
import warnings
warnings.warn(
    "research_dashboard.py is deprecated. "
    "Use: essence-wars report --tabs research",
    DeprecationWarning
)
```

---

### Phase 3: Unified Ratings Layer (~300 LOC)

Create `essence_wars/ratings/`:

```python
# ratings/unified.py
from dataclasses import dataclass
from .deck_ratings import load_deck_ratings
from .agent_ratings import AgentRatings

@dataclass
class UnifiedRatings:
    """Combined view of deck and agent ratings."""
    deck_ratings: dict[str, DeckRating]
    agent_ratings: dict[str, AgentRating]

    @classmethod
    def load(cls,
             deck_file: Path = "data/ratings/deck_elo.json",
             agent_file: Path = "data/ratings/agent_elo.json"):
        return cls(
            deck_ratings=load_deck_ratings(deck_file),
            agent_ratings=AgentRatings.load(agent_file),
        )

    def get_leaderboard(self, category: str = "all") -> list[RatingEntry]:
        """Get unified leaderboard of decks and agents."""
        ...
```

---

### Phase 4: Training Pipeline Integration (~400 LOC) - **DONE**

**Implementation:**

Created `python/essence_wars/training/callbacks.py` with structured callback system:

**Core Types:**
- `CallbackContext` - Context passed to callbacks (trainer, experiment_dir, config)
- `TrainingCallback` - Abstract base class with `on_train_start`, `on_step`, `on_train_complete`
- `CallbackList` - Container for multiple callbacks with functional conversion

**Built-in Callbacks:**
- `CheckpointCallback` - Save checkpoints during training
- `EvaluationCallback` - Run periodic evaluation during training
- `AutoEvaluateCallback` - Run final evaluation after training with optional ELO update
- `AutoReportCallback` - Generate HTML report after training
- `LoggingCallback` - Log metrics to console or file

**Convenience Function:**
```python
from essence_wars.training import make_callback

callbacks = make_callback(
    save_path="experiments/run1",
    auto_evaluate=True,
    auto_report=True,
    update_elo=True,
)
```

**CLI Integration (train_ppo.py):**
```bash
# Enable all auto-callbacks
python train_ppo.py --auto-callbacks

# Or individually
python train_ppo.py --auto-evaluate --auto-report --update-elo
```

**Files Created/Modified:**
| File | Changes |
|------|---------|
| `python/essence_wars/training/callbacks.py` | NEW: Full callback system (~500 LOC) |
| `python/essence_wars/training/__init__.py` | Updated: Export all callback types |
| `python/scripts/train_ppo.py` | Added `--auto-callbacks`, `--auto-evaluate`, `--auto-report`, `--update-elo` flags |

---

### Phase 5: Script Reorganization

Move scripts into categorized subpackages:

```
python/scripts/
├── __init__.py
├── training/
│   ├── __init__.py
│   ├── ppo.py
│   ├── alphazero.py
│   ├── behavioral_cloning.py
│   └── decision_transformer.py
├── evaluation/
│   ├── __init__.py
│   ├── benchmark.py
│   └── neural_mcts.py
├── data/
│   ├── __init__.py
│   ├── generate_distillation.py
│   └── generate_exits.py
└── analysis/
    ├── __init__.py
    ├── mcts.py
    └── tensorboard.py
```

Each becomes a thin wrapper calling module functions:
```python
# scripts/training/ppo.py
"""Train PPO agent."""
if __name__ == "__main__":
    from essence_wars.cli import train_ppo_command
    train_ppo_command()
```

---

## Implementation Priority

| Phase | Effort | Impact | Dependencies | Status |
|-------|--------|--------|--------------|--------|
| **1. Unified CLI** | Low | High | None | **DONE** |
| **2. Dashboard Consolidation** | Medium | High | None | **DONE** |
| **3. Unified Ratings** | Low | Medium | None | **DONE** |
| **4. Training Pipeline** | Medium | High | Phase 1, 3 | **DONE** |
| **5. Script Reorganization** | High | Medium | Phase 1 | Open |

**Recommended Order:** 1 → 2 → 3 → 4 → 5

### Phase 1 Implementation (COMPLETED 2026-01-31)

Created `python/essence_wars/cli.py` (~600 LOC) with Click framework:
- `essence-wars train` - PPO, AlphaZero, BC, Card2Vec, Decision Transformer
- `essence-wars benchmark` - Agent evaluation vs baselines
- `essence-wars report` - Generate, generate-all, aggregate, leaderboard
- `essence-wars data` - Generate distillation/exit data

Entry point added to `pyproject.toml`:
```toml
[project.scripts]
essence-wars = "essence_wars.cli:main"
```

Usage: `essence-wars --help`

### Phase 2 Implementation (COMPLETED 2026-01-31)

Consolidated dashboards into unified report generator:

**New Files Created:**
- `python/essence_wars/analysis/report/tabs/research.py` (~300 LOC)
  - Faction vs faction matchup heatmap
  - Faction win rates bar chart
  - Game length distribution histogram
  - Combat efficiency analysis
  - Detailed matchup tables by faction

**Charts Added to `charts.py`:**
- `create_faction_matchup_heatmap()` - 3x3 faction matrix with cell annotations
- `create_faction_winrate_bar()` - Overall faction win rates
- `create_game_length_histogram()` - Distribution with average line
- `create_combat_efficiency_chart()` - Trade ratio and face damage subplots

**Generator Updates:**
- Added "Research" tab to available tabs
- Updated tab labels and content generation
- Added CSS for research tab sub-navigation
- Added JavaScript for faction tab switching

**Deprecation Warnings Added:**
- `analysis/research_dashboard.py` - Warns to use `essence-wars report generate`
- `analysis/dashboard.py` - Warns to use unified report (Tuning tab)

**Usage:**
```bash
# Generate report with all tabs including Research
essence-wars report generate --run-id latest --open

# Reports now available at: experiments/reports/{run_id}/index.html
```

### Phase 3 Implementation (COMPLETED 2026-01-31)

Created unified ratings layer at `python/essence_wars/ratings/`:

**New Files Created:**
- `ratings/__init__.py` - Module exports
- `ratings/base.py` (~100 LOC) - Base classes and ELO utilities
  - `BaseRating` abstract class with common properties
  - `RatingCategory` enum (DECK, AGENT)
  - `expected_score()` and `calculate_rating_change()` functions
- `ratings/deck_ratings.py` (~200 LOC) - Deck ratings from Rust arena
  - `DeckRating` dataclass with faction, commander_name, history
  - `DeckRatings` manager with load/get_ranked/predict_matchup
- `ratings/agent_ratings.py` (~200 LOC) - Agent ratings from Python benchmarks
  - `AgentRating` dataclass with agent_type, model_path
  - `AgentRatings` manager with live tracking and persistence
- `ratings/unified.py` (~200 LOC) - Combined view
  - `UnifiedRatings` with deck_ratings + agent_ratings
  - `LeaderboardEntry` dataclass for combined leaderboards
  - `get_leaderboard()`, `get_summary()`, `format_leaderboard()`

**CLI Updates:**
- `essence-wars report leaderboard` now uses unified ratings
- Shows both deck and agent ratings in combined view
- HTML output with dark theme and category badges

**Usage:**
```python
from essence_wars.ratings import UnifiedRatings

ratings = UnifiedRatings.load()
print(ratings.format_leaderboard(limit=10))

# Access specific categories
for deck in ratings.deck_ratings.get_ranked()[:5]:
    print(f"{deck.display_name}: {deck.rating:.0f}")

# Track agent games
ratings.agent_ratings.update("my_ppo", "greedy", winner=0)
ratings.agent_ratings.save()
```

```bash
# CLI usage
essence-wars report leaderboard
essence-wars report leaderboard --format json
essence-wars report leaderboard --format html --output leaderboard.html
```

---

## API Design Principles

For the target audience (Engineers, ML/AI Researchers, Developers):

### 1. Progressive Disclosure
```python
# Simple use (researcher)
from essence_wars import EssenceWarsEnv
env = EssenceWarsEnv()

# Advanced use (engineer)
from essence_wars.benchmark import EssenceWarsBenchmark
benchmark = EssenceWarsBenchmark(
    opponents=["greedy", "mcts-100"],
    games_per_opponent=100,
    elo_tracking=True,
)
```

### 2. Sensible Defaults
```bash
# Just works
essence-wars train ppo

# Customizable
essence-wars train ppo \
    --tag production_v1 \
    --games 10000 \
    --lr 3e-4 \
    --callbacks auto-evaluate,auto-report
```

### 3. Composable Components
```python
# Mix and match
from essence_wars.analysis.report import ReportGenerator
from essence_wars.analysis.report.tabs import OverviewTab, EloTab

gen = ReportGenerator(tabs=[OverviewTab, EloTab])  # Custom tab selection
```

### 4. Clear Documentation
```python
class EssenceWarsBenchmark:
    """Standardized evaluation for Essence Wars agents.

    Example:
        >>> benchmark = EssenceWarsBenchmark()
        >>> results = benchmark.evaluate(my_agent)
        >>> print(f"Win rate: {results.win_rate_vs_greedy:.1%}")
        Win rate: 65.0%

    See Also:
        - `essence-wars benchmark --help` for CLI usage
        - docs/benchmarking.md for methodology
    """
```

---

## Migration Path

### For Existing Users

**Phase 1 (Non-breaking):**
- Add CLI as new entry point
- Keep all existing scripts working
- Add deprecation warnings to old dashboards

**Phase 2 (Deprecations):**
- Mark old dashboard scripts as deprecated
- Encourage migration to unified CLI
- Keep backward compatibility for 6 months

**Phase 3 (Cleanup):**
- Remove deprecated scripts
- Consolidate documentation
- Release v1.0 of unified API

---

## Success Metrics

1. **Discoverability:** `essence-wars --help` shows all available commands
2. **Integration:** Training → Evaluation → Report runs with single command
3. **Consistency:** All reports use same styling and components
4. **Documentation:** Every public API has docstrings and examples
5. **Performance:** No regression in benchmark throughput

---

## Appendix: File-by-File Analysis

### High-Value Files (Keep & Enhance)
- `analysis/report/generator.py` - Solid foundation for reports
- `benchmark/api.py` - Clean evaluation interface
- `agents/ppo.py` - Well-implemented PPO
- `data/datasets.py` - Efficient dataset loaders

### Consolidation Candidates (Merge)
- `analysis/dashboard.py` → `analysis/report/tabs/tuning.py`
- `analysis/research_dashboard.py` → `analysis/report/tabs/research.py`
- `benchmark/elo.py` + `report/loaders/elo.py` → `ratings/`

### Cleanup Candidates (Simplify)
- `viz/plots.py` - Merge into analysis module
- Multiple benchmark scripts - Consolidate to single CLI command

---

*Document generated as part of Analysis Suite Audit - Phase D*
