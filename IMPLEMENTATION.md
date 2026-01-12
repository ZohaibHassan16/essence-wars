# Implementation Plan

> **Document Purpose:** Orchestration guide for parallel implementation using Task agents
> **Senior Dev / Lead:** Claude (maintains high-level context, reviews, integrates)
> **Workers:** Task agents (implement specific modules)

---

## Overview

This document breaks the implementation into **discrete, parallelizable tasks** that can be dispatched to Task agents. Each task is self-contained with clear inputs, outputs, and acceptance criteria.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         IMPLEMENTATION PHASES                                │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  PHASE 1: Project Setup & Core Types (Foundation)                          │
│  ════════════════════════════════════════════════                          │
│  Tasks: 1.1, 1.2, 1.3, 1.4                                                 │
│  Can parallelize: 1.1 alone, then 1.2+1.3+1.4 together                     │
│                                                                             │
│  PHASE 2: Game State & Cards                                               │
│  ═══════════════════════════                                               │
│  Tasks: 2.1, 2.2, 2.3                                                      │
│  Can parallelize: 2.1+2.2 together, then 2.3                               │
│                                                                             │
│  PHASE 3: Actions & Legal Move Generation                                  │
│  ════════════════════════════════════════                                  │
│  Tasks: 3.1, 3.2                                                           │
│  Sequential: 3.1 then 3.2                                                  │
│                                                                             │
│  PHASE 4: Game Engine Core                                                 │
│  ═════════════════════════                                                 │
│  Tasks: 4.1, 4.2, 4.3                                                      │
│  Can parallelize: 4.1+4.2 together, then 4.3                               │
│                                                                             │
│  PHASE 5: Combat System                                                    │
│  ══════════════════════                                                    │
│  Tasks: 5.1, 5.2                                                           │
│  Sequential: 5.1 then 5.2                                                  │
│                                                                             │
│  PHASE 6: AI Interface                                                     │
│  ═════════════════════                                                     │
│  Tasks: 6.1, 6.2                                                           │
│  Can parallelize: 6.1+6.2 together                                         │
│                                                                             │
│  PHASE 7: Testing & Validation                                             │
│  ════════════════════════════                                              │
│  Tasks: 7.1, 7.2, 7.3                                                      │
│  Can parallelize: All together after engine complete                       │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Dependency Graph

```
Phase 1: Setup & Types
    1.1 Project Setup
     │
     ├──► 1.2 Core Types (CardId, PlayerId, Slot)
     │
     ├──► 1.3 Keywords (bitfield)
     │
     └──► 1.4 Effect Types

Phase 2: State & Cards (depends on Phase 1)
    1.2, 1.3, 1.4
     │
     ├──► 2.1 Game State structs
     │
     ├──► 2.2 Card Definition structs
     │
     └──► 2.3 YAML Loading + CardDatabase

Phase 3: Actions (depends on 1.2, 2.1)
    1.2, 2.1
     │
     └──► 3.1 Action enum + indexing
          │
          └──► 3.2 Legal action generation

Phase 4: Engine (depends on Phase 2, 3)
    2.x, 3.x
     │
     ├──► 4.1 Turn structure (start/end turn)
     │
     ├──► 4.2 Effect queue system
     │
     └──► 4.3 Card playing logic

Phase 5: Combat (depends on 4.1, 4.2, 1.3)
    4.1, 4.2, 1.3
     │
     └──► 5.1 Basic combat resolution
          │
          └──► 5.2 Keyword interactions

Phase 6: AI Interface (depends on 2.1, 3.1)
    2.1, 3.1
     │
     ├──► 6.1 State tensor conversion
     │
     └──► 6.2 Game interface trait

Phase 7: Testing (depends on all)
    All phases
     │
     ├──► 7.1 Unit tests for keywords
     │
     ├──► 7.2 Integration tests for combat
     │
     └──► 7.3 Full game simulation tests
```

---

## Phase 1: Project Setup & Core Types

### Task 1.1: Project Setup

**Priority:** FIRST (blocking all others)
**Estimated complexity:** Simple
**Can parallelize:** No (must complete first)

**Objective:** Create the Rust project structure with Cargo.toml and module organization.

**Files to create:**
```
ai-cardgame/
├── Cargo.toml
├── src/
│   ├── lib.rs           (module declarations)
│   ├── types.rs         (placeholder)
│   ├── keywords.rs      (placeholder)
│   ├── effects.rs       (placeholder)
│   ├── state.rs         (placeholder)
│   ├── cards.rs         (placeholder)
│   ├── actions.rs       (placeholder)
│   ├── legal.rs         (placeholder)
│   ├── engine.rs        (placeholder)
│   ├── combat.rs        (placeholder)
│   ├── tensor.rs        (placeholder)
│   └── config.rs        (placeholder)
├── data/
│   └── cards/
│       └── sets/
│           └── .gitkeep
└── tests/
    └── .gitkeep
```

**Cargo.toml dependencies:**
```toml
[package]
name = "cardgame"
version = "0.1.0"
edition = "2021"

[dependencies]
arrayvec = "0.7"           # Stack-allocated vectors
serde = { version = "1.0", features = ["derive"] }
serde_yaml = "0.9"         # YAML parsing
thiserror = "1.0"          # Error handling

[dev-dependencies]
criterion = "0.5"          # Benchmarking
```

**Acceptance criteria:**
- [ ] `cargo check` passes
- [ ] All module files exist (can be empty with `// TODO`)
- [ ] lib.rs declares all modules

---

### Task 1.2: Core Types

**Priority:** High
**Depends on:** Task 1.1
**Can parallelize with:** 1.3, 1.4

**Objective:** Implement fundamental type definitions.

**File:** `src/types.rs`

**Types to implement:**

