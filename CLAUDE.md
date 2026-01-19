# CLAUDE.md - AI Assistant Context for Essence Wars

## Project Overview

**Essence Wars** is a deterministic, perfect-information card game engine designed for AI research (reinforcement learning, MCTS). The engine is written in Rust with a focus on performance and correctness.

**Current Version:** 0.7.0

**Author: Christian Wissmann (Chris), Best Friends with Claude

## Quick Commands

```bash
# Build (from workspace root)
cargo build --release                    # Build all crates
cargo build --release -p cardgame        # Build core engine only
cargo build --release -p essence-wars-3d # Build 3D client only

# Run 3D client (Glassbox AI visualization)
cargo run --release -p essence-wars-3d   # Launch Bevy 3D client

# Run all tests
cargo nextest run --status-level=fail  # ~629 tests (only shows failures)
cargo test                              # Alternative: use standard cargo test

# Run Linter (production code only, excludes tests)
./scripts/run-clippy.sh                 # Recommended: checks lib + binaries
cargo clippy -p cardgame --lib --bins -- -D warnings  # Core engine only
cargo clippy -p essence-wars-3d -- -D warnings        # 3D client only

# Run stress tests by tier
./scripts/run-tests.sh                  # Standard tests only
./scripts/run-tests.sh quick            # + quick tier (~2 min)
./scripts/run-tests.sh medium           # + medium tier (~10 min)
./scripts/run-tests.sh long             # + long tier (~30 min)
./scripts/run-tests.sh overnight        # + overnight tier (~1-2 hours)

# Run arena matches
cargo run --release --bin arena -- --bot1 greedy --bot2 random --games 100
cargo run --release --bin arena -- --bot1 mcts --bot2 greedy --games 10 --progress
cargo run --release --bin arena -- --list-decks

# Run weight tuning (outputs to experiments/mcts/)
cargo run --release --bin tune -- --mode generalist --tag my_run --generations 50
cargo run --release --bin tune -- --mode specialist --deck architect_fortify --opponent broodmother_swarm --tag matchup_test
cargo run --release --bin tune -- --mode faction-specialist --faction argentum --tag argentum_v1

# Analyze tuning results
./scripts/analyze-tuning.sh --latest
./scripts/analyze-tuning.sh --all

# Run balance validation (round-robin: 66 deck matchups × 2 directions × games)
cargo run --release --bin validate -- --games 100              # 13.2k total games - quick check
cargo run --release --bin validate -- --games 500 --output results.json  # 66k total - full validation
cargo run --release --bin validate -- --games 1500             # 198k total games - comprehensive

# Run P1/P2 asymmetry diagnostics
cargo run --release --bin diagnose -- 200                                  # Basic analysis
cargo run --release --bin diagnose -- 500 --export json --output ./results # Export to JSON
cargo run --release --bin diagnose -- 500 --export csv --output ./diag     # Export to CSV
cargo run --release --bin diagnose -- 500 --export all --include-turns     # Full export with per-turn data

# Run benchmarks
cargo bench -p cardgame      # Criterion benchmarks for core engine
./scripts/run-benchmarks.sh  # Full benchmark suite with report

# Modal cloud training (requires Modal CLI: uv tool install modal && modal token new)
modal run modal_tune.py::main                         # Full pipeline (train + validate + auto-deploy)
modal run modal_tune.py::main --mode train-only       # Train only + auto-deploy weights
modal run modal_tune.py::main --mode validate-only    # Validate with existing weights
modal run modal_tune.py::main --no-deploy             # Full pipeline without auto-deploying weights
```

## Project Structure

This project uses a **Cargo workspace** with two crates:

| Crate | Purpose |
|-------|---------|
| `cardgame` | Core game engine, AI bots, tuning infrastructure |
| `essence-wars-3d` | Bevy 3D client with Glassbox AI visualization |

