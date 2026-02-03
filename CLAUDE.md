# CLAUDE.md - AI Assistant Context for Essence Wars

## Project Overview

**Essence Wars** is a deterministic, perfect-information card game engine. Written in Rust with focus on performance and correctness.

**Current Version:** 0.8.0 (Commander Edition) | **Author:** Christian Wissmann (Chris), Best Friends with Claude

## Quick Commands

```bash
# Build
cargo build --release                    # Full workspace
cargo build --release -p cardgame        # Core engine only

# Test
cargo nextest run --status-level=fail    # ~668 tests (recommended)
uv run pytest python/tests

# Lint and Type Checking
./scripts/run-clippy.sh                  # or cargo clippy
uv run mypy python/essence_wars
uv run ruff check python/essence_wars --fix
pnpm run check # For Tauri/Svelte Modules
pnpm run lint # In crates/essence-wars-ui crate

# Stress tests by tier
./scripts/run-tests.sh quick|medium|long|overnight

# Arena matches
cargo run --release --bin arena -- --list-decks

# Weight tuning
./scripts/tune-archetypes.sh                    # Tune all archetypes
./scripts/tune-archetypes.sh aggro tempo        # Tune specific archetypes
./scripts/tune-archetypes.sh --dry-run          # Preview commands
cargo run --release --bin tune -- --mode generalist --tag my_run --generations 50

# Quick balance validation (~1 sec, greedy bot, for CI and sanity checks)
cargo run --release --bin validate -- --progress
cargo run --release --bin validate -- --games-per-matchup 50 --progress

# Thorough benchmark (overnight runs with strong bots)
cargo run --release --bin benchmark -- --progress                    # Alpha-Beta depth 6
cargo run --release --bin benchmark -- --bot mcts --mcts-sims 200    # MCTS instead

# P1/P2 asymmetry analysis
cargo run --release --bin diagnose -- 200

# Swiss tournaments (deck rankings with ELO tracking)
cargo run --release --bin swiss -- --games 20 --progress
cargo run --release --bin swiss -- --faction symbiote --games 30 --output results.json

# Game replay (debugging, analysis)
cargo run --release --bin replay -- --seed 12345 --deck1 broodmother_pack --deck2 architect_fortify
cargo run --release --bin replay -- --file game.replay.json.gz --interactive
cargo run --release --bin replay -- --seed 12345 --deck1 X --deck2 Y --export transcript -o game.txt

# Benchmarks
cargo bench -p cardgame

# Enable logging (optional)
RUST_LOG=info cargo run --release --bin arena -- --bot1 greedy --bot2 random --games 10
RUST_LOG=debug cargo run --release --bin benchmark -- -n 10 --progress  # Verbose benchmark
```

## Python CLI

The `essence-wars` command provides a unified interface for all Python ML tools.

```bash
# Install CLI dependencies
pip install essence-wars[analysis]  # Or: uv sync --group analysis

# View available commands
essence-wars --help

# Training commands
essence-wars train ppo --timesteps 500000
essence-wars train alphazero --iterations 100
essence-wars train behavioral-cloning --data data.jsonl.gz
essence-wars train card2vec --data data.jsonl.gz --embed-dim 64
essence-wars train decision-transformer --data data.jsonl.gz

# Agent benchmarking
essence-wars benchmark --checkpoint model.pt
essence-wars benchmark --checkpoint model.pt --full-eval

# Report generation
essence-wars report generate --run-id latest --open
essence-wars report generate-all --force
essence-wars report aggregate --open
essence-wars report leaderboard

# Data generation
essence-wars data generate-distillation --games 10000 --output data.jsonl.gz
```

## Logging

The library uses `log` crate for diagnostic messages (weight loading, errors). CLIs initialize `env_logger` automatically.

```bash
# Log levels: error, warn, info, debug, trace
RUST_LOG=info cargo run --release --bin arena -- ...      # See weight loading info
RUST_LOG=warn cargo run --release --bin benchmark -- ...  # Only warnings/errors
RUST_LOG=debug cargo run --release --bin tune -- ...      # Verbose debugging
```

