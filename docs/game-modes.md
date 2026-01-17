# Game Modes

Essence Wars supports two game modes with different victory conditions, designed to support both balanced competitive play and transfer learning research.

## Attrition (Default)

The classic mode where games are won through sustained combat until one faction is exhausted.

**Victory Conditions:**
1. **Primary**: Reduce enemy life to 0
2. **Timeout**: After turn 30, the player with higher life wins (tie goes to Player 1)

**Characteristics:**
- Games can be short (aggressive strategies) or long (control strategies)
- Encourages both tempo and value-oriented play
- Turn limit prevents infinite stalls

**Training Status:** Fully trained. MCTS weights are optimized for this mode.

```bash
# Default mode - no flag needed
cargo run --release --bin arena -- --bot1 greedy --bot2 greedy --games 100

# Explicit mode
cargo run --release --bin arena -- --bot1 mcts --bot2 greedy --games 100 --mode attrition
```

---

## Essence Duel (Experimental)

A faster-paced mode where victory is claimed by extracting essence (dealing face damage) rather than complete annihilation.

**Victory Conditions:**
1. **Primary**: First player to deal 50 cumulative face damage wins (Victory Points)
2. **Alternate**: Reduce enemy life to 0 (still works)
3. **Timeout**: After turn 30, higher life wins (same as Attrition)

**Characteristics:**
- Rewards aggressive, damage-focused strategies
- Games tend to be shorter than Attrition
- Life total of 30 means games can end before 50 VP threshold
- Creates tension: chip damage vs burst vs defense

**Training Status:** Experimental. No dedicated training - use for transfer learning research.

```bash
cargo run --release --bin arena -- --bot1 greedy --bot2 greedy --games 100 --mode essence-duel
```

---

## Lore Context

> *"The Essence Wars manifest in two forms: the brutal Attrition campaigns where factions fight to exhaustion, and the ritualized Essence Duels where victory is claimed by extracting 50 units of essence from the opposing force. Most conflicts take the form of Duels—decisive yet measured—while Attrition is reserved for existential threats."*

---

## Research Applications

### Transfer Learning Experiment

```text
┌─────────────────────────────────────────────┐
│  Attrition Mode                             │
│  ✓ MCTS weights trained and validated       │
│  ✓ Balance verified across factions         │
└──────────────────┬──────────────────────────┘
                   │ Transfer
                   ▼
┌─────────────────────────────────────────────┐
│  Essence Duel Mode                          │
│  ✗ No dedicated training                    │
│  ? How do Attrition-trained agents perform? │
│  ? What strategy shifts emerge?             │
└─────────────────────────────────────────────┘
```

**Research Questions:**
1. Do MCTS weights trained for Attrition transfer well to Essence Duel?
2. What performance gap exists between modes?
3. How quickly can agents adapt to the new objective?

### Mode-Specific Analysis

Use the arena with different modes to compare:

```bash
# Compare same bots across modes
cargo run --release --bin arena -- --bot1 agent-symbiote --bot2 agent-argentum \
    --games 500 --mode attrition --progress

cargo run --release --bin arena -- --bot1 agent-symbiote --bot2 agent-argentum \
    --games 500 --mode essence-duel --progress
```

---

## API Reference

### GameMode Enum

```rust
use cardgame::core::state::GameMode;

pub enum GameMode {
    /// Reduce enemy to 0 life, or turn 30 → higher life wins
    Attrition,
    /// First to 50 VP (face damage) or reduce to 0 life
    EssenceDuel,
}
```

### Starting Games with Mode

```rust
use cardgame::engine::GameEngine;
use cardgame::core::state::GameMode;

let mut engine = GameEngine::new(&card_db);

// Default (Attrition)
engine.start_game(deck1, deck2, seed);

// Explicit mode
engine.start_game_with_mode(deck1, deck2, seed, GameMode::EssenceDuel);
```

### Configuration

Victory Points threshold is defined in `src/core/config.rs`:

```rust
pub mod game {
    pub const TURN_LIMIT: u8 = 30;
    pub const VICTORY_POINTS_THRESHOLD: u16 = 50;
}
```

---

## Future Considerations

- **Mode-Specific Cards**: Cards that interact with VP (e.g., "Gain 3 VP when this creature attacks")
- **Custom Thresholds**: Allow configuring VP threshold for variant modes
- **Hybrid Modes**: Combine conditions (e.g., VP + board control)
- **Mode-Specific Weights**: Train dedicated MCTS weights for Essence Duel