```
ai-cardgame/
├── Cargo.toml                      # Workspace root (shared dependencies)
├── crates/
│   ├── cardgame/                   # Core game engine library
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs              # Module exports
│   │   │   ├── version.rs          # Version info for reproducibility
│   │   │   ├── tensor.rs           # State to tensor conversion (326 floats)
│   │   │   ├── decks.rs            # DeckDefinition, DeckRegistry, Faction enum
│   │   │   ├── core/               # Core game engine
│   │   │   │   ├── mod.rs
│   │   │   │   ├── types.rs        # CardId, PlayerId, Slot, Rarity, Tag
│   │   │   │   ├── keywords.rs     # 14 keywords as u16 bitfield
│   │   │   │   ├── config.rs       # GameConfig with all parameters
│   │   │   │   ├── effects.rs      # Trigger, Effect, EffectTarget, TargetingRule
│   │   │   │   ├── state.rs        # GameState, PlayerState, Creature, Support
│   │   │   │   ├── cards.rs        # CardDefinition, CardDatabase, YAML loading
│   │   │   │   ├── actions.rs      # Action enum with index mapping (256 actions)
│   │   │   │   ├── legal.rs        # Legal action generation
│   │   │   │   ├── combat.rs       # Combat resolution with keyword interactions
│   │   │   │   └── tracing.rs      # Debug tracing infrastructure
│   │   │   ├── engine/             # Game engine implementation
│   │   │   │   ├── mod.rs
│   │   │   │   ├── game_engine.rs  # Core GameEngine, game loop, turn structure
│   │   │   │   ├── effect_queue.rs # Effect queue processing system
│   │   │   │   ├── effect_convert.rs
│   │   │   │   ├── passive.rs      # Passive effect resolution
│   │   │   │   └── environment.rs  # GameEnvironment trait for AI
│   │   │   ├── bots/
│   │   │   │   ├── mod.rs          # Bot trait definition
│   │   │   │   ├── random.rs       # RandomBot - uniform random selection
│   │   │   │   ├── greedy.rs       # GreedyBot - heuristic evaluation
│   │   │   │   ├── mcts.rs         # MctsBot - Monte Carlo Tree Search
│   │   │   │   ├── introspection.rs # AI decision introspection for Glassbox
│   │   │   │   ├── weights.rs      # BotWeights, GreedyWeights (24 params)
│   │   │   │   └── factory.rs      # BotType enum, create_bot(), resolve_weights()
│   │   │   ├── execution/          # Shared parallel execution utilities
│   │   │   │   ├── mod.rs          # Module exports
│   │   │   │   ├── seeds.rs        # GameSeeds for deterministic parallel execution
│   │   │   │   ├── progress.rs     # ProgressReporter with background thread
│   │   │   │   └── parallel.rs     # BatchConfig, run_batch_parallel()
│   │   │   ├── arena/
│   │   │   │   ├── mod.rs          # Arena module exports
│   │   │   │   ├── runner.rs       # GameRunner for executing matches
│   │   │   │   ├── logger.rs       # ActionLogger for debug tracing
│   │   │   │   └── stats.rs        # MatchStats for win rate tracking
│   │   │   ├── tuning/
│   │   │   │   ├── mod.rs          # Tuning module exports
│   │   │   │   ├── cmaes.rs        # CMA-ES optimizer implementation
│   │   │   │   └── evaluator.rs    # Fitness evaluation via game matches
│   │   │   ├── diagnostics/        # P1/P2 asymmetry analysis tools
│   │   │   │   ├── mod.rs          # Module exports
│   │   │   │   ├── collector.rs    # DiagnosticRunner, GameDiagnostics
│   │   │   │   ├── analyzer.rs     # AggregatedStats, BalanceAssessment
│   │   │   │   ├── metrics.rs      # BoardAdvantage, TempoMetrics
│   │   │   │   ├── statistics.rs   # Wilson CI, chi-square, percentiles
│   │   │   │   ├── export.rs       # CSV/JSON export functionality
│   │   │   │   └── report.rs       # Console report printing
│   │   │   └── bin/
│   │   │       ├── arena.rs        # CLI for running bot matches
│   │   │       ├── tune.rs         # CLI for weight optimization
│   │   │       ├── validate.rs     # CLI for balance validation
│   │   │       ├── diagnose.rs     # CLI for P1/P2 asymmetry analysis
│   │   │       ├── profile_mcts.rs # MCTS performance profiling
│   │   │       └── generate_dataset.rs # Generate training datasets
│   │   ├── tests/
│   │   │   ├── common/             # Shared test utilities
│   │   │   ├── unit/               # Unit tests (separate from src)
│   │   │   │   ├── types_tests.rs
│   │   │   │   ├── keywords_tests.rs
│   │   │   │   ├── combat_tests.rs
│   │   │   │   └── ... (23 files total)
│   │   │   ├── unit.rs             # Unit test entry point
│   │   │   ├── engine_tests.rs
│   │   │   ├── stress_mcts_tests.rs
│   │   │   └── ... (12 integration test files)
│   │   └── benches/
│   │       └── game_benchmarks.rs  # Criterion benchmarks
│   │
│   └── essence-wars-3d/            # Bevy 3D client
│       ├── Cargo.toml
│       ├── README.md
│       └── src/
│           ├── main.rs             # Bevy app entry point
│           ├── game/
│           │   ├── mod.rs          # Game module exports
│           │   ├── bridge.rs       # GameBridge resource (wraps GameClient)
│           │   └── state.rs        # AppState enum (Menu, Playing, GameOver)
│           ├── rendering/
│           │   ├── mod.rs
│           │   ├── board.rs        # 3D board with creature slots
│           │   ├── creatures.rs    # Creature3D component and spawning
│           │   ├── camera.rs       # Camera controller
│           │   └── lighting.rs     # Scene lighting
│           ├── ui/
│           │   ├── mod.rs
│           │   ├── hud.rs          # Game HUD (life, essence, turn)
│           │   ├── hand.rs         # Player hand display
│           │   └── menu.rs         # Main menu and game over screens
│           └── glassbox/           # AI visualization (Glassbox mode)
│               ├── mod.rs          # GlassboxState resource and plugin
│               ├── mcts_panel.rs   # MCTS tree visualization
│               ├── action_probs.rs # Action probability bars
│               └── value_gauge.rs  # Value estimate gauge
│
├── data/
│   ├── cards/core_set/             # 300 cards (4 faction files, 75 each)
│   │   ├── argentum.yaml           # IDs 1000-1074 (75 cards)
│   │   ├── symbiote.yaml           # IDs 2000-2074 (75 cards)
│   │   ├── obsidion.yaml           # IDs 3000-3074 (75 cards)
│   │   └── neutral.yaml            # IDs 4000-4074 (75 cards)
│   ├── decks/                      # 12 commander decks (organized by faction)
│   │   ├── argentum/               # 4 decks
│   │   ├── symbiote/               # 4 decks
│   │   └── obsidion/               # 4 decks
│   └── weights/
│       ├── generalist.toml         # Cross-faction weights
│       └── specialists/
│           ├── argentum.toml
│           ├── symbiote.toml
│           └── obsidion.toml
├── experiments/                    # Experiment outputs (GITIGNORED)
│   └── mcts/
│       └── YYYY-MM-DD_HHMM_tag/
│           ├── train.log           # Full training log
│           ├── weights.toml        # Best weights found
│           ├── version.toml        # Engine version for reproducibility
│           ├── summary.txt         # Quick stats
│           └── plots/              # Generated visualizations
├── scripts/
│   ├── run-tests.sh                # Test tier runner
│   ├── run-clippy.sh               # Linter (lib + binaries)
│   ├── run-benchmarks.sh           # Full benchmark suite
│   ├── run-all-tuning.sh           # Train all specialists + generalist
│   ├── analyze-tuning.sh           # Python analysis wrapper
│   ├── balance-test.sh             # Faction balance verification
│   └── perf-stats.sh               # Performance statistics
├── python/
│   ├── cardgame/
│   │   ├── infra/                  # Experiment management
│   │   └── analysis/               # Visualization & stats tools
│   │       ├── parse_log.py
│   │       └── visualize.py
│   └── scripts/
│       └── analyze_tuning.py       # Main analysis CLI
├── docs/
│   ├── essence-wars-design.md      # Game design document
│   ├── design-engine.md            # Engine architecture
│   ├── jrpg-architecture.md        # Bevy 3D client architecture
│   ├── bevy-glassbox-implementation.md  # Glassbox AI visualization
│   ├── workspace-migration.md      # Cargo workspace setup
│   ├── development-setup.md        # Developer getting started
│   ├── cards-new-horizons.md       # 300-card expansion details
│   ├── tuning-pipeline.md          # Weight optimization guide
│   ├── modal-cloud-setup.md        # Cloud training guide
│   ├── game-modes.md               # Game mode documentation
│   ├── data-strategy.md            # Data organization
│   ├── performance.md              # Performance optimization
│   └── lore.md                     # Faction lore and worldbuilding
└── .github/workflows/
    ├── ci.yml                      # Every push: tests
    ├── nightly.yml                 # Daily: stress tests
    └── weekly.yml                  # Sunday: exhaustive tests
```