## Project Structure

**Cargo workspace** with the core engine crate:

| Crate | Purpose |
|-------|---------|
| `cardgame` | Core game engine, AI bots, tuning infrastructure |

**Key locations:**
- `crates/cardgame/src/` - Core engine source (~25k lines)
- `crates/cardgame/src/core/` - Game types, state, actions, combat
- `crates/cardgame/src/engine/` - GameEngine, effect processing
- `crates/cardgame/src/bots/` - RandomBot, GreedyBot, MctsBot
- `crates/cardgame/src/bin/` - CLIs: arena, swiss, tune, validate, benchmark, diagnose, replay
- `crates/cardgame/tests/unit/` - Unit tests (separate from src)
- `python/essence_wars/` - Python bindings and ML agents
- `python/essence_wars/agents/` - PPO, AlphaZero, Card2Vec, embeddings
- `python/essence_wars/analysis/report/` - Unified HTML report generator (tabbed reports)
- `python/essence_wars/ratings/` - Unified ratings layer (deck + agent ELO)
- `python/essence_wars/training/` - Training utilities and callbacks
- `python/essence_wars/data/` - Dataset loaders (MCTSDataset, ChunkedMCTSDataset)
- `python/scripts/` - ML scripts organized by category:
  - `training/` - Model training (ppo.py, alphazero.py, etc.)
  - `evaluation/` - Model evaluation and benchmarks
  - `data/` - Dataset generation
  - `analysis/` - Training analysis and diagnostics
  - `reporting/` - HTML report generation
- `data/cards/core_set/` - 300 cards in 4 YAML files (by faction)
- `data/commanders/` - 12 commanders in 3 YAML files (by faction)
- `data/decks/{argentum,symbiote,obsidion}/` - 12 commander decks
- `data/datasets/` - MCTS game datasets (JSONL.gz)
- `data/weights/` - Tuned bot weights (generalist + archetypes)
- `models/` - Trained ML models (Card2Vec, BC, PPO checkpoints)
- `experiments/` - Training outputs (GITIGNORED)
- `docs/` - Design docs, guides

## Test Organization

**IMPORTANT**: Unit tests are **separate from source code** (not inline `#[cfg(test)]` blocks).

- **Unit tests**: `crates/cardgame/tests/unit/<module>_tests.rs`
- **Integration tests**: `crates/cardgame/tests/*.rs`
- **Shared utilities**: `crates/cardgame/tests/common/mod.rs`

When adding tests, create in `tests/unit/` and add module to `tests/unit.rs`.

## Bot System

| Bot | Description |
|-----|-------------|
| `random` | Uniform random selection |
| `greedy` | Heuristic evaluation (28 tunable weights) |
| `mcts` | UCB1 tree search with greedy rollouts |
| `alphabeta` | Alpha-Beta minimax search (depth 6-8 recommended) |
| `agent-generalist` | MCTS with cross-deck generalist weights |

### Alpha-Beta Bot

The `alphabeta` bot uses minimax search with alpha-beta pruning for efficient tree search. It achieves **60-70% win rate vs MCTS-1000** at depth 8.

```bash
# Basic usage (depth 6 default)
cargo run --release --bin arena -- --bot1 alphabeta --bot2 mcts --games 50

# Custom depth (8 recommended for strength, 6 for speed)
cargo run --release --bin arena -- --bot1 alphabeta --bot2 mcts --ab-depth 8 --games 50

# With tuned weights
cargo run --release --bin arena -- --bot1 alphabeta --bot2 mcts \
  --weights1 data/weights/alphabeta/generalist.toml --ab-depth 8 --games 50
```

**Performance characteristics:**
- Depth 6: ~8s/game, 90% vs MCTS-100
- Depth 8: ~50s/game, 60-70% vs MCTS-1000
- Depth 10+: Too slow for practical use

### MCTS Bot

The `mcts` bot uses Monte Carlo Tree Search with UCB1 selection and greedy rollouts.

