# Current Binary Landscape

*Last updated: 2026-01-31*

## Binary Inventory

| Binary | Lines | Purpose | Complexity |
|--------|-------|---------|------------|
| generate_dataset | ~710 | ML training data generation | Very High |
| tune | ~700 | CMA-ES weight optimization | High |
| arena | ~450 | Bot matches | Medium |
| swiss | ~200 | Swiss-system tournaments | Medium |
| diagnose | ~310 | P1/P2 asymmetry analysis | Medium |
| card_stats | ~340 | Per-card win contribution analysis | Medium |
| validate | ~300 | Balance testing + auto-diagnose | Medium |
| balance_diff | ~290 | Compare validation runs | Low |
| weight_diff | ~180 | Compare weight files | Low |
| dataset_stats | ~270 | Dataset statistics | Low |
| replay | ~350 | Game replay viewer | Low |
| insight_conditions | ~210 | Catch-up mechanic analysis | Low |
| profile_mcts | ~190 | Performance benchmarks | Low |
| starvation_analysis | ~180 | Hand starvation patterns | Low |
| find_tutorial_seed | ~140 | Tutorial hand finder | Low |
| validate_decks | ~110 | Deck integrity checks | Low |

---

## Opportunities Identified

### ✅ 1. Cross-Binary Code Consolidation [DONE]

~~Significant duplication in data loading, bot configuration, weight resolution, and output formatting. Could extract into a cli_common module.~~

**Status: DONE**

Comprehensive `execution/` module implemented (~3,600 lines):
- `GameData::load_with_overrides()` - Unified data loading
- `parse_bot_type_or_exit()` - Shared bot configuration
- `MctsArgs`, `AlphaBetaArgs`, `DataPaths` - Common CLI utilities
- `run_game_loop()` - Unified game loop with safety limits
- `run_batch_parallel()` - Parallel execution with progress
- `MatchupBuilder` - Matchup generation at multiple granularities

---

### ✅ 2. Integration Gaps [DONE]

| Gap | Status |
|-----|--------|
| validate and diagnose are siloed - validate could auto-invoke diagnose for outlier decks | DONE (`--auto-diagnose`) |
| tune doesn't connect to validate for post-training verification | DONE |
| No way to trace back from balance issues to specific cards | DONE (card_stats binary) |

---

### ✅ 3. Missing Capabilities [DONE]

| Idea | Description | Status |
|------|-------------|--------|
| Replay binary | Replay games from seeds or dataset entries for debugging | DONE |
| Card statistics | Per-card win contribution, usage rates, timing analysis | DONE (card_stats binary) |
| Balance diff | Compare validation runs across versions | DONE |
| Weight comparison | Side-by-side weight file analysis | DONE |
| Dataset analysis | Statistics on existing datasets (distribution, coverage) | DONE |

---

### ✅ 4. Validate Enhancements [DONE]

| Feature | Status |
|---------|--------|
| Matchup matrix output (`--matrix`, `--matrix-csv`) | DONE |
| Per-card win rate contribution (identify problematic cards, not just decks) | DONE (card_stats binary) |
| Historical tracking (trend across versions) | DONE (timestamped dirs + balance_diff) |
| HTML report generation with charts | DONE (generate_report.py) |

---

### ✅ 5. Arena Enhancements [DONE]

| Feature | Status |
|---------|--------|
| Tournament modes (Swiss) | DONE (swiss binary) |
| ELO rating persistence across sessions | DONE |
| Matchup matrix output | DONE (via validate --matrix) |
| Game replay export (seeds + decisions) | DONE (--export-replays) |

---

### ✅ 6. Diagnose Enhancements [DONE]

| Feature | Status |
|---------|--------|
| Comparative mode (deck A vs deck B side-by-side) | DONE (--deck1 + --deck2) |
| Action breakdown (% attacks vs plays vs abilities) | DONE |
| Critical turn identification | DONE (--critical-turns) |
| Essence (Mana) curve efficiency analysis | DONE (resource efficiency metrics) |

---

### 7. Tune Improvements

| Feature | Status |
|---------|--------|
| Resume from checkpoint | DONE (`--resume <run_id>`) |
| Post-tuning auto-validation | DONE (runs by default, --skip-validation to disable) |
| Multi-objective optimization (win rate + game length) | PARTIAL (game length is minor factor only) |
| Hyperparameter auto-tuning | OPEN |

---

## Remaining Open Items

### Low Priority (Nice to Have)
1. **Hyperparameter auto-tuning** - Adaptive CMA-ES parameters
2. **True multi-objective optimization** - Pareto frontier for win rate vs game length

---

## Recently Completed

- **HTML report generator (Full)** - DONE (2026-01-31)
  - New Python script: `python/scripts/generate_report.py`
  - Unified HTML report with 4 tabs: Overview, Validation, Tuning, ELO
  - Health gauge showing overall balance score
  - Key metrics cards: total games, decks tested, P1/P2 balance, outlier count
  - Interactive Plotly charts: deck win rates, matchup heatmap, faction distribution
  - Tuning tab with training curves (fitness, win rate, sigma), experiment comparison
  - ELO tab with rankings, rating timeline, matchup predictions, match history
  - Aggregated dashboard with `--aggregate` option
  - Batch generation with `--generate-all`, `--since`, `--limit`, `--force`
  - Dark/light theme support (`--theme light`)
  - Sortable tables with confidence intervals
  - Run with: `uv run python python/scripts/generate_report.py --run-id latest`
  - Core module: `python/essence_wars/analysis/report/` (~2000 lines)

- **Tune checkpoint resume** - DONE (2026-01-31)
  - `--resume <run_id>` flag to resume from interrupted runs
  - Checkpoint saved after every generation (~10 KB TOML)
  - Graceful Ctrl+C handling saves checkpoint before exit
  - Partial ID matching (e.g., `--resume baseline` finds `2026-01-31_1430_baseline`)
  - Accumulated elapsed time tracking across sessions
  - Core module: `src/tuning/checkpoint.rs` (~400 lines)

- **Swiss tournament mode** - DONE (2026-01-31)
  - New `swiss` binary for Swiss-system tournaments
  - Score-based pairing (similar scores play each other)
  - Tiebreakers: score, Buchholz (opponent strength), win rate
  - Automatic ELO updates after each match
  - Faction filtering (`--faction argentum`)
  - JSON export (`--output results.json`)
  - Core module: `src/arena/tournament.rs` (~700 lines)

- **ELO rating persistence** - DONE (2026-01-31)
  - Automatic per-deck/commander ELO tracking in arena binary
  - Stored in `data/ratings/deck_elo.json`
  - Brief console summary after each match
  - Full history with opponent info, deltas, and results
  - Standard ELO formula (K=32, initial 1500)

- **Validate → diagnose integration** - DONE (2026-01-31)
  - Added `--auto-diagnose` flag to validate binary
  - Automatically runs diagnostics on outlier decks (win rate <40% or >60%)
  - Configurable threshold via `--outlier-threshold`
  - Runs mirror match + worst matchup diagnostics
  - Includes critical turn analysis
  - Saves reports to `experiments/validation/{run_id}/diagnostics/`

- **Card synergy analysis** - DONE (2026-01-31)
  - Added `--synergy` flag to card_stats binary
  - Tracks card pair co-occurrence and win rates
  - Identifies synergistic pairs and anti-synergy pairs
  - Exports to CSV/JSON via `--synergy-csv` and `--synergy-json`
