//! Effect definitions, triggers, and the effect queue system.
//!
//! Effects use a queue-based resolution system where effects are added to a queue
//! and processed one at a time. This avoids recursion and makes resolution predictable.

use serde::{Deserialize, Serialize};

// Note: We'll use forward declarations here since types.rs defines these.
// The actual imports will work once types.rs is implemented.
use crate::core::types::{PlayerId, Slot, CardId};

/// Trigger conditions for abilities
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Trigger {
    /// When this card is played from hand
    OnPlay,
    /// When this creature declares an attack
    OnAttack,
    /// When this creature deals damage
    OnDealDamage,
    /// When this creature takes damage
    OnTakeDamage,
    /// When this creature kills another creature
    OnKill,
    /// When this creature dies
    OnDeath,
    /// At the start of owner's turn
    StartOfTurn,
    /// At the end of owner's turn
    EndOfTurn,
    /// When another friendly creature is played
    OnAllyPlayed,
    /// When another friendly creature dies
    OnAllyDeath,
}

/// What an effect targets
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EffectTarget {
    /// Specific creature on the board
    Creature { owner: PlayerId, slot: Slot },
    /// A player (for face damage/healing)
    Player(PlayerId),
    /// All creatures on the board
    AllCreatures,
    /// All friendly creatures of specified player
    AllAllyCreatures(PlayerId),
    /// All enemy creatures (relative to specified player)
    AllEnemyCreatures(PlayerId),
    /// The source of the trigger (self-referential)
    TriggerSource,
    /// No target (for effects that don't need one)
    None,
}

/// Token definition for summoning creatures that don't exist in the card database
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TokenDefinition {
    /// Display name for the token
    pub name: String,
    /// Attack value
    pub attack: u8,
    /// Health value
    pub health: u8,
    /// Keywords as a bitmask
    #[serde(default)]
    pub keywords: u16,
}

impl TokenDefinition {
    /// Create a new token definition
    pub fn new(name: impl Into<String>, attack: u8, health: u8) -> Self {
        Self {
            name: name.into(),
            attack,
            health,
            keywords: 0,
        }
    }

    /// Add a keyword to the token
    pub fn with_keyword(mut self, keyword: u16) -> Self {
        self.keywords |= keyword;
        self
    }
}

/// All possible effects in the game
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Effect {
    // === Damage & Healing ===
    /// Deal damage to target (filter applies to AoE targets)
    Damage { target: EffectTarget, amount: u8, filter: Option<CreatureFilter> },
    /// Heal target (filter applies to AoE targets)
    Heal { target: EffectTarget, amount: u8, filter: Option<CreatureFilter> },

    // === Stat Modification ===
    /// Buff attack and/or health (can be negative for debuffs, filter applies to AoE targets)
    BuffStats { target: EffectTarget, attack: i8, health: i8, filter: Option<CreatureFilter> },
    /// Set stats to specific values
    SetStats { target: EffectTarget, attack: u8, health: u8 },

    // === Card Flow ===
    /// Draw cards
    Draw { player: PlayerId, count: u8 },

    // === Creature Manipulation ===
    /// Destroy target creature (filter applies to AoE targets)
    Destroy { target: EffectTarget, filter: Option<CreatureFilter> },
    /// Summon a creature from card database (slot None = first available)
    Summon { owner: PlayerId, card_id: CardId, slot: Option<Slot> },
    /// Summon a token creature (not from card database)
    SummonToken { owner: PlayerId, token: TokenDefinition, slot: Option<Slot> },
    /// Transform target creature into a token (replaces the creature)
    Transform { target: EffectTarget, into: TokenDefinition },
    /// Create a copy of target creature in an empty slot
    Copy { target: EffectTarget, owner: PlayerId },

    // === Keyword Manipulation ===
    /// Grant a keyword to target (filter applies to AoE targets)
    GrantKeyword { target: EffectTarget, keyword: u16, filter: Option<CreatureFilter> },
    /// Remove a keyword from target (filter applies to AoE targets)
    RemoveKeyword { target: EffectTarget, keyword: u16, filter: Option<CreatureFilter> },
    /// Silence target (remove all keywords and abilities, filter applies to AoE targets)
    Silence { target: EffectTarget, filter: Option<CreatureFilter> },

    // === Resource Manipulation ===
    /// Gain essence this turn
    GainEssence { player: PlayerId, amount: u8 },
    /// Refresh a creature (remove exhausted status)
    RefreshCreature { target: EffectTarget },

    // === Board Manipulation ===
    /// Return target creature to its owner's hand (filter applies to AoE targets)
    Bounce { target: EffectTarget, filter: Option<CreatureFilter> },
}