**Config methods:**
```rust
MctsConfig::default()           // 1000 sims, sequential, optimizations enabled
MctsConfig::fast()              // 100 sims, for testing
MctsConfig::strong()            // 5000 sims, for serious play
MctsConfig::interactive(sims)   // Uses all CPU cores via parallel trees (MCP/UI)
MctsConfig::batch(sims)         // Sequential, for arena/benchmark with outer parallelism
MctsConfig::no_optimizations(sims)  // Baseline without early termination or TT
```

**Optimizations (enabled by default):**
- `early_termination_threshold: Some(300.0)` - Stop rollouts when position clearly won/lost
- `use_transposition_table: true` - Cache position evaluations across search
- `tt_min_visits: 3` - Minimum visits before trusting cached value

### Bot Trait

```rust
pub trait Bot: Send {
    fn select_action(&mut self, state_tensor: &[f32; 328],
                     legal_mask: &[f32; 256], legal_actions: &[Action]) -> Action;
    fn name(&self) -> &str;
    fn reset(&mut self);
    fn clone_box(&self) -> Box<dyn Bot>;
}
```

### GreedyBot Weights (24 params)

Life, creature stats, board state, resources, keywords (guard/lethal/lifesteal/rush/ranged/piercing/shield/quick), terminal bonuses. See `crates/cardgame/src/bots/weights.rs`.

## Game Rules

- **Commander System**: Each player has a Commander in the Command Zone (not on battlefield)
  - Commander Life = Player Life (30)
  - Commanders provide persistent Passive or Triggered abilities
  - Commanders cannot be targeted by attacks or spells
  - When Commander Life reaches 0, they "retreat" (you lose)
- 5 creature slots, 2 support slots per player
- 3 Action Points per turn
- 30 turn limit with life-based tiebreaker
- 16 keywords: Rush, Ranged, Piercing, Guard, Lifesteal, Lethal, Shield, Quick, Ephemeral, Regenerate, Stealth, Charge, Frenzy, Volatile, Fortify, Ward

## Faction System

| Faction | Keywords | Playstyle |
|---------|----------|-----------|
| **Argentum Combine** | Guard, Piercing, Shield | Defensive, outlast |
| **Symbiote Circles** | Rush, Lethal, Regenerate | Aggressive tempo |
| **Obsidion Syndicate** | Lifesteal, Stealth, Quick | Burst damage |
| **Free-Walkers** (neutral) | Ranged, Charge | Utility splash |

**Deck composition**: Minimum 30 cards (29 + commander), maximum 60 cards. Recommended: 21 faction (70%) + 9 neutral (30%)

## Card System

- **300 cards** total (75 per faction)
- **12 commanders** (4 per faction, no neutral commanders)
- **ID ranges**: Argentum 1000-1074, Symbiote 2000-2074, Obsidion 3000-3074, Neutral 4000-4074
- **Card files**: `data/cards/core_set/{argentum,symbiote,obsidion,neutral}.yaml`
- **Commander files**: `data/commanders/{argentum,symbiote,obsidion}.yaml`

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

# Commander (in data/commanders/*.yaml)
- id: 5002
  name: "Siege Marshal Vex"
  card_type: commander
  faction: Argentum
  rarity: Legendary
  passive_ability:
    description: "Your creatures have +1 Attack"
    effect:
      type: buff_stats
      attack: 1
      health: 0
  flavor: "A wall is just a door that hasn't been opened hard enough."
