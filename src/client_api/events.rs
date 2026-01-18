//! Game events for reactive UI updates.
//!
//! GameEvent represents all state changes that occur during a game.
//! These events are emitted by GameClient to enable reactive UIs
//! without needing to poll or diff the entire game state.

use serde::{Deserialize, Serialize};
use crate::core::actions::Action;
use crate::core::keywords::Keywords;
use crate::core::state::{GameMode, GameResult};
use crate::core::types::{CardId, CreatureInstanceId, PlayerId, Slot};

/// A game event representing a state change.
///
/// Events are emitted in order and can be used to:
/// - Update a UI incrementally without full state diffs
/// - Build a replay log
/// - Debug game flow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GameEvent {
    // =========================================================================
    // GAME LIFECYCLE
    // =========================================================================

    /// Game has been initialized and is ready to play.
    GameStarted {
        seed: u64,
        mode: GameMode,
        player1_deck_size: usize,
        player2_deck_size: usize,
    },

    /// Game has ended with a result.
    GameEnded {
        result: GameResult,
        final_turn: u16,
    },

    // =========================================================================
    // TURN STRUCTURE
    // =========================================================================

    /// A new turn has started.
    TurnStarted {
        player: PlayerId,
        turn_number: u16,
        max_essence: u8,
        action_points: u8,
    },

    /// A turn has ended.
    TurnEnded {
        player: PlayerId,
        turn_number: u16,
    },

    // =========================================================================
    // ACTIONS
    // =========================================================================

    /// A player has taken an action.
    ActionTaken {
        player: PlayerId,
        action: Action,
        turn: u16,
    },

    // =========================================================================
    // CARDS IN HAND
    // =========================================================================

    /// A card was drawn from deck to hand.
    CardDrawn {
        player: PlayerId,
        card_id: CardId,
        hand_position: usize,
        cards_remaining_in_deck: usize,
    },

    /// A card was played from hand.
    CardPlayed {
        player: PlayerId,
        card_id: CardId,
        hand_index: usize,
        target_slot: Slot,
        essence_cost: u8,
    },

    /// A card was discarded (overdraw).
    CardDiscarded {
        player: PlayerId,
        card_id: CardId,
        reason: DiscardReason,
    },

    // =========================================================================
    // CREATURES
    // =========================================================================

    /// A creature was spawned onto the battlefield.
    CreatureSpawned {
        player: PlayerId,
        slot: Slot,
        card_id: CardId,
        instance_id: CreatureInstanceId,
        attack: i8,
        health: i8,
        keywords: Keywords,
    },

    /// A creature took damage.
    CreatureDamaged {
        player: PlayerId,
        slot: Slot,
        damage: u8,
        new_health: i8,
        source: DamageSource,
    },

    /// A creature was healed.
    CreatureHealed {
        player: PlayerId,
        slot: Slot,
        amount: u8,
        new_health: i8,
    },

    /// A creature's stats were modified (buff/debuff).
    CreatureStatsChanged {
        player: PlayerId,
        slot: Slot,
        old_attack: i8,
        new_attack: i8,
        old_health: i8,
        new_health: i8,
    },

    /// A creature's keywords were modified.
    CreatureKeywordsChanged {
        player: PlayerId,
        slot: Slot,
        old_keywords: Keywords,
        new_keywords: Keywords,
    },

    /// A creature died and was removed from the battlefield.
    CreatureDied {
        player: PlayerId,
        slot: Slot,
        card_id: CardId,
        instance_id: CreatureInstanceId,
        cause: DeathCause,
    },

    /// A creature was returned to hand (bounce effect).
    CreatureBounced {
        player: PlayerId,
        slot: Slot,
        card_id: CardId,
    },

    // =========================================================================
    // COMBAT
    // =========================================================================

    /// Combat was initiated between two creatures.
    CombatStarted {
        attacker_player: PlayerId,
        attacker_slot: Slot,
        defender_player: PlayerId,
        defender_slot: Slot,
    },

    /// Combat was resolved (damage dealt).
    CombatResolved {
        attacker_damage_dealt: u8,
        defender_damage_dealt: u8,
        attacker_died: bool,
        defender_died: bool,
    },

    // =========================================================================
    // SUPPORTS
    // =========================================================================

    /// A support card was placed on the battlefield.
    SupportPlaced {
        player: PlayerId,
        slot: Slot,
        card_id: CardId,
        durability: u8,
    },

    /// A support's durability decreased.
    SupportDurabilityChanged {
        player: PlayerId,
        slot: Slot,
        old_durability: u8,
        new_durability: u8,
    },

    /// A support was removed from the battlefield.
    SupportRemoved {
        player: PlayerId,
        slot: Slot,
        card_id: CardId,
        reason: SupportRemovalReason,
    },

    // =========================================================================
    // PLAYER RESOURCES
    // =========================================================================

    /// A player's life total changed.
    LifeChanged {
        player: PlayerId,
        old_life: i16,
        new_life: i16,
        source: LifeChangeSource,
    },

    /// A player's essence changed.
    EssenceChanged {
        player: PlayerId,
        old_essence: u8,
        new_essence: u8,
        max_essence: u8,
    },

    /// A player's action points changed.
    ActionPointsChanged {
        player: PlayerId,
        old_ap: u8,
        new_ap: u8,
    },

    // =========================================================================
    // EFFECTS & ABILITIES
    // =========================================================================

    /// A triggered ability was activated.
    AbilityTriggered {
        source_player: PlayerId,
        source_slot: Slot,
        card_id: CardId,
        trigger_name: String,
    },

    /// A spell effect was applied.
    SpellEffectApplied {
        card_id: CardId,
        effect_description: String,
    },
}