```rust
/// Unique identifier for a card definition
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CardId(pub u16);

/// Player identifier (0 or 1)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PlayerId(pub u8);

impl PlayerId {
    pub const PLAYER_ONE: PlayerId = PlayerId(0);
    pub const PLAYER_TWO: PlayerId = PlayerId(1);

    #[inline]
    pub fn opponent(self) -> PlayerId {
        PlayerId(1 - self.0)
    }

    #[inline]
    pub fn index(self) -> usize {
        self.0 as usize
    }
}

/// Board slot position (0-4 for creatures, 0-1 for supports)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Slot(pub u8);

impl Slot {
    pub const CREATURE_SLOTS: usize = 5;
    pub const SUPPORT_SLOTS: usize = 2;

    /// Get slots that can be attacked from this slot (lane adjacency)
    pub fn adjacent_slots(self) -> &'static [Slot] {
        match self.0 {
            0 => &[Slot(0), Slot(1)],
            1 => &[Slot(0), Slot(1), Slot(2)],
            2 => &[Slot(1), Slot(2), Slot(3)],
            3 => &[Slot(2), Slot(3), Slot(4)],
            4 => &[Slot(3), Slot(4)],
            _ => &[],
        }
    }

    /// Check if this slot can attack target slot
    pub fn can_attack(self, target: Slot) -> bool {
        self.adjacent_slots().contains(&target)
    }
}

/// Unique identifier for a creature instance on the board
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CreatureInstanceId(pub u32);

/// Card rarity
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum Rarity {
    #[default]
    Common,
    Uncommon,
    Rare,
    Legendary,
}

/// Creature type tags
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Tag {
    Soldier,
    Beast,
    Mage,
    Undead,
    Construct,
    Divine,
    Assassin,
    Healer,
    Cultist,
    Giant,
}
```

**Acceptance criteria:**
- [ ] All types compile
- [ ] `PlayerId::opponent()` returns correct opponent
- [ ] `Slot::adjacent_slots()` returns correct adjacencies
- [ ] All types derive necessary traits (Clone, Copy, Debug, etc.)

---

### Task 1.3: Keywords (Bitfield)

**Priority:** High
**Depends on:** Task 1.1
**Can parallelize with:** 1.2, 1.4

**Objective:** Implement the keyword bitfield for efficient keyword checking.

**File:** `src/keywords.rs`

**Implementation:**

```rust
/// Keywords packed into a single byte for efficiency
/// Each bit represents one keyword
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Keywords(pub u8);

impl Keywords {
    // Bit positions
    pub const RUSH: u8      = 0b0000_0001;
    pub const RANGED: u8    = 0b0000_0010;
    pub const PIERCING: u8  = 0b0000_0100;
    pub const GUARD: u8     = 0b0000_1000;
    pub const LIFESTEAL: u8 = 0b0001_0000;
    pub const LETHAL: u8    = 0b0010_0000;
    pub const SHIELD: u8    = 0b0100_0000;
    pub const QUICK: u8     = 0b1000_0000;

    pub const fn none() -> Self { Self(0) }
    pub const fn all() -> Self { Self(0xFF) }

    // Checkers (inline for performance)
    #[inline(always)] pub const fn has_rush(self) -> bool { self.0 & Self::RUSH != 0 }
    #[inline(always)] pub const fn has_ranged(self) -> bool { self.0 & Self::RANGED != 0 }
    #[inline(always)] pub const fn has_piercing(self) -> bool { self.0 & Self::PIERCING != 0 }
    #[inline(always)] pub const fn has_guard(self) -> bool { self.0 & Self::GUARD != 0 }
    #[inline(always)] pub const fn has_lifesteal(self) -> bool { self.0 & Self::LIFESTEAL != 0 }
    #[inline(always)] pub const fn has_lethal(self) -> bool { self.0 & Self::LETHAL != 0 }
    #[inline(always)] pub const fn has_shield(self) -> bool { self.0 & Self::SHIELD != 0 }
    #[inline(always)] pub const fn has_quick(self) -> bool { self.0 & Self::QUICK != 0 }

    // Mutators
    #[inline(always)] pub fn add(&mut self, keyword: u8) { self.0 |= keyword; }
    #[inline(always)] pub fn remove(&mut self, keyword: u8) { self.0 &= !keyword; }
    #[inline(always)] pub fn clear(&mut self) { self.0 = 0; }
    #[inline(always)] pub fn has(&self, keyword: u8) -> bool { self.0 & keyword != 0 }

    // Builder pattern
    pub const fn with_rush(self) -> Self { Self(self.0 | Self::RUSH) }
    pub const fn with_ranged(self) -> Self { Self(self.0 | Self::RANGED) }
    pub const fn with_piercing(self) -> Self { Self(self.0 | Self::PIERCING) }
    pub const fn with_guard(self) -> Self { Self(self.0 | Self::GUARD) }
    pub const fn with_lifesteal(self) -> Self { Self(self.0 | Self::LIFESTEAL) }
    pub const fn with_lethal(self) -> Self { Self(self.0 | Self::LETHAL) }
    pub const fn with_shield(self) -> Self { Self(self.0 | Self::SHIELD) }
    pub const fn with_quick(self) -> Self { Self(self.0 | Self::QUICK) }

    // Combine keywords
    pub const fn union(self, other: Keywords) -> Keywords {
        Keywords(self.0 | other.0)
    }
}

// For serde serialization from YAML (list of keyword names)
impl Keywords {
    pub fn from_names(names: &[&str]) -> Self {
        let mut kw = Self::none();
        for name in names {
            match name.to_lowercase().as_str() {
                "rush" => kw.add(Self::RUSH),
                "ranged" => kw.add(Self::RANGED),
                "piercing" => kw.add(Self::PIERCING),
                "guard" => kw.add(Self::GUARD),
                "lifesteal" => kw.add(Self::LIFESTEAL),
                "lethal" => kw.add(Self::LETHAL),
                "shield" => kw.add(Self::SHIELD),
                "quick" => kw.add(Self::QUICK),
                _ => {} // Ignore unknown keywords
            }
        }
        kw
    }
}
```

**Acceptance criteria:**
- [ ] All keyword checks work correctly
- [ ] Builder pattern allows chaining: `Keywords::none().with_rush().with_guard()`
- [ ] `from_names` parses string lists correctly
- [ ] Size of Keywords is exactly 1 byte: `assert_eq!(std::mem::size_of::<Keywords>(), 1)`

---

### Task 1.4: Effect Types

**Priority:** High
**Depends on:** Task 1.1
**Can parallelize with:** 1.2, 1.3

**Objective:** Define effect and trigger enums for the effect system.

**File:** `src/effects.rs`

**Types to implement:**