```

**Advanced features**: filters (`max_health`, `has_keyword`), conditional effects (`target_died`), bounce. See `docs/cards-new-horizons.md`.

## Deck System

12 commander decks (4 per faction). Use `cargo run --release --bin arena -- --list-decks` to see all.

**Format** (`data/decks/{faction}/{deck_id}.toml`):
```toml
id = "architect_fortify"
name = "Architect's Bastion"
commander = 1059               # Commander ID (separate from cards)
cards = [1001, 1002, ...]      # 29-60 cards (commander NOT included)
```

**Note:** Commanders are NOT in the deck. The deck has 29-60 cards + 1 commander (defined separately).

## Commander System

Commanders are the player's persona in battle. They define deck identity and provide persistent abilities.

| Commander | Faction | Ability Type | Ability |
|-----------|---------|--------------|---------|
| The High Artificer | Argentum | Triggered | StartOfTurn: Summon 2/2 Brass Cog |
| The Sanctum Healer | Argentum | Passive | Creatures have Ward and +0/+2 |
| Siege Marshal Vex | Argentum | Passive | Creatures have +2 Attack |
| The Grand Architect | Argentum | Passive | Creatures have Fortify and +0/+2 |
| The Broodmother | Symbiote | Passive | All creatures have Rush |
| Plague Sovereign | Symbiote | Triggered | OnAllyDeath: 2 damage to enemy commander |
| Alpha of the Hunt | Symbiote | Triggered | OnAttack: Give all creatures +1/+0 |
| The Eternal Grove | Symbiote | Triggered | StartOfTurn: Give all creatures +1/+1 |
| The Blood Sovereign | Obsidion | Passive | Creatures have Lifesteal and +0/+1 |
| The Deathmaster | Obsidion | Passive | Creatures with Lethal have Quick |
| The Shadow Weaver | Obsidion | Passive | Creatures have Stealth |
| Void Archon | Obsidion | Passive | Creatures have Quick |

**Key Rules:**
- Commanders are in Command Zone (not on battlefield)
- Commander Life = Player Life (30)
- Face attacks damage the commander
- Spells/abilities cannot target commanders
- See `docs/design-commanders.md` for full design doc

## State Tensor (328 floats)

| Section | Indices | Size | Description |
|---------|---------|------|-------------|
| Global state | 0-5 | 6 | Turn number, current player, game state |
| Player 1 state | 6-80 | 75 | Life, essence, AP, deck/hand info, board |
| Player 2 state | 81-155 | 75 | Same as P1 |
| Card embeddings | 156-325 | 170 | Card IDs from hands/boards (normalized) |
| Commander IDs | 326-327 | 2 | P1 commander (326), P2 commander (327) |

**Player state breakdown (75 floats each):**
- Base stats: 5 (life, essence, AP, deck size, hand size)
- Hand cards: 10 (normalized card IDs)
- Creatures: 50 (5 slots × 10 floats)
- Supports: 10 (2 slots × 5 floats)

**Normalization:**
- Card IDs: Divided by 6000.0 (handles cards up to 4074, commanders up to 5011)
- Commander IDs at indices 326-327: Also divided by 6000.0

## Action Space (256 indices)

| Range | Action |
|-------|--------|
| 0-99 | PlayCard (hand × slot) |
| 100-149 | Attack (slot × target) |
| 150-249 | UseAbility (slot × ability × target) |
| 255 | EndTurn |

## AI Interface (GameEnvironment trait)

```rust
trait GameEnvironment {
    fn get_state_tensor(&self) -> [f32; 328];
    fn get_legal_action_mask(&self) -> [f32; 256];
    fn apply_action_by_index(&mut self, index: u8);
    fn get_reward(&self, player: PlayerId) -> f32;  // -1.0, 0.0, or 1.0
    fn fork(&self) -> Self;  // Clone for MCTS
}
```

## Weight Tuning

CMA-ES optimizer with parallel evaluation. Outputs to `experiments/mcts/YYYY-MM-DD_HHMM_tag/`.

| Mode | Use Case |
|------|----------|
| `generalist` | Cross-deck optimization (all archetypes) |
| `archetype` | Playstyle-specific (aggro, control, tempo, midrange) |
| `specialist` | Specific deck matchup |

```bash
# Tune all archetypes (aggro, control, tempo, midrange)
./scripts/tune-archetypes.sh

# Tune specific archetypes
./scripts/tune-archetypes.sh aggro tempo

# Quick test run with fewer generations
./scripts/tune-archetypes.sh -g 50 -n 50 aggro

# Show available archetypes and their decks
./scripts/tune-archetypes.sh --list