/// Reason a card was discarded.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiscardReason {
    /// Hand was full when drawing (overdraw).
    Overdraw,
    /// Discarded by an effect.
    Effect,
}

/// Source of damage to a creature.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DamageSource {
    /// Damage from combat.
    Combat,
    /// Damage from a spell.
    Spell,
    /// Damage from an ability.
    Ability,
    /// Damage from a support effect.
    Support,
}

/// Cause of a creature's death.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeathCause {
    /// Died in combat.
    Combat,
    /// Killed by a spell.
    Spell,
    /// Killed by an ability.
    Ability,
    /// Destroyed by Ephemeral at end of turn.
    Ephemeral,
    /// Destroyed by Destroy effect.
    Destroyed,
}

/// Reason a support was removed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SupportRemovalReason {
    /// Durability reached zero.
    DurabilityDepleted,
    /// Destroyed by an effect.
    Destroyed,
}

/// Source of life change for a player.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LifeChangeSource {
    /// Damage from combat (creature attacking face).
    Combat,
    /// Damage from a spell.
    Spell,
    /// Healed by an effect.
    Healing,
    /// Lifesteal from combat.
    Lifesteal,
}

impl GameEvent {
    /// Get the player primarily affected by this event (if any).
    pub fn affected_player(&self) -> Option<PlayerId> {
        match self {
            GameEvent::TurnStarted { player, .. } => Some(*player),
            GameEvent::TurnEnded { player, .. } => Some(*player),
            GameEvent::ActionTaken { player, .. } => Some(*player),
            GameEvent::CardDrawn { player, .. } => Some(*player),
            GameEvent::CardPlayed { player, .. } => Some(*player),
            GameEvent::CardDiscarded { player, .. } => Some(*player),
            GameEvent::CreatureSpawned { player, .. } => Some(*player),
            GameEvent::CreatureDamaged { player, .. } => Some(*player),
            GameEvent::CreatureHealed { player, .. } => Some(*player),
            GameEvent::CreatureStatsChanged { player, .. } => Some(*player),
            GameEvent::CreatureKeywordsChanged { player, .. } => Some(*player),
            GameEvent::CreatureDied { player, .. } => Some(*player),
            GameEvent::CreatureBounced { player, .. } => Some(*player),
            GameEvent::SupportPlaced { player, .. } => Some(*player),
            GameEvent::SupportDurabilityChanged { player, .. } => Some(*player),
            GameEvent::SupportRemoved { player, .. } => Some(*player),
            GameEvent::LifeChanged { player, .. } => Some(*player),
            GameEvent::EssenceChanged { player, .. } => Some(*player),
            GameEvent::ActionPointsChanged { player, .. } => Some(*player),
            GameEvent::AbilityTriggered { source_player, .. } => Some(*source_player),
            GameEvent::CombatStarted { attacker_player, .. } => Some(*attacker_player),
            GameEvent::CombatResolved { .. } => None,
            GameEvent::SpellEffectApplied { .. } => None,
            GameEvent::GameStarted { .. } => None,
            GameEvent::GameEnded { .. } => None,
        }
    }

    /// Check if this is a terminal event (game over).
    pub fn is_terminal(&self) -> bool {
        matches!(self, GameEvent::GameEnded { .. })
    }
}