```rust
use crate::types::{PlayerId, Slot, CardId};

/// Trigger conditions for abilities
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Trigger {
    OnPlay,
    OnAttack,
    OnDealDamage,
    OnTakeDamage,
    OnKill,
    OnDeath,
    StartOfTurn,
    EndOfTurn,
    OnAllyPlayed,
    OnAllyDeath,
}

/// What an effect targets
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EffectTarget {
    /// Specific creature
    Creature { owner: PlayerId, slot: Slot },
    /// A player (for face damage/healing)
    Player(PlayerId),
    /// All creatures on board
    AllCreatures,
    /// All friendly creatures
    AllAllyCreatures(PlayerId),
    /// All enemy creatures
    AllEnemyCreatures(PlayerId),
    /// The source of the trigger
    TriggerSource,
    /// No target (self-contained effect)
    None,
}

/// All possible effects in the game
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Effect {
    // Damage & Healing
    Damage { target: EffectTarget, amount: u8 },
    Heal { target: EffectTarget, amount: u8 },

    // Stat modification
    BuffStats { target: EffectTarget, attack: i8, health: i8 },
    SetStats { target: EffectTarget, attack: u8, health: u8 },

    // Card flow
    Draw { player: PlayerId, count: u8 },

    // Creature manipulation
    Destroy { target: EffectTarget },
    Summon { owner: PlayerId, card_id: CardId, slot: Option<Slot> },

    // Keyword manipulation
    GrantKeyword { target: EffectTarget, keyword: u8 },
    RemoveKeyword { target: EffectTarget, keyword: u8 },
    Silence { target: EffectTarget },

    // Resource manipulation
    GainEssence { player: PlayerId, amount: u8 },
    RefreshCreature { target: EffectTarget },
}

/// Source of an effect (for tracking/debugging)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EffectSource {
    Card(CardId),
    Creature { owner: PlayerId, slot: Slot },
    Support { owner: PlayerId, slot: Slot },
    System, // Game rules
}

/// Pending effect in the resolution queue
#[derive(Clone, Debug)]
pub struct PendingEffect {
    pub effect: Effect,
    pub source: EffectSource,
}

/// Targeting rules for spells and abilities
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub enum TargetingRule {
    #[default]
    NoTarget,
    TargetCreature(CreatureFilter),
    TargetAllyCreature,
    TargetEnemyCreature,
    TargetPlayer,
    TargetEnemyPlayer,
    TargetAny,
    TargetSlot,
}

/// Filter for creature targeting
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct CreatureFilter {
    pub max_health: Option<u8>,
    pub min_health: Option<u8>,
    pub has_keyword: Option<u8>,
    pub lacks_keyword: Option<u8>,
}

impl CreatureFilter {
    pub fn any() -> Self { Self::default() }

    pub fn with_max_health(mut self, max: u8) -> Self {
        self.max_health = Some(max);
        self
    }
}
```

**Acceptance criteria:**
- [ ] All effect types compile
- [ ] EffectTarget covers all targeting scenarios
- [ ] CreatureFilter builder pattern works

---

## Phase 2: Game State & Cards

### Task 2.1: Game State Structs

**Priority:** High
**Depends on:** Tasks 1.2, 1.3, 1.4
**Can parallelize with:** 2.2

**Objective:** Implement the complete game state representation.

**File:** `src/state.rs`

**Implementation requirements:**

```rust
use arrayvec::ArrayVec;
use crate::types::*;
use crate::keywords::Keywords;

/// Status flags for creatures (packed bitfield)
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CreatureStatus(pub u8);

impl CreatureStatus {
    pub const EXHAUSTED: u8 = 0b0000_0001;
    pub const SILENCED: u8  = 0b0000_0010;

    #[inline] pub fn is_exhausted(self) -> bool { self.0 & Self::EXHAUSTED != 0 }
    #[inline] pub fn is_silenced(self) -> bool { self.0 & Self::SILENCED != 0 }
    #[inline] pub fn set_exhausted(&mut self, val: bool) {
        if val { self.0 |= Self::EXHAUSTED; } else { self.0 &= !Self::EXHAUSTED; }
    }
    #[inline] pub fn set_silenced(&mut self, val: bool) {
        if val { self.0 |= Self::SILENCED; } else { self.0 &= !Self::SILENCED; }
    }
}

/// A creature on the battlefield
#[derive(Clone, Debug)]
pub struct Creature {
    pub instance_id: CreatureInstanceId,
    pub card_id: CardId,
    pub owner: PlayerId,
    pub slot: Slot,
    pub attack: i8,           // Current attack (can be negative from debuffs)
    pub current_health: i8,   // Current health
    pub max_health: i8,       // Maximum health (for healing cap)
    pub base_attack: u8,      // Original attack from card
    pub base_health: u8,      // Original health from card
    pub keywords: Keywords,
    pub status: CreatureStatus,
    pub turn_played: u16,
}

/// A support card on the battlefield
#[derive(Clone, Debug)]
pub struct Support {
    pub card_id: CardId,
    pub owner: PlayerId,
    pub slot: Slot,
    pub current_durability: u8,
}

/// A card instance (in hand or deck)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CardInstance {
    pub card_id: CardId,
}

/// Per-player state
#[derive(Clone, Debug)]
pub struct PlayerState {
    pub life: i16,
    pub max_essence: u8,
    pub current_essence: u8,
    pub action_points: u8,
    pub hand: ArrayVec<CardInstance, 10>,
    pub deck: ArrayVec<CardInstance, 30>,
    pub creatures: ArrayVec<Creature, 5>,
    pub supports: ArrayVec<Support, 2>,
    pub total_damage_dealt: u16,  // For victory points tracking
}

impl PlayerState {
    pub fn new() -> Self {
        Self {
            life: 30,
            max_essence: 0,
            current_essence: 0,
            action_points: 0,
            hand: ArrayVec::new(),
            deck: ArrayVec::new(),
            creatures: ArrayVec::new(),
            supports: ArrayVec::new(),
            total_damage_dealt: 0,
        }
    }
}

/// Game phase
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum GamePhase {
    #[default]
    Main,
    Ended,
}

/// Win reason
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WinReason {
    LifeReachedZero,
    TurnLimitHigherLife,
    VictoryPointsReached,
    Concession,
}

/// Game result
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameResult {
    Win { winner: PlayerId, reason: WinReason },
    Draw,
}

/// Complete game state
#[derive(Clone, Debug)]
pub struct GameState {
    pub players: [PlayerState; 2],
    pub current_turn: u16,
    pub active_player: PlayerId,
    pub phase: GamePhase,
    pub next_creature_id: u32,
    pub rng_state: u64,
    pub result: Option<GameResult>,
}

impl GameState {
    /// Get a creature by owner and slot
    pub fn get_creature(&self, owner: PlayerId, slot: Slot) -> Option<&Creature> {
        self.players[owner.index()].creatures.iter().find(|c| c.slot == slot)
    }

    /// Get a mutable creature by owner and slot
    pub fn get_creature_mut(&mut self, owner: PlayerId, slot: Slot) -> Option<&mut Creature> {
        self.players[owner.index()].creatures.iter_mut().find(|c| c.slot == slot)
    }

    /// Get a support by owner and slot
    pub fn get_support(&self, owner: PlayerId, slot: Slot) -> Option<&Support> {
        self.players[owner.index()].supports.iter().find(|s| s.slot == slot)
    }

    /// Check if game is over
    pub fn is_terminal(&self) -> bool {
        self.result.is_some()
    }

    /// Get active player's state
    pub fn active_player_state(&self) -> &PlayerState {
        &self.players[self.active_player.index()]
    }

    /// Get active player's state mutably
    pub fn active_player_state_mut(&mut self) -> &mut PlayerState {
        &mut self.players[self.active_player.index()]
    }
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            players: [PlayerState::new(), PlayerState::new()],
            current_turn: 0,
            active_player: PlayerId::PLAYER_ONE,
            phase: GamePhase::Main,
            next_creature_id: 0,
            rng_state: 0,
            result: None,
        }
    }
}
```