# Preview commands without running
./scripts/tune-archetypes.sh --dry-run
```

See `docs/bots-tuning-pipeline.md` for full options.

## Balance Validation & Benchmarking

Two tools for balance testing with different speed/accuracy tradeoffs:

| Tool | Bot | Speed | Use Case |
|------|-----|-------|----------|
| `validate` | Greedy (fixed) | ~1 sec | Quick sanity checks, CI, after card changes |
| `benchmark` | Alpha-Beta/MCTS | Hours | Overnight runs, statistical rigor, pre-release |

### Quick Validation (`validate`)

Fast balance check using Greedy bot. Default: 10 games per matchup (~1 sec total).

```bash
cargo run --release --bin validate -- --progress              # Quick check (~1 sec)
cargo run --release --bin validate -- -n 50 --progress        # More games for confidence
cargo run --release --bin validate -- --run-id my_run         # Named output directory
```

### Thorough Benchmark (`benchmark`)

Statistical analysis with Alpha-Beta (default) or MCTS. Default: 50 games per matchup.

```bash
cargo run --release --bin benchmark -- --progress             # Alpha-Beta depth 6 (default)
cargo run --release --bin benchmark -- --ab-depth 8           # Deeper search (slower)
cargo run --release --bin benchmark -- --bot mcts --mcts-sims 200  # Use MCTS instead
cargo run --release --bin benchmark -- -n 100 --progress      # More games
cargo run --release --bin benchmark -- --deck1 broodmother_pack  # Single deck vs all others
```

**Output includes:**
- Per-deck win rates with 95% confidence intervals
- Best/worst matchups per deck
- Visual indicators: ▲ (>60% win rate), ▼ (<40% win rate)
- Results saved to `experiments/validation/` or `experiments/benchmark/`

**Balance thresholds:**
- Balanced deck: 40-60% win rate
- Outlier deck: <40% or >60% win rate
- Sample size: 50+ games per matchup recommended for statistical confidence

**Note:** The `--games-per-matchup` (or `-n`) flag sets games *per matchup per direction*. With 48 matchups × 2 directions, `-n 50` runs 4,800 total games.

## ELO Ratings & Swiss Tournaments

**ELO Tracking**: Arena matches automatically update deck ratings in `data/ratings/deck_elo.json`. Ratings persist across sessions for tracking deck strength over time.

**Swiss Tournaments** (`swiss` binary): Run full tournaments with score-based pairing. Useful for ranking all decks, testing new cards/expansions, or comparing deck archetypes.

```bash
cargo run --release --bin swiss -- --games 20 --progress          # All 12 decks, 5 rounds
cargo run --release --bin swiss -- --faction argentum --games 30  # Single faction (4 decks)
cargo run --release --bin swiss -- --bot mcts --rounds 4          # Custom bot/rounds
```

Features: Buchholz tiebreakers, bye handling, automatic ELO updates, JSON export (`--output`).

## Game Replay

The `replay` binary allows stepping through games for debugging and analysis.

**Input sources:**
- `--seed` + `--deck1` + `--deck2`: Generate a fresh game with bots
- `--file`: Load from `.replay.json` or `.replay.json.gz` (auto-validated)
- `--dataset` + `--game-index`: Extract from JSONL dataset

**Output modes:**
- Default: Game summary (turns, actions, result)
- `--interactive`: Step through with keyboard (n/p/s/q)
- `--export transcript`: Plain text action log
- `--export replay`: Compressed replay file

```bash
# Generate and summarize a game
cargo run --release --bin replay -- --seed 12345 --deck1 broodmother_pack --deck2 architect_fortify

# Interactive step-through (useful for debugging)
cargo run --release --bin replay -- --seed 12345 --deck1 broodmother_pack --deck2 architect_fortify -i

# Export to transcript for sharing/review
cargo run --release --bin replay -- --seed 12345 --deck1 X --deck2 Y --export transcript -o game.txt