## Test Organization

**IMPORTANT**: This project keeps unit tests **separate from source code**, not inline with `#[cfg(test)] mod tests` blocks. This reduces token usage when AI assistants read source files.

- **Unit tests**: `crates/cardgame/tests/unit/<module>_tests.rs`
- **Integration tests**: `crates/cardgame/tests/*.rs`
- **Shared utilities**: `crates/cardgame/tests/common/mod.rs`

When adding new tests, create them in `crates/cardgame/tests/unit/` and add the module to `crates/cardgame/tests/unit.rs`.

## Test Tiers & CI/CD

Tests are organized into tiers by runtime:

| Tier | Duration | When Run | Examples |
|------|----------|----------|----------|
| **Standard** | ~2 min | Every commit | Unit tests, integration tests |
| **tier_quick** | ~2 min | Every PR | 20-50 game MCTS tests |
| **tier_medium** | ~10 min | Nightly | 100 game bot validation |
| **tier_long** | ~30 min | Nightly | 500 game stress tests |
| **tier_overnight** | ~1-2 hours | Weekly | 100k game exhaustive tests |

### GitHub Actions Workflows

| Workflow | Trigger | What Runs |
|----------|---------|-----------|
| **ci.yml** | Every push/PR | Standard tests only |
| **nightly.yml** | Daily 2 AM UTC | Quick + medium + long tiers |
| **weekly.yml** | Sunday 3 AM UTC | All tiers including overnight |

**Note:** Clippy and rustfmt are run manually via scripts, not in CI.

### Running Tests Locally

```bash
./scripts/run-tests.sh              # Standard tests (~2 min)
./scripts/run-tests.sh quick        # + quick tier
./scripts/run-tests.sh medium       # + medium tier
./scripts/run-tests.sh long         # + long tier
./scripts/run-tests.sh overnight    # Full suite (~2 hours)
```

## Bot System

### Bot Types

| Bot | Description | Strength |
|-----|-------------|----------|
| RandomBot | Uniform random action selection | Baseline |
| GreedyBot | Simulates each action, picks best by heuristic | Beats Random 100% |
| MctsBot | UCB1 tree search with GreedyBot rollouts | Beats Greedy 60-100% |

### Agent Types (MCTS with tuned weights)

| Bot Type | Weights | Usage |
|----------|---------|-------|
| `agent-argentum` | `data/weights/specialists/argentum.toml` | Argentum faction decks |
| `agent-symbiote` | `data/weights/specialists/symbiote.toml` | Symbiote faction decks |
| `agent-obsidion` | `data/weights/specialists/obsidion.toml` | Obsidion faction decks |
| `agent-generalist` | `data/weights/generalist.toml` | Any deck |

### Bot Trait

```rust
pub trait Bot: Send {
    fn name(&self) -> &str;
    fn select_action(
        &mut self,
        state_tensor: &[f32; 326],
        legal_mask: &[f32; 256],
        legal_actions: &[Action],
    ) -> Action;
    fn reset(&mut self);
    fn clone_box(&self) -> Box<dyn Bot>;
}
```

### GreedyBot Weights (24 parameters)