/// Source of an effect (for tracking and debugging)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EffectSource {
    /// Effect from a card being played
    Card(CardId),
    /// Effect from a creature's ability
    Creature { owner: PlayerId, slot: Slot },
    /// Effect from a support card
    Support { owner: PlayerId, slot: Slot },
    /// Effect from game rules (start of turn draw, etc.)
    System,
}

/// A pending effect in the resolution queue
#[derive(Clone, Debug)]
pub struct PendingEffect {
    pub effect: Effect,
    pub source: EffectSource,
}

impl PendingEffect {
    pub fn new(effect: Effect, source: EffectSource) -> Self {
        Self { effect, source }
    }
}

// === Conditional Triggers ===

/// Conditions that can be checked after an effect resolves
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Condition {
    /// The primary target of the effect died
    TargetDied,
    // Future: TargetSurvived, AllyCountAtLeast(u8), LifeBelow(u8), etc.
}

/// Result of applying an effect (for conditional triggers)
#[derive(Clone, Debug, Default)]
pub struct EffectResult {
    /// Whether the primary target died from the effect
    pub target_died: bool,
    // Future: damage_dealt: u8, creatures_destroyed: u8, etc.
}

impl EffectResult {
    /// Create an empty result with no triggered conditions
    pub fn none() -> Self {
        Self::default()
    }

    /// Check if a condition is satisfied by this result
    pub fn check(&self, condition: &Condition) -> bool {
        match condition {
            Condition::TargetDied => self.target_died,
        }
    }
}

// === Targeting Rules (for spells and abilities) ===

/// Rules for what a spell or ability can target
#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum TargetingRule {
    /// No target needed (effect is automatic)
    #[default]
    NoTarget,
    /// Target any creature matching the filter
    TargetCreature(CreatureFilter),
    /// Target only friendly creatures
    TargetAllyCreature,
    /// Target only enemy creatures
    TargetEnemyCreature,
    /// Target any player
    TargetPlayer,
    /// Target enemy player only
    TargetEnemyPlayer,
    /// Target any creature or player
    TargetAny,
    /// Target an empty slot (for summon effects)
    TargetSlot,
}

/// Filter for creature targeting
#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct CreatureFilter {
    /// Maximum health (e.g., for "destroy creature with 4 or less health")
    pub max_health: Option<u8>,
    /// Minimum health
    pub min_health: Option<u8>,
    /// Must have this keyword
    pub has_keyword: Option<u16>,
    /// Must NOT have this keyword
    pub lacks_keyword: Option<u16>,
}

impl CreatureFilter {
    /// No filter - matches any creature
    pub fn any() -> Self {
        Self::default()
    }

    /// Builder: set max health
    pub fn with_max_health(mut self, max: u8) -> Self {
        self.max_health = Some(max);
        self
    }

    /// Builder: set min health
    pub fn with_min_health(mut self, min: u8) -> Self {
        self.min_health = Some(min);
        self
    }

    /// Builder: must have keyword
    pub fn with_keyword(mut self, keyword: u16) -> Self {
        self.has_keyword = Some(keyword);
        self
    }

    /// Builder: must lack keyword
    pub fn without_keyword(mut self, keyword: u16) -> Self {
        self.lacks_keyword = Some(keyword);
        self
    }

    /// Check if a creature matches this filter.
    /// Takes creature values directly to avoid circular module dependencies.
    ///
    /// # Arguments
    /// * `health` - Current health of the creature
    /// * `keywords` - Keyword bitfield of the creature (u16)
    pub fn matches(&self, health: i8, keywords: u16) -> bool {
        // Check max health filter
        if let Some(max) = self.max_health {
            if health > max as i8 {
                return false;
            }
        }

        // Check min health filter
        if let Some(min) = self.min_health {
            if health < min as i8 {
                return false;
            }
        }

        // Check has_keyword filter
        if let Some(required_keyword) = self.has_keyword {
            if keywords & required_keyword == 0 {
                return false;
            }
        }

        // Check lacks_keyword filter
        if let Some(forbidden_keyword) = self.lacks_keyword {
            if keywords & forbidden_keyword != 0 {
                return false;
            }
        }

        true
    }

    /// Check if this filter is empty (matches any creature)
    pub fn is_empty(&self) -> bool {
        self.max_health.is_none()
            && self.min_health.is_none()
            && self.has_keyword.is_none()
            && self.lacks_keyword.is_none()
    }
}