**Acceptance criteria:**
- [ ] GameState fits in reasonable memory (~1KB or less)
- [ ] All helper methods work correctly
- [ ] Clone is efficient (ArrayVec = no heap allocation)
- [ ] `cargo test` passes for state module

---

### Task 2.2: Card Definition Structs

**Priority:** High
**Depends on:** Tasks 1.2, 1.3, 1.4
**Can parallelize with:** 2.1

**Objective:** Define card definition structures for the card database.

**File:** `src/cards.rs`

**Implementation:**

```rust
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use crate::types::*;
use crate::keywords::Keywords;
use crate::effects::*;

/// Definition of a triggered ability
#[derive(Clone, Debug, Deserialize)]
pub struct AbilityDefinition {
    pub trigger: Trigger,
    pub targeting: TargetingRule,
    pub effects: Vec<EffectDefinition>,
}

/// Definition of an effect (serializable from YAML)
#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "type")]
pub enum EffectDefinition {
    Damage { amount: u8 },
    Heal { amount: u8 },
    Draw { count: u8 },
    BuffStats { attack: i8, health: i8 },
    Destroy,
    GrantKeyword { keyword: String },
    Silence,
}

/// Definition of a passive effect (for supports)
#[derive(Clone, Debug, Deserialize)]
pub struct PassiveEffectDefinition {
    pub modifier: PassiveModifier,
}

#[derive(Clone, Debug, Deserialize)]
pub enum PassiveModifier {
    AttackBonus(i8),
    HealthBonus(i8),
    GrantKeyword(String),
}

/// Card type with type-specific data
#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "card_type")]
pub enum CardType {
    Creature {
        attack: u8,
        health: u8,
        #[serde(default)]
        keywords: Vec<String>,
        #[serde(default)]
        abilities: Vec<AbilityDefinition>,
    },
    Spell {
        #[serde(default)]
        targeting: TargetingRule,
        effects: Vec<EffectDefinition>,
    },
    Support {
        durability: u8,
        #[serde(default)]
        passive_effects: Vec<PassiveEffectDefinition>,
        #[serde(default)]
        triggered_effects: Vec<AbilityDefinition>,
    },
}

/// Complete card definition
#[derive(Clone, Debug, Deserialize)]
pub struct CardDefinition {
    pub id: u16,
    pub name: String,
    pub cost: u8,
    #[serde(flatten)]
    pub card_type: CardType,
    #[serde(default)]
    pub rarity: Rarity,
    #[serde(default)]
    pub tags: Vec<String>,
}

impl CardDefinition {
    /// Get keywords for a creature card
    pub fn keywords(&self) -> Keywords {
        match &self.card_type {
            CardType::Creature { keywords, .. } => {
                let refs: Vec<&str> = keywords.iter().map(|s| s.as_str()).collect();
                Keywords::from_names(&refs)
            }
            _ => Keywords::none(),
        }
    }

    /// Get attack for a creature card
    pub fn attack(&self) -> Option<u8> {
        match &self.card_type {
            CardType::Creature { attack, .. } => Some(*attack),
            _ => None,
        }
    }

    /// Get health for a creature card
    pub fn health(&self) -> Option<u8> {
        match &self.card_type {
            CardType::Creature { health, .. } => Some(*health),
            _ => None,
        }
    }

    /// Check if this is a creature card
    pub fn is_creature(&self) -> bool {
        matches!(self.card_type, CardType::Creature { .. })
    }

    /// Check if this is a spell card
    pub fn is_spell(&self) -> bool {
        matches!(self.card_type, CardType::Spell { .. })
    }

    /// Check if this is a support card
    pub fn is_support(&self) -> bool {
        matches!(self.card_type, CardType::Support { .. })
    }
}

/// Card set loaded from YAML
#[derive(Clone, Debug, Deserialize)]
pub struct CardSet {
    pub name: String,
    pub cards: Vec<CardDefinition>,
}

/// The complete card database
#[derive(Clone)]
pub struct CardDatabase {
    cards: Arc<Vec<CardDefinition>>,
}

impl CardDatabase {
    /// Create a new database from a list of cards
    pub fn new(mut cards: Vec<CardDefinition>) -> Self {
        // Sort by ID for O(1) lookup
        cards.sort_by_key(|c| c.id);
        Self { cards: Arc::new(cards) }
    }

    /// Get a card by ID
    pub fn get(&self, id: CardId) -> Option<&CardDefinition> {
        self.cards.get(id.0 as usize)
    }

    /// Get total number of cards
    pub fn len(&self) -> usize {
        self.cards.len()
    }

    /// Check if database is empty
    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }

    /// Iterate over all cards
    pub fn iter(&self) -> impl Iterator<Item = &CardDefinition> {
        self.cards.iter()
    }
}
```