# Load and validate existing replay
cargo run --release --bin replay -- --file game.replay.json.gz
```

**Use cases:**
- Debugging unexpected game outcomes
- Understanding why a bot made certain decisions
- Reproducing specific game states for testing
- Sharing game transcripts for review

## Performance

Benchmarks use real 30-card decks with commanders (v0.8.0+).

| Benchmark | Result | Notes |
|-----------|--------|-------|
| Random game | ~33k/sec | ~30 µs/game |
| Greedy game | ~4.3k/sec | ~230 µs/game |
| MCTS 100 sims | ~22 ms | Per move, with early termination + TT |
| MCTS 200 sims | ~44 ms | Per move, with early termination + TT |
| Engine fork | ~245 ns | State cloning |
| State tensor | ~158 ns | 328-float encoding |
| Legal actions | ~55 ns | Action enumeration |
| Throughput | ~18k games/sec | 10-game batches |

### MCTS Optimizations (v0.8.4+)

MCTS now includes three optimizations enabled by default:
- **Early Rollout Termination**: Stops rollouts when position evaluation exceeds ±300
- **Transposition Table**: Caches position evaluations to skip redundant rollouts
- **Interactive Config**: `MctsConfig::interactive(sims)` uses parallel trees for MCP/UI

## Versioning

Version in **root `Cargo.toml`** `[workspace.package]`.

| Change | Bump |
|--------|------|
| Game rules/API | MINOR |
| New features | MINOR |
| Bug fixes | PATCH |

**Update checklist**:
1. Root `Cargo.toml`: `version = "X.Y.Z"`
2. `crates/cardgame/src/version.rs`: Update test
3. Run tests

## MCP Server

The MCP (Model Context Protocol) server enables Claude Code to play Essence Wars games interactively.

### Architecture

```
Claude Code (MCP Client)
    │
    │ JSON-RPC (stdio)
    ▼
essence-wars-mcp (MCP Server)
    │
    │ HTTP sync → POST /sync_state (optional)
    ▼
essence-wars-ui (Tauri App)
```

### MCP Tools

| Tool | Description |
|------|-------------|
| `list_decks` | List available decks grouped by faction |
| `list_bots` | List AI opponent types |
| `start_game` | Start new game (player_deck, opponent_deck, bot_type, seed) |
| `show_state` | Display current board state |
| `show_hand` | Display cards in hand |
| `legal_actions` | List all legal moves with indices |
| `play_action` | Execute action by index |
| `ai_hint` | Get MCTS analysis and recommended move |
| `end_game` | End current game session |
| `sync_ui_state` | Manually sync MCP state to Tauri UI (if UI is running) |

### Typical Session

1. Use MCP tools to play:
   - `list_decks` → choose decks
   - `start_game` → begins game
   - `show_hand` / `legal_actions` → see options
   - `play_action` → make moves
2. `end_game` when done

### UI Sync (Optional)

If the Tauri UI app is running, you can sync game state to it:
- Use `sync_ui_state` to push current game state to the UI
- The UI will auto-switch to display the synced game state

```bash
# Start the UI
./scripts/launch-ui.sh

# Health check
curl http://127.0.0.1:9999/health
```

### Key Files

| File | Purpose |
|------|---------|
| `crates/essence-wars-mcp/` | MCP server crate |
| `crates/essence-wars-ui/` | Tauri desktop app |
| `.mcp.json` | Claude Code MCP configuration |

## Tauri Desktop App

The desktop UI is built with Tauri 2 (Rust backend + Svelte frontend).

### Development

```bash
cd crates/essence-wars-ui
pnpm install                    # Install dependencies (first time)
pnpm tauri:dev                  # Run in dev mode with hot reload
```

### Building for Linux

```bash
cd crates/essence-wars-ui
pnpm tauri:build                # Creates .deb, .rpm, .AppImage
```

Output: `target/release/bundle/{deb,rpm,appimage}/`

### Building for Windows (from WSL2)

Cross-compilation to Windows is fully supported from WSL2/Linux.

```bash
# Build unsigned (quick testing)
./scripts/build-windows.sh

# Build signed (for distribution)
./scripts/build-windows.sh --sign