| Category | Parameters |
|----------|------------|
| Life | own_life, enemy_life_damage |
| Creatures | own_creature_attack, own_creature_health, enemy_creature_attack, enemy_creature_health |
| Board | creature_count, board_advantage |
| Resources | cards_in_hand, action_points |
| Keywords | guard, lethal, lifesteal, rush, ranged, piercing, shield, quick |
| Terminal | win_bonus, lose_penalty |

### Bot Introspection (AI Transparency)

The `cardgame::bots` module provides introspection types for AI decision transparency, used by the Bevy 3D client's Glassbox visualization.

**Location**: `crates/cardgame/src/bots/introspection.rs`

| Type | Purpose |
|------|---------|
| `MctsNodeStats` | Per-action statistics (action, visits, win_rate) |
| `MctsTreeSnapshot` | Complete MCTS state at decision time |
| `BotDecision` | Full decision record with optional MCTS snapshot |
| `IntrospectionConfig` | Configuration for what data to collect |

**Quick Usage:**

```rust
use cardgame::bots::{MctsBot, IntrospectionConfig};

let bot = MctsBot::new(1000).with_introspection(IntrospectionConfig::full());
// After bot makes decision:
if let Some(decision) = bot.last_decision() {
    if let Some(snapshot) = &decision.mcts_snapshot {
        println!("Simulations: {}", snapshot.total_simulations);
        println!("Selected: {:?}", snapshot.selected_action);
    }
}
```

**Glassbox Integration**: The Bevy 3D client accesses these types via `GameBridge.last_decision` to render real-time AI visualization panels (MCTS tree, action probabilities, value gauge).

**Full Documentation**: See `docs/jrpg-architecture.md` Section 3.2 and Section 4 for comprehensive Glassbox Mode details.

## Weight Tuning Pipeline

### CMA-ES Optimizer

The tuning system uses CMA-ES (Covariance Matrix Adaptation Evolution Strategy) to optimize GreedyBot/MCTS weights.

- **Parallel by default**: Uses all CPU cores (~14x speedup on 16-core machines)
- **Experiments saved to**: `experiments/mcts/YYYY-MM-DD_HHMM_tag/`
- **Auto-deployment**: Best weights copied to `data/weights/` on completion

### Tuning Modes (3 Primary)

| Mode | Description | Use Case |
|------|-------------|----------|
| `generalist` | Optimize across all deck matchups vs Random/Greedy/MCTS | Default, most robust |
| `specialist` | Optimize for specific deck vs opponent | Fine-tuning matchups |
| `faction-specialist` | Train specialist for a faction | Faction-specific agents |

### Tune CLI

```bash
# Generalist training (default mode)
cargo run --release --bin tune -- \
  --mode generalist \
  --tag generalist_v1 \
  --generations 100 \
  --games 100

# Specialist for specific matchup
cargo run --release --bin tune -- \
  --mode specialist \
  --deck architect_fortify \
  --opponent broodmother_swarm \
  --tag argentum_vs_symbiote \
  --generations 50

# Faction specialist (auto-saves to data/weights/specialists/)
cargo run --release --bin tune -- \
  --mode faction-specialist \
  --faction argentum \
  --tag argentum_v1 \
  --generations 100

# Full options
cargo run --release --bin tune -- \
  --mode generalist \
  --generations 100 \
  --population 20 \
  --games 100 \
  --sigma 0.3 \
  --min_sigma 0.001 \
  --mcts_sims 50 \
  --target_win_rate 0.95 \
  --seed 12345 \
  --initial_weights existing.toml \
  --tag my_experiment \
  --verbose
```

### Analysis Pipeline

```bash
# Analyze latest experiment (generates plots and REPORT.md)
./scripts/analyze-tuning.sh --latest

# Analyze specific experiment
./scripts/analyze-tuning.sh experiments/mcts/2026-01-14_0755_generalist-v0.4

# Analyze all experiments
./scripts/analyze-tuning.sh --all
```

Output includes:
- `plots/overview.png` - 4-panel dashboard (fitness, win rate, sigma, improvement)
- `plots/efficiency.png` - Time analysis
- `plots/convergence.png` - Optimization dynamics
- `plots/REPORT.md` - Comprehensive markdown report with insights

## Modal Cloud Training

For cloud-based tuning runs, see `docs/modal-cloud-setup.md`. This enables running expensive tuning jobs on Modal.com infrastructure with 4x parallel speedup.

```bash
# Full pipeline: train all 4 configs + validate + auto-deploy weights
modal run modal_tune.py

# Train only (skip validation)
modal run modal_tune.py --mode train-only

# Validate only (with existing weights on Modal volume)
modal run modal_tune.py --mode validate-only

# Single configuration
modal run modal_tune.py --single generalist

# Skip auto-deploy to local repo
modal run modal_tune.py --no-deploy
```

**Note:** Training parameters (generations, games, mcts-sims) are configured in `TRAINING_CONFIGS` within `modal_tune.py`, not via CLI flags.

### Local vs Cloud: When to Use Each

| Scenario | Recommendation | Command |
|----------|----------------|---------|
| Quick experiment, single config | **Local** | `cargo run --release --bin tune -- --mode generalist` |
| Training all 4 specialists | **Cloud** | `modal run modal_tune.py` |
| Balance validation only | **Either** | Local: `cargo run --release --bin validate` / Cloud: `--mode validate-only` |
| Hyperparameter tuning | **Cloud** | Edit `TRAINING_CONFIGS`, run parallel |

