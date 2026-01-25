# CLAUDE.md - AI Assistant Context for Essence Wars

## Project Overview

**Essence Wars** is a deterministic, perfect-information card game engine. Written in Rust with focus on performance and correctness.

**Current Version:** 0.8.0 (Commander Edition) | **Author:** Christian Wissmann (Chris), Best Friends with Claude

## Quick Commands

```bash
# Build
cargo build --release                    # Full workspace
cargo build --release -p cardgame        # Core engine only
./scripts/build-windows.sh --sign        # Windows installer (from WSL2)

# Test
cargo nextest run --status-level=fail    # ~629 tests (recommended)
cargo test                               # Alternative

# Lint
./scripts/run-clippy.sh                  # Recommended: lib + binaries

# Stress tests by tier
./scripts/run-tests.sh                   # Standard only
./scripts/run-tests.sh quick|medium|long|overnight

# Arena matches
cargo run --release --bin arena -- --bot1 mcts --bot2 greedy --games 100 --progress
cargo run --release --bin arena -- --list-decks

# Weight tuning
cargo run --release --bin tune -- --mode generalist --tag my_run --generations 50
cargo run --release --bin tune -- --mode faction-specialist --faction argentum --tag argentum_v1

# Analysis & validation
./scripts/analyze-tuning.sh --latest
cargo run --release --bin validate -- --games 100
cargo run --release --bin diagnose -- 200

# Benchmarks
cargo bench -p cardgame

# Modal cloud (setup: uv tool install modal && modal token new)
modal run modal_tune.py::main
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
- `crates/cardgame/src/bin/` - CLIs: arena, tune, validate, diagnose
- `crates/cardgame/tests/unit/` - Unit tests (separate from src)
- `python/essence_wars/` - Python bindings and ML agents
- `python/essence_wars/agents/` - PPO, AlphaZero, Card2Vec, embeddings
- `python/essence_wars/data/` - Dataset loaders (MCTSDataset, ChunkedMCTSDataset)
- `python/scripts/` - Training scripts (train_ppo.py, train_alphazero.py, etc.)
- `data/cards/core_set/` - 300 cards in 4 YAML files (by faction)
- `data/commanders/` - 12 commanders in 3 YAML files (by faction)
- `data/decks/{argentum,symbiote,obsidion}/` - 12 commander decks
- `data/datasets/` - MCTS game datasets (JSONL.gz)
- `data/weights/` - Tuned bot weights (generalist + specialists)
- `models/` - Trained ML models (Card2Vec, BC, PPO checkpoints)
- `experiments/` - Training outputs (GITIGNORED)
- `docs/` - Design docs, guides

## Test Organization

**IMPORTANT**: Unit tests are **separate from source code** (not inline `#[cfg(test)]` blocks).

- **Unit tests**: `crates/cardgame/tests/unit/<module>_tests.rs`
- **Integration tests**: `crates/cardgame/tests/*.rs`
- **Shared utilities**: `crates/cardgame/tests/common/mod.rs`

When adding tests, create in `tests/unit/` and add module to `tests/unit.rs`.

### Test Tiers

| Tier | Duration | When |
|------|----------|------|
| Standard | ~2 min | Every commit |
| tier_quick | ~2 min | PRs |
| tier_medium | ~10 min | Nightly |
| tier_long | ~30 min | Nightly |
| tier_overnight | ~2 hours | Weekly |

## Bot System

| Bot | Description |
|-----|-------------|
| `random` | Uniform random selection |
| `greedy` | Heuristic evaluation (28 tunable weights) |
| `mcts` | UCB1 tree search with greedy rollouts |
| `alphabeta` | Alpha-Beta minimax search (depth 6-8 recommended) |
| `agent-{faction}` | MCTS with faction-specialist weights |
| `agent-generalist` | MCTS with cross-faction weights |

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

**Deck composition**: 30 cards = 21 faction (70%) + 9 neutral (30%)

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
- id: 1058
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
cards = [1001, 1002, ...]      # 30 cards (commander NOT included)
```

**Note:** Commanders are NOT in the deck. The deck has 30 cards + 1 commander (defined separately).

## Commander System

Commanders are the player's persona in battle. They define deck identity and provide persistent abilities.

| Commander | Faction | Ability Type | Ability |
|-----------|---------|--------------|---------|
| The High Artificer | Argentum | Triggered | StartOfTurn: Summon 1/1 Brass Cog |
| The Sanctum Healer | Argentum | Passive | Creatures have Regenerate |
| Siege Marshal Vex | Argentum | Passive | Creatures have +1 Attack |
| The Grand Architect | Argentum | Passive | Creatures have Fortify |
| The Broodmother | Symbiote | Triggered | OnCreaturePlayed (Rush): Summon 1/1 Rush Broodling |
| Plague Sovereign | Symbiote | Triggered | OnAllyDeath: 1 damage to enemy commander |
| Alpha of the Hunt | Symbiote | Passive | Creatures have +1 Attack |
| The Eternal Grove | Symbiote | Passive | Creatures have Regenerate |
| The Blood Sovereign | Obsidion | Passive | Creatures have Lifesteal |
| Shadow Emperor Kael | Obsidion | Triggered | OnEnemyDeath: Draw a card |
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
| `generalist` | Cross-faction optimization |
| `specialist` | Specific deck matchup |
| `faction-specialist` | Faction-wide optimization |

See `docs/tuning-pipeline.md` for full options.

## Performance

| Benchmark | Throughput |
|-----------|------------|
| Random game | ~80k/sec |
| Greedy game | ~17k/sec |
| Engine fork | ~99 ns |

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
- `target/x86_64-pc-windows-gnu/release/essence-wars-ui.exe` (54MB)
- `target/x86_64-pc-windows-gnu/release/bundle/nsis/essence-wars-ui_0.1.0_x64-setup.exe` (36MB)

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