**Acceptance criteria:**
- [ ] CardDefinition can represent all card types
- [ ] Serde derives work for YAML parsing
- [ ] CardDatabase provides O(1) lookup by CardId
- [ ] All helper methods work correctly

---

### Task 2.3: YAML Card Loading

**Priority:** High
**Depends on:** Tasks 2.1, 2.2
**Can parallelize with:** None (needs 2.2)

**Objective:** Implement YAML file loading for card data.

**File:** `src/cards.rs` (extend existing)

**Additional implementation:**

```rust
use std::fs;
use std::path::Path;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CardLoadError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("YAML parse error: {0}")]
    Yaml(#[from] serde_yaml::Error),
    #[error("Card validation error: {0}")]
    Validation(String),
}

impl CardDatabase {
    /// Load cards from a directory of YAML files
    pub fn load_from_directory<P: AsRef<Path>>(path: P) -> Result<Self, CardLoadError> {
        let mut all_cards = Vec::new();

        let sets_path = path.as_ref().join("sets");
        if sets_path.exists() {
            for entry in fs::read_dir(sets_path)? {
                let entry = entry?;
                let file_path = entry.path();

                if file_path.extension().map_or(false, |ext| ext == "yaml" || ext == "yml") {
                    let yaml_content = fs::read_to_string(&file_path)?;
                    let card_set: CardSet = serde_yaml::from_str(&yaml_content)?;
                    all_cards.extend(card_set.cards);
                }
            }
        }

        // Validate no duplicate IDs
        let mut seen_ids = std::collections::HashSet::new();
        for card in &all_cards {
            if !seen_ids.insert(card.id) {
                return Err(CardLoadError::Validation(
                    format!("Duplicate card ID: {}", card.id)
                ));
            }
        }

        Ok(Self::new(all_cards))
    }

    /// Load cards from a single YAML string (useful for testing)
    pub fn load_from_yaml(yaml: &str) -> Result<Self, CardLoadError> {
        let card_set: CardSet = serde_yaml::from_str(yaml)?;
        Ok(Self::new(card_set.cards))
    }
}
```

**Also create:** `data/cards/sets/starter.yaml`

```yaml
name: "Starter Set"
cards:
  # 1-Cost Creatures
  - id: 1
    name: "Eager Recruit"
    cost: 1
    card_type: Creature
    attack: 2
    health: 1
    keywords: []
    tags: [Soldier]
    rarity: Common

  - id: 2
    name: "Village Guard"
    cost: 1
    card_type: Creature
    attack: 1
    health: 2
    keywords: []
    tags: [Soldier]
    rarity: Common

  - id: 3
    name: "Nimble Scout"
    cost: 1
    card_type: Creature
    attack: 1
    health: 1
    keywords: [Rush]
    tags: [Soldier]
    rarity: Common

  - id: 4
    name: "Toxic Spider"
    cost: 1
    card_type: Creature
    attack: 1
    health: 1
    keywords: [Lethal]
    tags: [Beast]
    rarity: Common

  # 2-Cost Creatures
  - id: 5
    name: "Iron Defender"
    cost: 2
    card_type: Creature
    attack: 1
    health: 4
    keywords: [Guard]
    tags: [Soldier]
    rarity: Common

  - id: 6
    name: "Frontier Ranger"
    cost: 2
    card_type: Creature
    attack: 2
    health: 2
    keywords: [Ranged]
    tags: [Soldier]
    rarity: Common

  - id: 7
    name: "Young Knight"
    cost: 2
    card_type: Creature
    attack: 2
    health: 3
    keywords: []
    tags: [Soldier]
    rarity: Common

  # ... (continue with all 43 cards from cards.md)

  # Spells
  - id: 32
    name: "Quick Strike"
    cost: 1
    card_type: Spell
    targeting: TargetCreature
    effects:
      - type: Damage
        amount: 2
    rarity: Common

  - id: 33
    name: "Arcane Intellect"
    cost: 3
    card_type: Spell
    targeting: NoTarget
    effects:
      - type: Draw
        count: 2
    rarity: Common

  # Supports
  - id: 40
    name: "War Drums"
    cost: 3
    card_type: Support
    durability: 3
    passive_effects:
      - modifier:
          AttackBonus: 1
    rarity: Uncommon
```

**Acceptance criteria:**
- [ ] `CardDatabase::load_from_directory()` successfully loads YAML files
- [ ] Duplicate ID detection works
- [ ] At least 10 starter cards are defined in YAML
- [ ] `cargo test` passes for loading

---

## Phase 3: Actions & Legal Move Generation

### Task 3.1: Action Enum & Indexing

**Priority:** High
**Depends on:** Tasks 1.2, 2.1
**Can parallelize with:** None

**Objective:** Implement the Action enum with bidirectional index mapping.

**File:** `src/actions.rs`

**Implementation:**