**Setup:** `uv tool install modal && modal token new` (one-time, 2 minutes)

## Deck System

### Available Decks (12 Commander Decks)

**Argentum Combine (4 decks):**
| Deck ID | Name | Commander | Archetype |
|---------|------|-----------|-----------|
| `architect_fortify` | Architect's Bastion | Architect Prime | Guard/Fortify |
| `artificer_tokens` | Artificer's Workshop | Master Artificer | Token swarm |
| `colossus_wall` | Colossus Defense | Iron Colossus | Wall/Defense |
| `vex_piercing` | Vex's Assault | Vex, the Piercer | Piercing aggro |

**Symbiote Circles (4 decks):**
| Deck ID | Name | Commander | Archetype |
|---------|------|-----------|-----------|
| `alpha_frenzy` | Alpha's Hunt | Alpha Predator | Frenzy aggro |
| `broodmother_swarm` | Broodmother's Hive | Broodmother | Swarm/Rush |
| `grove_regenerate` | Grove's Embrace | Grove Warden | Regenerate/Sustain |
| `plague_volatile` | Plague Bearers | Plague Carrier | Volatile/Damage |

**Obsidion Syndicate (4 decks):**
| Deck ID | Name | Commander | Archetype |
|---------|------|-----------|-----------|
| `archon_burst` | Archon's Gambit | Shadow Archon | Burst damage |
| `kael_assassin` | Kael's Blades | Kael, Shadow Blade | Lethal/Assassin |
| `shadow_weaver` | Shadow Weaver | Shadow Weaver | Control/Stealth |
| `sovereign_lifesteal` | Sovereign's Reign | Sovereign | Lifesteal/Sustain |

### Deck Organization

Decks are organized by faction in subdirectories:
```
data/decks/
├── argentum/
│   ├── architect_fortify.toml
│   ├── artificer_tokens.toml
│   ├── colossus_wall.toml
│   └── vex_piercing.toml
├── symbiote/
│   ├── alpha_frenzy.toml
│   ├── broodmother_swarm.toml
│   ├── grove_regenerate.toml
│   └── plague_volatile.toml
└── obsidion/
    ├── archon_burst.toml
    ├── kael_assassin.toml
    ├── shadow_weaver.toml
    └── sovereign_lifesteal.toml
```

### TOML Format

```toml
id = "architect_fortify"
name = "Architect's Bastion"
description = "Defensive deck featuring Architect Prime and fortify synergies."
commander = 1060  # Architect Prime
tags = ["control", "defensive", "fortify", "argentum"]

cards = [
    1060, 1001, 1002, # Commander + core cards
    # ... 30 card entries total (21 faction + 9 neutral)
]
```

### Arena CLI

```bash
# List available decks
cargo run --release --bin arena -- --list-decks

# Run match with specific decks
cargo run --release --bin arena -- \
  --deck1 architect_fortify --deck2 broodmother_swarm \
  --bot1 mcts --bot2 greedy \
  --games 100 --progress

# Agent specialist matches
cargo run --release --bin arena -- \
  --bot1 agent-argentum --deck1 colossus_wall \
  --bot2 agent-symbiote --deck2 alpha_frenzy \
  --games 100 --progress

# Debug mode (sequential, with logging)
cargo run --release --bin arena -- \
  --bot1 greedy --bot2 mcts \
  --games 10 --debug --log-file arena.log
```

### Diagnostics CLI

The `diagnose` binary analyzes P1/P2 asymmetry with statistical rigor:

```bash
# Basic analysis (200 games)
cargo run --release --bin diagnose -- 200

# Export to JSON with full game metadata
cargo run --release --bin diagnose -- 500 --export json --output ./results.json

# Export to CSV (creates aggregate_stats.csv and game_metadata.csv)
cargo run --release --bin diagnose -- 500 --export csv --output ./diagnostics

# Export all formats with per-turn data (can be large)
cargo run --release --bin diagnose -- 500 --export all --include-turns --output ./full_export

# Use specific deck and seed for reproducibility
cargo run --release --bin diagnose -- 500 --deck architect_fortify --seed 12345
```

**Output includes:**
- Win rates with 95% Wilson confidence intervals
- Chi-square significance testing (★★★ highly significant, ★★ significant, ★ marginal)
- Tempo metrics (first creature, first blood timing)
- Board advantage tracking (turns ahead/behind)
- Resource efficiency (essence spent vs board impact)
- Combat efficiency (trade ratios, face damage)
- Game length percentiles (P10, P50, P90)
- Balance assessment (Balanced, P1/P2 Favored, etc.)

## Faction System

### True Factions (3)

| Faction | Identity | Primary Keywords | Playstyle |
|---------|----------|------------------|-----------|
| **Argentum Combine** | "The Wall" | Guard, Piercing, Shield | Defensive, high-HP, outlast |
| **Symbiote Circles** | "The Swarm" | Rush, Lethal, Regenerate | Aggressive tempo, efficient trades |
| **Obsidion Syndicate** | "The Shadow" | Lifesteal, Stealth, Ephemeral, Quick | Burst damage, life manipulation |

### Neutral Cards

| Category | Identity | Primary Keywords | Role |
|----------|----------|------------------|------|
| **Free-Walkers** | "The Toolbox" | Ranged, Charge | Utility splash for any faction |

Free-Walkers are **not a standalone faction**—they are neutral utility cards that can be splashed into any faction deck.

