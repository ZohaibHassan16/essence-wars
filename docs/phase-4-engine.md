# Phase 4: Engine Enhancements

This document describes the Phase 4 engine enhancements that expand the card design space without using keyword slots.

## Overview

Phase 4 introduces three major features:
1. **Creature Filters** - Target creatures by health, keywords, or other criteria
2. **Conditional Triggers** - Execute bonus effects based on results (e.g., "if target dies")
3. **Bounce Effect** - Return creatures to their owner's hand

## 1. Creature Filters

### Purpose
Allow effects to target or affect only creatures matching specific criteria.

### Filter Fields

| Field | Type | Description |
|-------|------|-------------|
| `max_health` | u8 | Target must have current health ≤ value |
| `min_health` | u8 | Target must have current health ≥ value |
| `has_keyword` | u16 | Target must have the keyword (bit value) |
| `lacks_keyword` | u16 | Target must NOT have the keyword (bit value) |

### Keyword Bit Values

| Keyword | Bit Value |
|---------|-----------|
| Guard | 1 |
| Lethal | 2 |
| Lifesteal | 4 |
| Rush | 8 |
| Ranged | 16 |
| Piercing | 32 |
| Shield | 64 |
| Quick | 128 |
| Ephemeral | 256 |
| Regenerate | 512 |
| Stealth | 1024 |
| Charge | 2048 |
| Frenzy | 4096 |
| Volatile | 8192 |

### YAML Examples

**Execute-style removal:**
```yaml
- id: 1025
  name: "Execute"
  cost: 2
  card_type: spell
  targeting: TargetEnemyCreature
  effects:
    - type: destroy
      filter:
        max_health: 3
  rarity: Common
```

**Keyword synergy buff:**
```yaml
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
        has_keyword: 8  # Rush
  rarity: Uncommon
```

**Anti-Guard tech:**
```yaml
effects:
  - type: damage
    amount: 3
    filter:
      has_keyword: 1  # Guard
```

### Supported Effect Types

Filters can be applied to:
- `damage` - Deal damage only to matching creatures
- `heal` - Heal only matching creatures
- `buff_stats` - Buff only matching creatures
- `destroy` - Destroy only matching creatures
- `grant_keyword` - Grant keyword to matching creatures
- `remove_keyword` - Remove keyword from matching creatures
- `silence` - Silence only matching creatures
- `bounce` - Bounce only matching creatures

## 2. Conditional Triggers

### Purpose
Allow spells and abilities to have bonus effects that trigger based on the outcome of the primary effects.

### Conditions

| Condition | Description |
|-----------|-------------|
| `target_died` | The primary target was destroyed |

### YAML Syntax

```yaml
effects:
  - type: damage
    amount: 3
conditional_effects:
  - condition: target_died
    effects:
      - type: draw
        count: 1
```

### Examples

**Kill reward spell:**
```yaml
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
  rarity: Uncommon
```

**Kill reward with heal:**
```yaml
- id: 3030
  name: "Soul Harvest"
  cost: 3
  card_type: spell
  targeting: TargetEnemyCreature
  effects:
    - type: damage
      amount: 3
  conditional_effects:
    - condition: target_died
      effects:
        - type: heal
          amount: 3
  rarity: Uncommon
```

**Multiple bonus effects:**
```yaml
- id: 3032
  name: "Vampiric Execution"
  cost: 5
  card_type: spell
  targeting: TargetEnemyCreature
  effects:
    - type: damage
      amount: 5
  conditional_effects:
    - condition: target_died
      effects:
        - type: heal
          amount: 5
        - type: draw
          count: 1
  rarity: Rare
```

### Conditional Effects on Creatures

Creature abilities can also have conditional effects:

```yaml
- id: 2038
  name: "Pounce Hunter"
  cost: 3
  card_type: creature
  attack: 3
  health: 2
  keywords: [Rush]
  rarity: Uncommon
  tags: [Beast]
  abilities:
    - trigger: OnKill
      targeting: NoTarget
      effects:
        - type: draw
          count: 1
```

Note: The `OnKill` trigger fires when this creature kills another in combat. For spell-based kill rewards, use `conditional_effects` with `target_died`.

## 3. Bounce Effect

### Purpose
Return a creature from the battlefield to its owner's hand.

### Behavior
1. The creature is removed from the board
2. The original card is added to the owner's hand
3. If the hand is full (20 cards), the card is discarded
4. All buffs, damage, and applied effects are removed (creature returns as fresh card)

### YAML Syntax

**Simple bounce:**
```yaml
- type: bounce
```

**Bounce with filter:**
```yaml
- type: bounce
  filter:
    max_health: 3
```

### Examples

**Single-target bounce:**
```yaml
- id: 1029
  name: "Temporal Displacement"
  cost: 3
  card_type: spell
  targeting: TargetEnemyCreature
  effects:
    - type: bounce
  rarity: Common
```

**Mass bounce with filter:**
```yaml
- id: 1030
  name: "Mass Recall"
  cost: 5
  card_type: spell
  targeting: NoTarget
  effects:
    - type: bounce
      filter:
        max_health: 3
  rarity: Rare
```

**Bounce + damage combo:**
```yaml
- id: 3033
  name: "Shadow Return"
  cost: 2
  card_type: spell
  targeting: TargetEnemyCreature
  effects:
    - type: bounce
    - type: damage
      amount: 1
  rarity: Common
```

**Self-bounce for card advantage:**
```yaml
- id: 2040
  name: "Pack Recall"
  cost: 2
  card_type: spell
  targeting: TargetAllyCreature
  effects:
    - type: bounce
    - type: draw
      count: 1
  rarity: Common
```

## Implementation Details

### Files Modified
- `src/core/effects.rs` - Added `Condition`, `EffectResult`, filter on `Effect` variants
- `src/core/cards.rs` - Added `ConditionalEffectGroup`, filter on `EffectDefinition` variants
- `src/core/engine/effect_queue.rs` - Filter application, bounce handler, condition tracking
- `src/core/engine/effect_convert.rs` - Filter propagation in effect conversion
- `src/core/legal.rs` - Filter application in targeting validation

### Key Structs

```rust
// Condition for conditional effects
pub enum Condition {
    TargetDied,
}

// Result tracking for condition checking
pub struct EffectResult {
    pub target_died: bool,
}

// Group of conditional effects
pub struct ConditionalEffectGroup {
    pub condition: Condition,
    pub effects: Vec<EffectDefinition>,
}

// Creature filter
pub struct CreatureFilter {
    pub max_health: Option<u8>,
    pub min_health: Option<u8>,
    pub has_keyword: Option<u16>,
    pub lacks_keyword: Option<u16>,
}
```

## Design Rationale

### Why Filters?
- Enables "execute" style conditional removal without new keywords
- Allows keyword-synergy effects (e.g., "buff all Rush creatures")
- Creates design space for anti-meta tech cards

### Why Conditional Triggers?
- Rewards skilled play (getting value from kills)
- Creates risk/reward decisions (overkill vs efficiency)
- Enables combo-style card design

### Why Bounce?
- Provides tempo-based control tools
- Enables reset/replay synergies (re-trigger OnPlay effects)
- Creates interesting decisions (bounce own creatures for value)

## Testing

Phase 4 features are covered by:
- Unit tests in `tests/unit/effects_tests.rs` - Filter matching
- Integration tests in `tests/effect_queue_tests.rs` - Bounce, conditional effects
- Card loading tests in `tests/unit/cards_tests.rs` - YAML parsing

Run tests with:
```bash
cargo nextest run --status-level=fail
```