# Or via pnpm
cd crates/essence-wars-ui
pnpm tauri:build:windows        # Unsigned
pnpm tauri:build:windows:signed # Signed
```

Output:
- `target/x86_64-pc-windows-gnu/release/essence-wars-ui.exe`
- `target/x86_64-pc-windows-gnu/release/bundle/nsis/essence-wars-ui_0.1.0_x64-setup.exe`

### Windows Code Signing

Self-signed certificate for Windows builds (reduces SmartScreen warnings):

```bash
# One-time setup: generate certificate
./scripts/build-windows.sh --setup-cert
```

Certificate location: `.certs/codesign.pfx` (gitignored, valid 10 years)

**Prerequisites for Windows builds:**
- `rustup target add x86_64-pc-windows-gnu`
- `sudo apt install mingw-w64 nsis osslsigncode`

### Build Scripts

| Script | Purpose |
|--------|---------|
| `scripts/build-windows.sh` | Windows cross-compile from WSL2 |
| `scripts/build-windows.sh --sign` | Build + sign with certificate |
| `scripts/build-windows.sh --setup-cert` | Generate self-signed cert |
| `scripts/launch-ui.sh` | Launch Linux UI in dev mode |

### UI Architecture

The UI is built with **Svelte 5** (using runes: `$state`, `$derived`, `$effect`, `$props`) and **Tailwind CSS v4**.

#### Game Modes

| Mode | Description |
|------|-------------|
| **Human vs AI** | Play against an AI opponent |
| **AI vs AI (Spectator)** | Watch two AI players battle |

#### Setup Flow (Deck Selection Wizard)

Both game modes use a 3-step wizard for game setup:

```
Step 1: Choose Your Commander (or Player 1)
  ├── Faction tabs (Argentum, Symbiote, Obsidion)
  ├── Deck grid (4 decks per faction)
  └── Deck preview panel

Step 2: Choose Your Opponent (or Player 2)
  └── Same UI as Step 1

Step 3: Game Options
  ├── Bot selection (opponent AI for Human vs AI, both AIs for spectator)
  ├── Turn order (Human vs AI only)
  └── Advanced options (AI vs AI only):
      ├── Watch Live toggle
      ├── AI Commentary toggle
      ├── MCTS simulations preset
      ├── Alpha-Beta depth preset
      └── Custom seed input
```

#### Key UI Components

| Component | Location | Purpose |
|-----------|----------|---------|
| `DeckSelectionWizard` | `src/lib/components/menu/` | 3-step setup wizard |
| `WizardStep` | `src/lib/components/menu/` | Step wrapper with progress indicator |
| `DeckCard` | `src/lib/components/menu/` | Deck selection card with commander portrait |
| `DeckGrid` | `src/lib/components/menu/` | Grid of deck cards filtered by faction |
| `DeckPreview` | `src/lib/components/menu/` | Large preview panel for selected deck |
| `FactionTabs` | `src/lib/components/menu/` | Faction filter tabs |
| `CommanderCardLarge` | `src/lib/components/board/` | Commander display during gameplay |
| `GameBoard` | `src/lib/components/board/` | Main gameplay board |
| `SetupScreen` | `src/lib/components/` | Human vs AI setup (uses wizard) |
| `SpectatorSetup` | `src/lib/components/` | AI vs AI setup (uses wizard) |

#### Faction Theming

Each faction has distinct colors defined in `src/app.css`:

| Faction | Primary | Accent |
|---------|---------|--------|
| Argentum | `#F5F5F5` | `#D4AF37` (gold) |
| Symbiote | `#1A472A` | `#7FFF00` (lime) |
| Obsidion | `#8B0000` | `#00FFFF` (cyan) |
| Neutral | `#8B4513` | `#B87333` (copper) |

#### Keyboard Navigation

| Key | Action |
|-----|--------|
| `Tab` | Navigate between interactive elements |
| `Enter` / `Space` | Select/activate focused element |
| `Escape` | Go back to previous step |

#### Audio System

Sound effects and music are managed in `src/lib/audio/`:

| Module | Purpose |
|--------|---------|
| `manager.ts` | Sound effects (UI, cards, combat) |
| `music.ts` | Background music and victory/defeat stings |

Key sounds: `buttonClick`, `buttonHover`, `cardSelect`, `cardHover`, `menuOpen`, `menuClose`