### Deck Composition

```
STANDARD DECK: 30 cards
├── Faction Core: 21 cards (70%)    ← Primary faction identity
└── Neutral Splash: 9 cards (30%)   ← Free-Walker utility
```

## Performance

```bash
cargo bench -p cardgame        # Criterion benchmarks for core engine
./scripts/run-benchmarks.sh    # Full suite with report
```

| Benchmark | Time | Throughput |
|-----------|------|------------|
| Random game | ~12 µs | ~80,000 games/sec |
| Greedy game | ~58 µs | ~17,000 games/sec |
| State tensor | ~138 ns | - |
| Legal actions | ~29 ns | - |
| Engine fork | ~99 ns | - |

### Key Optimizations

- **ArrayVec** for fixed-size collections - stack allocation, fast cloning for MCTS
- **u8 bitfield** for keywords - compact representation
- **Indexed action space** (256 actions) - fixed-size for neural networks
- **Engine fork()** - efficient state cloning for tree search
- **Rayon** for parallel game evaluation in tuning

## Key Design Decisions

### Game Rules
- 5 creature slots, 2 support slots per player
- 3 Action Points per turn
- 30 turn limit with life-based tiebreaker
- 14 keywords: Rush, Ranged, Piercing, Guard, Lifesteal, Lethal, Shield, Quick, Ephemeral, Regenerate, Stealth, Charge, Frenzy, Volatile

### Game Modes
| Mode | Win Condition | Training Status |
|------|---------------|-----------------|
| **Attrition** (default) | 0 life OR turn 30 → higher life | Trained (MCTS weights) |
| **Essence Duel** | 50 VP (face damage) OR 0 life | Experimental |

```bash
# Default mode (Attrition)
cargo run --release --bin arena -- --bot1 mcts --bot2 greedy --games 100

# Essence Duel mode
cargo run --release --bin arena -- --bot1 mcts --bot2 greedy --games 100 --mode essence-duel
```

### AI Interface (GameEnvironment trait)
- `get_state_tensor()` - 326 floats representing full game state
- `get_legal_action_mask()` - 256 floats (0.0 or 1.0)
- `apply_action_by_index(u8)` - apply action from neural network output
- `get_reward(PlayerId)` - -1.0, 0.0, or 1.0
- `fork()` - clone game state for MCTS tree search

## Card System

### Card Counts
- **Total**: 300 cards (New Horizons Edition)
- **Argentum Combine**: 75 cards (IDs 1000-1074)
- **Symbiote Circles**: 75 cards (IDs 2000-2074)
- **Obsidion Syndicate**: 75 cards (IDs 3000-3074)
- **Free-Walkers (Neutral)**: 75 cards (IDs 4000-4074)

### Card ID Ranges

| Faction | ID Range | Current | Reserved For |
|---------|----------|---------|--------------|
| Argentum | 1000-1999 | 1000-1074 | Future expansion |
| Symbiote | 2000-2999 | 2000-2074 | Future expansion |
| Obsidion | 3000-3999 | 3000-3074 | Future expansion |
| Neutral | 4000-4999 | 4000-4074 | Future expansion |

### YAML Schema

```yaml
# Creature
- id: 1000
  name: "Brass Sentinel"
  cost: 2
  card_type: creature
  attack: 2
  health: 4
  keywords: [Guard]
  rarity: Common
  tags: [Construct]
  abilities:
    - trigger: OnPlay
      targeting: TargetEnemyCreature
      effects:
        - type: damage
          amount: 2

# Spell
- id: 1010
  name: "Reinforce"
  cost: 2
  card_type: spell
  targeting: TargetAllyCreature
  effects:
    - type: buff_stats
      attack: 0
      health: 3

# Support
- id: 1013
  name: "Assembly Line"
  cost: 4
  card_type: support
  durability: 3
  triggered_effects:
    - trigger: StartOfTurn
      effects:
        - type: heal
          amount: 2
  passive_effects:
    - modifier:
        attack_bonus: 1

# Phase 4 Features: Filters, Conditionals, Bounce

# Filtered Effect (destroy creature with ≤3 health)
- id: 1025
  name: "Execute"
  cost: 2
  card_type: spell
  targeting: TargetEnemyCreature
  effects:
    - type: destroy
      filter:
        max_health: 3

# Conditional Effect (if target dies, draw a card)
- id: 2035
  name: "Soul Reaper"
  cost: 3
  card_type: spell
  targeting: TargetEnemyCreature
  effects:
    - type: damage
      amount: 3
  conditional_effects:
    - condition: target_died
      effects:
        - type: draw
          count: 1

# Bounce Effect (return creature to hand)
- id: 1029
  name: "Temporal Displacement"
  cost: 3
  card_type: spell
  targeting: TargetEnemyCreature
  effects:
    - type: bounce

# Keyword Filter (buff all Rush creatures)
- id: 2039
  name: "Evolution Burst"
  cost: 3
  card_type: spell
  targeting: NoTarget
  effects:
    - type: buff_stats
      attack: 2
      health: 1
      filter:
        has_keyword: 8  # Rush keyword bit
```

### Filter Fields
| Field | Type | Description |
|-------|------|-------------|
| `max_health` | u8 | Target must have health ≤ value |
| `min_health` | u8 | Target must have health ≥ value |
| `has_keyword` | u16 | Target must have keyword (bit value) |
| `lacks_keyword` | u16 | Target must NOT have keyword |

