# CLAUDE.md - AI Assistant Context for Essence Wars

## Project Overview

**Essence Wars** is a deterministic, perfect-information card game engine designed for AI research (reinforcement learning, MCTS). The engine is written in Rust with a focus on performance and correctness.

## Quick Commands

```bash
# Build
cargo build --release

# Run all tests
cargo test  # 218 tests

# Run arena matches
cargo run --release --bin arena -- --bot1 greedy --bot2 random --games 100
cargo run --release --bin arena -- --bot1 mcts --bot2 greedy --games 10
cargo run --release --bin arena -- --deck1 aggressive_assault --deck2 defensive_control
cargo run --release --bin arena -- --list-decks
```

## Project Structure

```
ai-cardgame/
├── src/
│   ├── lib.rs          # Module exports
│   ├── types.rs        # Core types: CardId, PlayerId, Slot, etc.
│   ├── keywords.rs     # 8 keywords as u8 bitfield
│   ├── effects.rs      # Trigger, Effect, EffectTarget, TargetingRule
│   ├── state.rs        # GameState, PlayerState, Creature, Support
│   ├── cards.rs        # CardDefinition, CardDatabase, YAML loading
│   ├── actions.rs      # Action enum with index mapping (256 actions)
│   ├── legal.rs        # Legal action generation
│   ├── engine.rs       # GameEngine, effect queue, turn structure
│   ├── combat.rs       # Combat resolution with keyword interactions
│   ├── tensor.rs       # State to tensor conversion (326 floats)
│   ├── config.rs       # GameConfig with all parameters
│   ├── decks.rs        # DeckDefinition, DeckRegistry, TOML loading
│   ├── bots/
│   │   ├── mod.rs      # Bot trait definition
│   │   ├── random.rs   # RandomBot - uniform random selection
│   │   ├── greedy.rs   # GreedyBot - heuristic evaluation, action simulation
│   │   ├── mcts.rs     # MctsBot - Monte Carlo Tree Search with UCB1
│   │   └── weights.rs  # Configurable weights for GreedyBot
│   ├── arena/
│   │   ├── mod.rs      # Arena module exports
│   │   ├── runner.rs   # GameRunner for executing matches
│   │   ├── logger.rs   # ActionLogger for debug tracing
│   │   └── stats.rs    # MatchStats for win rate tracking
│   └── bin/
│       └── arena.rs    # CLI for running bot matches
├── data/
│   ├── cards/sets/
│   │   └── starter.yaml  # 43 card definitions
│   └── decks/
│       ├── aggressive_assault.toml  # Aggro deck
│       └── defensive_control.toml   # Control deck
├── tests/
│   ├── engine_tests.rs
│   ├── effect_queue_tests.rs
│   ├── card_playing_tests.rs
│   ├── game_interface_tests.rs
│   ├── integration_tests.rs
│   └── simulation_tests.rs
└── docs/
    ├── essence-wars-design.md
    ├── design-engine.md
    └── implementation-plan.md
```

## Bot System

### Bot Types

| Bot | Description | Strength |
|-----|-------------|----------|
| RandomBot | Uniform random action selection | Baseline |
| GreedyBot | Simulates each action, picks best by heuristic | Beats Random 100% |
| MctsBot | UCB1 tree search with GreedyBot rollouts | Beats Greedy 60-100% |

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

### GreedyBot Weights

GreedyBot uses configurable weights for state evaluation:
- Life values (own_life, enemy_life_damage)
- Creature stats (attack, health)
- Board control (creature_count, board_advantage)
- Keywords (guard, lethal, shield, etc.)
- Win/lose bonuses

Weights can be saved/loaded as TOML for tuning.

### MCTS Configuration

```rust
MctsConfig {
    simulations: 500,      // Iterations per move
    exploration: 1.414,    // UCB1 exploration constant (sqrt(2))
    max_rollout_depth: 100,
}
```

## Deck System

### TOML Format

```toml
id = "aggressive_assault"
name = "Aggressive Assault"
description = "Fast aggro deck"
tags = ["aggro", "creature-focused"]

cards = [
    1, 1,   # Eager Recruit x2
    3, 3,   # Nimble Scout x2
    # ... 18 total cards
]
```

### Arena CLI

```bash
# List available decks
cargo run --bin arena -- --list-decks

# Run match with specific decks
cargo run --bin arena -- --deck1 aggressive_assault --deck2 defensive_control

# Full options
cargo run --bin arena -- \
  --bot1 mcts --bot2 greedy \
  --deck1 aggressive_assault --deck2 defensive_control \
  --games 100 --seed 12345 \
  --debug  # or --verbose for state snapshots
```

## Key Design Decisions

### Performance Optimizations
- **ArrayVec** for fixed-size collections - stack allocation, fast cloning for MCTS
- **u8 bitfield** for keywords - compact representation
- **Indexed action space** (256 actions) - fixed-size for neural networks
- **Engine fork()** - efficient state cloning for tree search

### Game Rules
- 5 creature slots, 2 support slots per player
- 3 Action Points per turn
- 30 turn limit with life-based tiebreaker
- 8 keywords: Rush, Ranged, Piercing, Guard, Lifesteal, Lethal, Shield, Quick

### AI Interface (GameEnvironment trait)
- `get_state_tensor()` - 326 floats representing full game state
- `get_legal_action_mask()` - 256 floats (0.0 or 1.0)
- `apply_action_by_index(u8)` - apply action from neural network output
- `get_reward(PlayerId)` - -1.0, 0.0, or 1.0
- `fork()` - clone game state for MCTS tree search

## Card YAML Schema

```yaml
# Creature
- id: 9
  name: "Blood Cultist"
  cost: 2
  card_type: creature
  attack: 3
  health: 2
  keywords: [Rush]  # optional
  abilities:        # optional
    - trigger: OnPlay
      effects:
        - type: damage
          amount: 2

# Spell
- id: 34
  name: "Lightning Bolt"
  cost: 3
  card_type: spell
  targeting: TargetAny
  effects:
    - type: damage
      amount: 4

# Support
- id: 40
  name: "War Drums"
  cost: 3
  card_type: support
  durability: 3
  passive_effects:
    - modifier:
        attack_bonus: 1
```

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
- All core game rules and 8 keywords
- AI interface (tensor, action mask, rewards)
- 43-card starter set
- Bot system (RandomBot, GreedyBot, MctsBot)
- Arena for running matches
- Deck system with TOML definitions
- 218 tests passing

**Future work:**
- Phase 5: Tuning Pipeline (CMA-ES optimizer)
- Phase 6: Polish & Performance (benchmarks, progress bars)
- Python bindings (PyO3) for ML training
- Additional card sets