```rust
use crate::types::*;

/// Maximum action space size (power of 2 for efficient masking)
pub const MAX_ACTIONS: usize = 256;

/// Target for playing a card
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PlayTarget {
    /// No target needed
    None,
    /// Place creature/support in specific slot
    Slot(Slot),
    /// Target a creature
    Creature { owner: PlayerId, slot: Slot },
    /// Target a player
    Player(PlayerId),
}

/// Target for attacks
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AttackTarget {
    /// Attack enemy creature at slot
    Creature(Slot),
    /// Attack enemy face
    Face,
}

/// All possible player actions
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Action {
    /// End the current turn
    EndTurn,

    /// Play a card from hand
    PlayCard {
        hand_index: u8,
        target: PlayTarget,
    },

    /// Attack with a creature
    Attack {
        attacker_slot: Slot,
        target: AttackTarget,
    },
}

impl Action {
    /// Convert action to flat index for neural network output
    pub fn to_index(&self) -> usize {
        match self {
            Action::EndTurn => 0,

            Action::PlayCard { hand_index, target } => {
                let hand_offset = (*hand_index as usize) * 18; // 18 possible targets per hand slot
                let target_offset = match target {
                    PlayTarget::None => 0,
                    PlayTarget::Slot(s) => 1 + s.0 as usize,  // 1-5
                    PlayTarget::Creature { owner, slot } => {
                        6 + owner.0 as usize * 5 + slot.0 as usize  // 6-15
                    }
                    PlayTarget::Player(p) => 16 + p.0 as usize,  // 16-17
                };
                1 + hand_offset + target_offset  // Actions 1-181
            }

            Action::Attack { attacker_slot, target } => {
                let slot_offset = attacker_slot.0 as usize * 6;
                let target_offset = match target {
                    AttackTarget::Face => 0,
                    AttackTarget::Creature(s) => 1 + s.0 as usize,
                };
                182 + slot_offset + target_offset  // Actions 182-211
            }
        }
    }

    /// Convert flat index back to Action
    pub fn from_index(index: usize) -> Option<Action> {
        match index {
            0 => Some(Action::EndTurn),

            1..=181 => {
                let idx = index - 1;
                let hand_index = (idx / 18) as u8;
                let target_idx = idx % 18;

                let target = match target_idx {
                    0 => PlayTarget::None,
                    1..=5 => PlayTarget::Slot(Slot((target_idx - 1) as u8)),
                    6..=15 => {
                        let adj = target_idx - 6;
                        PlayTarget::Creature {
                            owner: PlayerId((adj / 5) as u8),
                            slot: Slot((adj % 5) as u8),
                        }
                    }
                    16..=17 => PlayTarget::Player(PlayerId((target_idx - 16) as u8)),
                    _ => return None,
                };

                Some(Action::PlayCard { hand_index, target })
            }

            182..=211 => {
                let idx = index - 182;
                let attacker_slot = Slot((idx / 6) as u8);
                let target_idx = idx % 6;

                let target = if target_idx == 0 {
                    AttackTarget::Face
                } else {
                    AttackTarget::Creature(Slot((target_idx - 1) as u8))
                };

                Some(Action::Attack { attacker_slot, target })
            }

            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_index_roundtrip() {
        // Test EndTurn
        let action = Action::EndTurn;
        assert_eq!(Action::from_index(action.to_index()), Some(action));

        // Test PlayCard
        let action = Action::PlayCard {
            hand_index: 3,
            target: PlayTarget::Slot(Slot(2)),
        };
        assert_eq!(Action::from_index(action.to_index()), Some(action));

        // Test Attack
        let action = Action::Attack {
            attacker_slot: Slot(1),
            target: AttackTarget::Creature(Slot(3)),
        };
        assert_eq!(Action::from_index(action.to_index()), Some(action));
    }
}
```

**Acceptance criteria:**
- [ ] `to_index()` produces unique indices for all actions
- [ ] `from_index(to_index(a)) == Some(a)` for all valid actions
- [ ] All indices fit within MAX_ACTIONS (256)
- [ ] Tests pass

---

### Task 3.2: Legal Action Generation

**Priority:** High
**Depends on:** Tasks 2.1, 2.2, 3.1
**Can parallelize with:** None

**Objective:** Implement legal action enumeration.

**File:** `src/legal.rs`

**Implementation:**

