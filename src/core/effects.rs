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

/// All possible effects in the game
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Effect {
    // === Damage & Healing ===
    /// Deal damage to target
    Damage { target: EffectTarget, amount: u8 },
    /// Heal target
    Heal { target: EffectTarget, amount: u8 },

    // === Stat Modification ===
    /// Buff attack and/or health (can be negative for debuffs)
    BuffStats { target: EffectTarget, attack: i8, health: i8 },
    /// Set stats to specific values
    SetStats { target: EffectTarget, attack: u8, health: u8 },

    // === Card Flow ===
    /// Draw cards
    Draw { player: PlayerId, count: u8 },

    // === Creature Manipulation ===
    /// Destroy target creature
    Destroy { target: EffectTarget },
    /// Summon a creature (slot None = first available)
    Summon { owner: PlayerId, card_id: CardId, slot: Option<Slot> },

    // === Keyword Manipulation ===
    /// Grant a keyword to target
    GrantKeyword { target: EffectTarget, keyword: u8 },
    /// Remove a keyword from target
    RemoveKeyword { target: EffectTarget, keyword: u8 },
    /// Silence target (remove all keywords and abilities)
    Silence { target: EffectTarget },

    // === Resource Manipulation ===
    /// Gain essence this turn
    GainEssence { player: PlayerId, amount: u8 },
    /// Refresh a creature (remove exhausted status)
    RefreshCreature { target: EffectTarget },
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
    pub has_keyword: Option<u8>,
    /// Must NOT have this keyword
    pub lacks_keyword: Option<u8>,
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
    pub fn with_keyword(mut self, keyword: u8) -> Self {
        self.has_keyword = Some(keyword);
        self
    }

    /// Builder: must lack keyword
    pub fn without_keyword(mut self, keyword: u8) -> Self {
        self.lacks_keyword = Some(keyword);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_effect_target_variants() {
        let target = EffectTarget::Creature {
            owner: PlayerId(0),
            slot: Slot(2),
        };

        match target {
            EffectTarget::Creature { owner, slot } => {
                assert_eq!(owner, PlayerId(0));
                assert_eq!(slot, Slot(2));
            }
            _ => panic!("Wrong variant"),
        }
    }

    #[test]
    fn test_pending_effect_creation() {
        let effect = Effect::Damage {
            target: EffectTarget::Player(PlayerId(1)),
            amount: 5,
        };
        let source = EffectSource::System;

        let pending = PendingEffect::new(effect.clone(), source);
        assert_eq!(pending.effect, effect);
        assert_eq!(pending.source, source);
    }

    #[test]
    fn test_creature_filter_builder() {
        let filter = CreatureFilter::any()
            .with_max_health(4)
            .without_keyword(0b0000_1000); // Guard

        assert_eq!(filter.max_health, Some(4));
        assert_eq!(filter.lacks_keyword, Some(0b0000_1000));
        assert_eq!(filter.min_health, None);
    }

    #[test]
    fn test_targeting_rule_default() {
        let rule: TargetingRule = Default::default();
        assert_eq!(rule, TargetingRule::NoTarget);
    }

    #[test]
    fn test_trigger_variants() {
        // Just ensure all variants exist and are distinct
        let triggers = [
            Trigger::OnPlay,
            Trigger::OnAttack,
            Trigger::OnDealDamage,
            Trigger::OnTakeDamage,
            Trigger::OnKill,
            Trigger::OnDeath,
            Trigger::StartOfTurn,
            Trigger::EndOfTurn,
            Trigger::OnAllyPlayed,
            Trigger::OnAllyDeath,
        ];

        // Verify all are unique
        for (i, t1) in triggers.iter().enumerate() {
            for (j, t2) in triggers.iter().enumerate() {
                if i != j {
                    assert_ne!(t1, t2);
                }
            }
        }
    }
}