### Keyword Bit Values
| Keyword | Bit | Keyword | Bit |
|---------|-----|---------|-----|
| Guard | 1 | Quick | 128 |
| Lethal | 2 | Ephemeral | 256 |
| Lifesteal | 4 | Regenerate | 512 |
| Rush | 8 | Stealth | 1024 |
| Ranged | 16 | Charge | 2048 |
| Piercing | 32 | Frenzy | 4096 |
| Shield | 64 | Volatile | 8192 |

## State Tensor Layout (326 floats)

| Section | Size | Description |
|---------|------|-------------|
| Global | 10 | turn, phase, active_player, game_over, winner |
| Player 1 creatures | 60 | 5 slots × 12 floats |
| Player 2 creatures | 60 | 5 slots × 12 floats |
| Player 1 supports | 8 | 2 slots × 4 floats |
| Player 2 supports | 8 | 2 slots × 4 floats |
| Player 1 hand | 60 | 20 cards × 3 floats |
| Player 2 hand | 60 | 20 cards × 3 floats |
| Player 1 deck | 30 | 30 card IDs |
| Player 2 deck | 30 | 30 card IDs |

## Action Space (256 indices)

| Range | Action Type |
|-------|-------------|
| 0-99 | PlayCard (hand 0-9 × slot 0-9) |
| 100-149 | Attack (slot 0-4 × target 0-9) |
| 150-249 | UseAbility (slot 0-4 × ability 0-1 × target 0-9) |
| 255 | EndTurn |

## Implementation Status

**Complete:**
- Core game engine with 14 keywords
- Essence/mana system (grows +1/turn, caps at 10)
- AI interface (tensor, action mask, rewards)
- **300-card New Horizons Edition** (75 cards × 4 factions)
- Faction system (3 factions + neutrals)
- Bot system (RandomBot, GreedyBot, MctsBot)
- Arena CLI with parallel execution and progress indicator
- **12 commander decks** (4 per faction with unique commanders)
- Weight tuning pipeline with CMA-ES optimizer
- Analysis pipeline with visualizations
- P1/P2 asymmetry diagnostics with statistical analysis
- CSV/JSON export for external analysis tools
- Version tracking for ML reproducibility
- Phase 4 engine: Creature filters, conditional triggers, bounce effect
- Criterion benchmarks
- CI/CD with GitHub Actions (nightly + weekly)
- Cargo workspace migration (cardgame + essence-wars-3d crates)
- Replay system for game recording/playback
- **Bevy 3D client with Glassbox AI visualization:**
  - 3D board rendering with creature slots
  - egui-based UI (HUD, hand display, menus)
  - MCTS tree panel showing AI decision process
  - Action probability bars and value gauge
  - Press 'G' to toggle glassbox panels
- **Python/ML infrastructure** (published to PyPI as `essence-wars`):
  - PyO3 bindings with PyGame and PyParallelGames classes
  - Gymnasium v26+ environment with action masking
  - PettingZoo multi-agent environment
  - PPO and AlphaZero neural network trainers
  - Dataset generation and PyTorch DataLoader integration
  - Benchmark framework with Elo ratings
  - ~268k steps/sec vectorized throughput
- ~629 tests passing (Rust) + comprehensive Python test suite

**Current Focus:**
- Modal cloud training pipeline optimization
- Bevy 3D client enhancements (creature animations, card rendering)
- ML trainer integration testing (end-to-end validation)

**ML/AI Infrastructure (Complete):**
- **Python bindings (PyO3)**: `PyGame` and `PyParallelGames` classes, published to PyPI as `essence-wars`
- **Gymnasium environment**: v26+ API compliant with action masking, vectorized environments (~268k steps/sec)
- **PettingZoo multi-agent**: Two-player turn-based environment for MARL algorithms
- **PPO trainer**: Policy-value network with action masking, self-play support
- **AlphaZero trainer**: Residual network with MCTS integration
- **Dataset infrastructure**: JSONL format, gzip compression, PyTorch DataLoader ready
- **Benchmark framework**: Elo ratings, cross-faction evaluation, baseline comparisons
- **HuggingFace Hub**: Model/dataset upload and download utilities

**Future Work:**
- Network multiplayer for 3D client
- Story mode and overworld (Phase 5B)
- ML training cookbook/examples documentation

## Bevy 3D Client

The `essence-wars-3d` crate provides a 3D visualization client built with Bevy 0.15.

### Running the Client

```bash
cargo run --release -p essence-wars-3d
```

**Requirements:**
- Linux/WSL: X11 or Wayland display server (WSLg works)
- Graphics drivers with OpenGL/Vulkan support

### Glassbox AI Visualization

Press **'G'** to toggle Glassbox mode, which shows AI decision-making in real-time:

| Panel | Location | Shows |
|-------|----------|-------|
| **MCTS Tree** | Right sidebar | Top actions by visit count, win rates |
| **Action Probs** | Bottom-left | Win rate bars for top 5 actions |
| **Value Gauge** | Top-left | Position evaluation (-1.0 to +1.0) |

### Architecture

The client uses the `GameBridge` resource to wrap the cardgame engine:

```rust
// GameBridge provides access to game state and AI decisions
pub struct GameBridge {
    pub client: Option<GameClient>,
    pub card_db: Arc<CardDatabase>,
    pub deck_registry: Arc<DeckRegistry>,
    pub last_decision: Option<BotDecision>,  // For glassbox visualization
}
```