```rust
use crate::actions::*;
use crate::state::*;
use crate::cards::*;
use crate::types::*;
use crate::keywords::Keywords;

/// Generate all legal actions for the current player
pub fn get_legal_actions(state: &GameState, db: &CardDatabase) -> Vec<Action> {
    if state.is_terminal() {
        return vec![];
    }

    let mut actions = Vec::with_capacity(32);
    let player = state.active_player;
    let p = &state.players[player.index()];

    // EndTurn is always legal
    actions.push(Action::EndTurn);

    // Need AP for other actions
    if p.action_points == 0 {
        return actions;
    }

    // Check playable cards
    for (hand_idx, card_instance) in p.hand.iter().enumerate() {
        if let Some(card) = db.get(card_instance.card_id) {
            // Check essence cost
            if card.cost > p.current_essence {
                continue;
            }

            // Get valid targets for this card
            for target in get_play_targets(state, card, player) {
                actions.push(Action::PlayCard {
                    hand_index: hand_idx as u8,
                    target,
                });
            }
        }
    }

    // Check attack actions
    for creature in &p.creatures {
        if can_creature_attack(creature, state) {
            for target in get_attack_targets(creature, state) {
                actions.push(Action::Attack {
                    attacker_slot: creature.slot,
                    target,
                });
            }
        }
    }

    actions
}

/// Generate legal action mask for neural network
pub fn get_legal_action_mask(state: &GameState, db: &CardDatabase) -> [bool; MAX_ACTIONS] {
    let mut mask = [false; MAX_ACTIONS];

    for action in get_legal_actions(state, db) {
        let idx = action.to_index();
        if idx < MAX_ACTIONS {
            mask[idx] = true;
        }
    }

    mask
}

/// Get valid targets for playing a card
fn get_play_targets(state: &GameState, card: &CardDefinition, player: PlayerId) -> Vec<PlayTarget> {
    let mut targets = Vec::new();
    let p = &state.players[player.index()];

    match &card.card_type {
        CardType::Creature { .. } => {
            // Find empty creature slots
            let occupied: Vec<u8> = p.creatures.iter().map(|c| c.slot.0).collect();
            for slot_idx in 0..5u8 {
                if !occupied.contains(&slot_idx) {
                    targets.push(PlayTarget::Slot(Slot(slot_idx)));
                }
            }
        }

        CardType::Spell { targeting, .. } => {
            match targeting {
                TargetingRule::NoTarget => {
                    targets.push(PlayTarget::None);
                }
                TargetingRule::TargetCreature(filter) => {
                    // All creatures matching filter
                    for pid in [PlayerId::PLAYER_ONE, PlayerId::PLAYER_TWO] {
                        for creature in &state.players[pid.index()].creatures {
                            if matches_filter(creature, filter) {
                                targets.push(PlayTarget::Creature {
                                    owner: pid,
                                    slot: creature.slot,
                                });
                            }
                        }
                    }
                }
                TargetingRule::TargetAllyCreature => {
                    for creature in &p.creatures {
                        targets.push(PlayTarget::Creature {
                            owner: player,
                            slot: creature.slot,
                        });
                    }
                }
                TargetingRule::TargetEnemyCreature => {
                    let enemy = player.opponent();
                    for creature in &state.players[enemy.index()].creatures {
                        targets.push(PlayTarget::Creature {
                            owner: enemy,
                            slot: creature.slot,
                        });
                    }
                }
                TargetingRule::TargetPlayer => {
                    targets.push(PlayTarget::Player(PlayerId::PLAYER_ONE));
                    targets.push(PlayTarget::Player(PlayerId::PLAYER_TWO));
                }
                TargetingRule::TargetEnemyPlayer => {
                    targets.push(PlayTarget::Player(player.opponent()));
                }
                TargetingRule::TargetAny => {
                    // All creatures + players
                    for pid in [PlayerId::PLAYER_ONE, PlayerId::PLAYER_TWO] {
                        targets.push(PlayTarget::Player(pid));
                        for creature in &state.players[pid.index()].creatures {
                            targets.push(PlayTarget::Creature {
                                owner: pid,
                                slot: creature.slot,
                            });
                        }
                    }
                }
                TargetingRule::TargetSlot => {
                    let occupied: Vec<u8> = p.creatures.iter().map(|c| c.slot.0).collect();
                    for slot_idx in 0..5u8 {
                        if !occupied.contains(&slot_idx) {
                            targets.push(PlayTarget::Slot(Slot(slot_idx)));
                        }
                    }
                }
            }
        }

        CardType::Support { .. } => {
            // Check if there's an empty support slot
            if p.supports.len() < 2 {
                targets.push(PlayTarget::None);
            }
        }
    }

    targets
}

/// Check if a creature can attack
fn can_creature_attack(creature: &Creature, state: &GameState) -> bool {
    // Check exhaustion
    if creature.status.is_exhausted() {
        return false;
    }

    // Check summoning sickness (unless Rush)
    if creature.turn_played == state.current_turn && !creature.keywords.has_rush() {
        return false;
    }

    // Check attack value
    if creature.attack <= 0 {
        return false;
    }

    true
}

/// Get valid attack targets for a creature
fn get_attack_targets(creature: &Creature, state: &GameState) -> Vec<AttackTarget> {
    let mut targets = Vec::new();
    let enemy = creature.owner.opponent();
    let enemy_creatures = &state.players[enemy.index()].creatures;

    if creature.keywords.has_ranged() {
        // Ranged: can attack any enemy creature
        // But still must respect Guard
        let guards: Vec<Slot> = enemy_creatures
            .iter()
            .filter(|c| c.keywords.has_guard())
            .map(|c| c.slot)
            .collect();

        if !guards.is_empty() {
            // Must attack a Guard
            for slot in guards {
                targets.push(AttackTarget::Creature(slot));
            }
        } else {
            // Can attack any creature
            for enemy_creature in enemy_creatures {
                targets.push(AttackTarget::Creature(enemy_creature.slot));
            }

            // Can attack face if direct lane is empty
            let direct_lane_empty = !enemy_creatures.iter().any(|c| c.slot == creature.slot);
            if direct_lane_empty {
                targets.push(AttackTarget::Face);
            }
        }
    } else {
        // Normal: only adjacent lanes
        let adjacent = creature.slot.adjacent_slots();

        // Find Guards in range
        let guards_in_range: Vec<Slot> = enemy_creatures
            .iter()
            .filter(|c| adjacent.contains(&c.slot) && c.keywords.has_guard())
            .map(|c| c.slot)
            .collect();

        if !guards_in_range.is_empty() {
            // Must attack a Guard in range
            for slot in guards_in_range {
                targets.push(AttackTarget::Creature(slot));
            }
        } else {
            // Can attack any adjacent creature
            for enemy_creature in enemy_creatures {
                if adjacent.contains(&enemy_creature.slot) {
                    targets.push(AttackTarget::Creature(enemy_creature.slot));
                }
            }

            // Can attack face if direct lane is empty
            let direct_lane_empty = !enemy_creatures.iter().any(|c| c.slot == creature.slot);
            if direct_lane_empty {
                targets.push(AttackTarget::Face);
            }
        }
    }

    targets
}

/// Check if a creature matches a filter
fn matches_filter(creature: &Creature, filter: &CreatureFilter) -> bool {
    if let Some(max_hp) = filter.max_health {
        if creature.current_health > max_hp as i8 {
            return false;
        }
    }
    if let Some(min_hp) = filter.min_health {
        if creature.current_health < min_hp as i8 {
            return false;
        }
    }
    if let Some(kw) = filter.has_keyword {
        if !creature.keywords.has(kw) {
            return false;
        }
    }
    if let Some(kw) = filter.lacks_keyword {
        if creature.keywords.has(kw) {
            return false;
        }
    }
    true
}
```

**Acceptance criteria:**
- [ ] EndTurn always included when game not over
- [ ] Only playable cards (enough essence, valid targets) are included
- [ ] Only attackable creatures (not exhausted, no summoning sickness) can attack
- [ ] Guard rules correctly enforced
- [ ] Ranged bypass of lane restrictions works
- [ ] Face attacks only when direct lane is empty

---

## Phase 4: Game Engine Core

### Task 4.1: Turn Structure

**Priority:** High
**Depends on:** Phase 2, Phase 3
**Can parallelize with:** 4.2

**Objective:** Implement turn start/end logic and game initialization.

**File:** `src/engine.rs`

**Implementation requirements:**
- `new_game()` - Create initial game state with shuffled decks
- `start_turn()` - Essence gain, AP reset, draw card, refresh creatures, tick supports
- `end_turn()` - Trigger end of turn effects, switch player, check turn limit
- `shuffle_deck()` - Deterministic shuffle using xorshift64 RNG
- `draw_card()` - Draw from deck to hand

**Key rules from DESIGN.md:**
- Player 1 skips draw on turn 1
- Essence increases by 1 each turn (cap at 10)
- AP resets to 3
- Support durability decreases by 1, removed if 0
- Turn limit is 30

---

### Task 4.2: Effect Queue System

**Priority:** High
**Depends on:** Tasks 1.4, 2.1
**Can parallelize with:** 4.1

**Objective:** Implement the queue-based effect resolution system.

**File:** `src/effects.rs` (extend existing)