See `crates/essence-wars-3d/README.md` for full documentation.

## Python/ML Infrastructure

The project includes a complete Python ML training infrastructure, published to PyPI as `essence-wars`.

### Installation

```bash
# From PyPI (recommended)
pip install essence-wars

# With training dependencies
pip install essence-wars[train]

# Development install
maturin develop
```

### Core Python Classes

**PyGame** - Single game environment:
```python
from essence_wars import PyGame

game = PyGame(deck1="architect_fortify", deck2="broodmother_swarm")
game.reset(seed=42)

obs = game.observe()           # (326,) state tensor
mask = game.action_mask()      # (256,) legal action mask
reward, done = game.step(action)
```

**PyParallelGames** - Vectorized environments (~268k steps/sec):
```python
from essence_wars import PyParallelGames

envs = PyParallelGames(num_envs=64, deck1="architect_fortify", deck2="broodmother_swarm")
envs.reset(seeds=list(range(64)))

obs_batch = envs.observe_batch()      # (64, 326)
mask_batch = envs.action_mask_batch() # (64, 256)
rewards, dones = envs.step_batch(actions)
```

### Gymnasium Environment

```python
from essence_wars import EssenceWarsEnv

env = EssenceWarsEnv(opponent="greedy")
obs, info = env.reset(seed=42)
obs, reward, terminated, truncated, info = env.step(action)

# Action masking for SB3
mask = env.action_masks()  # or info["action_mask"]
```

### PettingZoo Multi-Agent

```python
from essence_wars import parallel_env

env = parallel_env()
observations, infos = env.reset(seed=42)
# Two-player turn-based: only active player has legal actions
```

### Neural Network Agents

```python
from essence_wars.agents import EssenceWarsNetwork, AlphaZeroNetwork

# PPO policy-value network
ppo_net = EssenceWarsNetwork(hidden_dim=256)
logits, value = ppo_net(obs_tensor, mask_tensor)

# AlphaZero with residual blocks
az_net = AlphaZeroNetwork(hidden_dim=256, num_residual=4)
policy, value = az_net(obs_tensor)
```

### Dataset Generation

```bash
# Generate MCTS self-play data
cargo run --release --bin generate_dataset -- \
  --games 1000 --sims 100 --output data.jsonl.gz
```

```python
from essence_wars.data import MCTSDataset
from torch.utils.data import DataLoader

dataset = MCTSDataset("data.jsonl.gz")
loader = DataLoader(dataset, batch_size=256, shuffle=True)
```

### Package Structure
```
python/essence_wars/
├── __init__.py          # Main exports (PyGame, PyParallelGames)
├── _core.so             # Compiled Rust bindings
├── env.py               # Gymnasium environment
├── parallel_env.py      # PettingZoo multi-agent
├── agents/
│   ├── networks.py      # Neural network architectures
│   ├── ppo.py           # PPO trainer
│   └── alphazero.py     # AlphaZero trainer
├── data/
│   └── dataset.py       # MCTSDataset for PyTorch
├── benchmark/
│   └── framework.py     # Evaluation benchmarks
├── hub.py               # HuggingFace Hub integration
└── analysis/            # Visualization dashboards
```

## Python Tooling

### Setup with uv
```bash
uv sync --all-groups
uv run pytest python/tests -v
uv run ruff check python/
uv run mypy python/
```

## Data & Experiment Strategy

### Directory Structure
```
ai-cardgame/
├── experiments/           # ALL run artifacts (GITIGNORED)
│   └── mcts/             # Weight tuning runs
├── data/                  # Configuration and weights
│   ├── cards/core_set/   # Card definitions (YAML) - organized by faction
│   ├── decks/            # Deck definitions (TOML) - organized by faction
│   └── weights/          # Tuned weight files
└── docs/                  # Documentation (committed)
```

### Experiment ID Convention
Every run gets: `{YYYY-MM-DD_HHMM}_{tag}`

Example: `2026-01-14_0755_generalist-v0.4`

### Rules for Claude
- **ALWAYS** create experiment folders for training/tuning runs
- **NEVER** commit experiments/ to git
- **USE** timestamped tags for reproducibility

## Versioning

Version is in the **root `Cargo.toml`** under `[workspace.package]`. Both crates use `version.workspace = true` to share the same version. Uses Semantic Versioning: `MAJOR.MINOR.PATCH`

| Change Type | Version Bump | Examples |
|-------------|--------------|----------|
| Game rules/mechanics change | MINOR | Combat changes, essence system |
| API breaking changes | MINOR (pre-1.0) | Tensor layout, action space |
| New features | MINOR | New bot type, new effects |
| Bug fixes | PATCH | Fix crash, fix test |
| Refactoring | PATCH | Code cleanup, optimization |

### Version Update Checklist

1. Update root `Cargo.toml` `[workspace.package]`: `version = "X.Y.Z"`
2. Update `crates/cardgame/src/version.rs`: Update test assertion
3. Update `CHANGELOG.md`: Add entry
4. Run tests: `cargo nextest run --status-level=fail`

Note: Both `cardgame` and `essence-wars-3d` crates share the workspace version automatically.

### Reproducibility

Experiments save `version.toml` with engine version + git hash:

```rust
use cardgame::version::{self, VersionInfo};

println!("Engine: {}", version::version_string());
// Output: "cardgame v0.7.0 (git_hash)"
```