**Implementation requirements:**
- `EffectQueue` struct with VecDeque<PendingEffect>
- `queue_effect()` - Add effect to queue
- `resolve_all()` - Process queue until empty
- `resolve_single()` - Handle one effect
- `cleanup_deaths()` - Remove dead creatures, trigger OnDeath
- Handlers for each Effect variant

---

### Task 4.3: Card Playing Logic

**Priority:** High
**Depends on:** Tasks 4.1, 4.2, 2.2
**Can parallelize with:** None

**Objective:** Implement card playing from hand.

**File:** `src/engine.rs` (extend)

**Implementation requirements:**
- `play_card()` - Main entry point
- Handle creature placement (create Creature, add to board)
- Handle spell resolution (queue effects, resolve)
- Handle support placement (create Support, add to board)
- Apply support passive effects to creatures
- Trigger OnPlay abilities

---

## Phase 5: Combat System

### Task 5.1: Basic Combat Resolution

**Priority:** High
**Depends on:** Tasks 4.1, 4.2
**Can parallelize with:** None

**Objective:** Implement basic attack resolution without keyword interactions.

**File:** `src/combat.rs`

**Implementation requirements:**
- `resolve_attack()` - Main entry point
- `attack_face()` - Direct damage to player
- `attack_creature()` - Creature vs creature combat
- Simultaneous damage by default
- Mark attacker as exhausted
- Trigger OnAttack

---

### Task 5.2: Keyword Combat Interactions

**Priority:** High
**Depends on:** Task 5.1
**Can parallelize with:** None

**Objective:** Implement all keyword interactions in combat.

**File:** `src/combat.rs` (extend)

**Implementation requirements per DESIGN.md:**

| Keyword | Implementation |
|---------|----------------|
| Rush | Already handled in legal.rs (summoning sickness bypass) |
| Ranged | Already handled in legal.rs (target selection) |
| Guard | Already handled in legal.rs (forced targeting) |
| Quick | Deal damage first, opponent only retaliates if survives |
| Shield | First damage prevented, then remove Shield |
| Lethal | If damage > 0 dealt, target dies regardless of health |
| Lifesteal | If damage_actually_dealt > 0, heal owner |
| Piercing | If target dies, excess damage to face |

**Critical interactions:**
- Shield vs Lethal: Shield absorbs → no Lethal trigger
- Shield vs Lifesteal: Shield absorbs → no heal
- Piercing vs Guard: Piercing only if Guard dies
- Quick vs Quick: Simultaneous

---

## Phase 6: AI Interface

### Task 6.1: State Tensor Conversion

**Priority:** Medium
**Depends on:** Task 2.1
**Can parallelize with:** 6.2

**Objective:** Convert GameState to fixed-size tensor for neural networks.

**File:** `src/tensor.rs`

**Implementation per DESIGN.md section 11.1:**
- Global state: 10 floats
- Creature slots: 10 slots × 12 features = 120 floats
- Support slots: 4 slots × 4 features = 16 floats
- Hand encoding: 20 slots × 3 features = 60 floats
- Deck encoding: 60 slots × 2 features = 120 floats
- Total: ~326 floats (pad to 512)

Output raw card IDs as integers (embedding handled in Python).

---

### Task 6.2: Game Interface Trait

**Priority:** Medium
**Depends on:** All engine tasks
**Can parallelize with:** 6.1

**Objective:** Define the public API trait for AI agents.

**File:** `src/lib.rs` (public API)

**Trait definition:**
```rust
pub trait GameInterface {
    fn new_game(&self, deck1: &[CardId], deck2: &[CardId], seed: u64) -> GameState;
    fn get_legal_actions(&self, state: &GameState) -> Vec<Action>;
    fn get_legal_action_mask(&self, state: &GameState) -> [bool; MAX_ACTIONS];
    fn apply_action(&mut self, state: &mut GameState, action: Action);
    fn is_terminal(&self, state: &GameState) -> bool;
    fn get_result(&self, state: &GameState) -> Option<GameResult>;
    fn state_to_tensor(&self, state: &GameState) -> Vec<f32>;
    fn get_reward(&self, state: &GameState, player: PlayerId) -> f32;
}
```

---

## Phase 7: Testing & Validation

### Task 7.1: Keyword Unit Tests

**Priority:** Medium
**Depends on:** Phase 5

**File:** `tests/keywords.rs`

Test all keyword interactions from DESIGN.md section 7.2.

---

### Task 7.2: Combat Integration Tests

**Priority:** Medium
**Depends on:** Phase 5

**File:** `tests/combat.rs`

Test combat scenarios from cards.md "Key Interactions to Verify".

---

### Task 7.3: Full Game Simulation Tests

**Priority:** Medium
**Depends on:** All phases

**File:** `tests/simulation.rs`

- Test complete games with random actions
- Verify no panics or invalid states
- Verify games terminate within turn limit

---

## Execution Order Summary

```
Sequential Order (respecting dependencies):
═══════════════════════════════════════════

BATCH 1: Task 1.1 (Project Setup)
   ↓
BATCH 2: Tasks 1.2, 1.3, 1.4 (Core Types - parallel)
   ↓
BATCH 3: Tasks 2.1, 2.2 (State & Cards - parallel)
   ↓
BATCH 4: Task 2.3 (YAML Loading)
   ↓
BATCH 5: Task 3.1 (Actions)
   ↓
BATCH 6: Task 3.2 (Legal Actions)
   ↓
BATCH 7: Tasks 4.1, 4.2 (Turn Structure & Effects - parallel)
   ↓
BATCH 8: Task 4.3 (Card Playing)
   ↓
BATCH 9: Task 5.1 (Basic Combat)
   ↓
BATCH 10: Task 5.2 (Keyword Combat)
   ↓
BATCH 11: Tasks 6.1, 6.2 (AI Interface - parallel)
   ↓
BATCH 12: Tasks 7.1, 7.2, 7.3 (Testing - parallel)
```

---

## Agent Dispatch Notes

When dispatching Task agents:

1. **Provide context:** Each agent needs access to DESIGN.md for rules
2. **Provide dependencies:** Tell agent which files already exist
3. **Be specific:** Give exact file paths and function signatures
4. **Request tests:** Ask agent to include unit tests in the same file
5. **Review output:** Senior dev (Claude) reviews before proceeding

---

*End of Implementation Plan*
