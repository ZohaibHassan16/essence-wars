//! State diffing utilities for event detection.
//!
//! This module provides utilities to compare game states and generate
//! events representing the changes between them.

use crate::client_api::events::{
    DamageSource, DeathCause, GameEvent, LifeChangeSource, SupportRemovalReason,
};
use crate::core::state::{Creature, GameState, PlayerState, Support};
use crate::core::types::{PlayerId, Slot};

/// A snapshot of game state for diffing.
///
/// This is a lightweight copy of the data we need to detect changes.
#[derive(Clone, Debug)]
pub struct StateSnapshot {
    pub turn: u16,
    pub active_player: PlayerId,
    pub players: [PlayerSnapshot; 2],
    pub rng_state: u64,
}

/// Per-player snapshot for diffing.
#[derive(Clone, Debug)]
pub struct PlayerSnapshot {
    pub life: i16,
    pub max_essence: u8,
    pub current_essence: u8,
    pub action_points: u8,
    pub hand_size: usize,
    pub deck_size: usize,
    pub creatures: Vec<CreatureSnapshot>,
    pub supports: Vec<SupportSnapshot>,
}

/// Creature snapshot for diffing.
#[derive(Clone, Debug)]
pub struct CreatureSnapshot {
    pub slot: Slot,
    pub instance_id: u32,
    pub card_id: u16,
    pub attack: i8,
    pub health: i8,
    pub max_health: i8,
    pub keywords: u16,
    pub exhausted: bool,
}

/// Support snapshot for diffing.
#[derive(Clone, Debug)]
pub struct SupportSnapshot {
    pub slot: Slot,
    pub card_id: u16,
    pub durability: u8,
}

impl StateSnapshot {
    /// Create a snapshot from the current game state.
    pub fn from_state(state: &GameState) -> Self {
        Self {
            turn: state.current_turn,
            active_player: state.active_player,
            players: [
                PlayerSnapshot::from_player(&state.players[0]),
                PlayerSnapshot::from_player(&state.players[1]),
            ],
            rng_state: state.rng_state,
        }
    }

    /// Get player snapshot by PlayerId.
    pub fn player(&self, player: PlayerId) -> &PlayerSnapshot {
        &self.players[player.index()]
    }
}

impl PlayerSnapshot {
    /// Create a snapshot from a PlayerState.
    pub fn from_player(state: &PlayerState) -> Self {
        Self {
            life: state.life,
            max_essence: state.max_essence,
            current_essence: state.current_essence,
            action_points: state.action_points,
            hand_size: state.hand.len(),
            deck_size: state.deck.len(),
            creatures: state
                .creatures
                .iter()
                .map(CreatureSnapshot::from_creature)
                .collect(),
            supports: state
                .supports
                .iter()
                .map(SupportSnapshot::from_support)
                .collect(),
        }
    }

    /// Find a creature by slot.
    pub fn get_creature(&self, slot: Slot) -> Option<&CreatureSnapshot> {
        self.creatures.iter().find(|c| c.slot == slot)
    }

    /// Find a support by slot.
    pub fn get_support(&self, slot: Slot) -> Option<&SupportSnapshot> {
        self.supports.iter().find(|s| s.slot == slot)
    }
}

impl CreatureSnapshot {
    /// Create a snapshot from a Creature.
    pub fn from_creature(creature: &Creature) -> Self {
        Self {
            slot: creature.slot,
            instance_id: creature.instance_id.0,
            card_id: creature.card_id.0,
            attack: creature.attack,
            health: creature.current_health,
            max_health: creature.max_health,
            keywords: creature.keywords.0,
            exhausted: creature.status.is_exhausted(),
        }
    }
}

impl SupportSnapshot {
    /// Create a snapshot from a Support.
    pub fn from_support(support: &Support) -> Self {
        Self {
            slot: support.slot,
            card_id: support.card_id.0,
            durability: support.current_durability,
        }
    }
}

/// Compare two state snapshots and generate events for all changes.
///
/// This is the main diffing function that detects all changes between
/// a before and after state snapshot.
pub fn diff_states(before: &StateSnapshot, after: &StateSnapshot) -> Vec<GameEvent> {
    let mut events = Vec::new();

    // Check for turn changes
    if after.turn > before.turn {
        events.push(GameEvent::TurnStarted {
            player: after.active_player,
            turn_number: after.turn,
            max_essence: after.player(after.active_player).max_essence,
            action_points: after.player(after.active_player).action_points,
        });
    }

    // Check player state changes for both players
    for player_id in [PlayerId::PLAYER_ONE, PlayerId::PLAYER_TWO] {
        let before_player = before.player(player_id);
        let after_player = after.player(player_id);

        // Life changes
        if after_player.life != before_player.life {
            let source = if after_player.life < before_player.life {
                LifeChangeSource::Combat // Default to combat, caller can override
            } else {
                LifeChangeSource::Healing
            };
            events.push(GameEvent::LifeChanged {
                player: player_id,
                old_life: before_player.life,
                new_life: after_player.life,
                source,
            });
        }

        // Essence changes
        if after_player.current_essence != before_player.current_essence
            || after_player.max_essence != before_player.max_essence
        {
            events.push(GameEvent::EssenceChanged {
                player: player_id,
                old_essence: before_player.current_essence,
                new_essence: after_player.current_essence,
                max_essence: after_player.max_essence,
            });
        }

        // Action points changes
        if after_player.action_points != before_player.action_points {
            events.push(GameEvent::ActionPointsChanged {
                player: player_id,
                old_ap: before_player.action_points,
                new_ap: after_player.action_points,
            });
        }

        // Creature changes
        diff_creatures(player_id, before_player, after_player, &mut events);

        // Support changes
        diff_supports(player_id, before_player, after_player, &mut events);
    }

    events
}

/// Diff creatures between two player snapshots.
fn diff_creatures(
    player: PlayerId,
    before: &PlayerSnapshot,
    after: &PlayerSnapshot,
    events: &mut Vec<GameEvent>,
) {
    use crate::core::keywords::Keywords;
    use crate::core::types::{CardId, CreatureInstanceId};

    // Check for new creatures (spawned)
    for after_creature in &after.creatures {
        if before.get_creature(after_creature.slot).is_none() {
            events.push(GameEvent::CreatureSpawned {
                player,
                slot: after_creature.slot,
                card_id: CardId(after_creature.card_id),
                instance_id: CreatureInstanceId(after_creature.instance_id),
                attack: after_creature.attack,
                health: after_creature.health,
                keywords: Keywords(after_creature.keywords),
            });
        }
    }

    // Check for removed creatures (died) and stat changes
    for before_creature in &before.creatures {
        match after.get_creature(before_creature.slot) {
            None => {
                // Creature died
                events.push(GameEvent::CreatureDied {
                    player,
                    slot: before_creature.slot,
                    card_id: CardId(before_creature.card_id),
                    instance_id: CreatureInstanceId(before_creature.instance_id),
                    cause: DeathCause::Combat, // Default, caller can override
                });
            }
            Some(after_creature) => {
                // Check for stat changes
                if before_creature.attack != after_creature.attack
                    || before_creature.health != after_creature.health
                    || before_creature.max_health != after_creature.max_health
                {
                    // Determine if damage or healing
                    if after_creature.health < before_creature.health {
                        let damage = (before_creature.health - after_creature.health) as u8;
                        events.push(GameEvent::CreatureDamaged {
                            player,
                            slot: before_creature.slot,
                            damage,
                            new_health: after_creature.health,
                            source: DamageSource::Combat,
                        });
                    } else if after_creature.health > before_creature.health {
                        let amount = (after_creature.health - before_creature.health) as u8;
                        events.push(GameEvent::CreatureHealed {
                            player,
                            slot: before_creature.slot,
                            amount,
                            new_health: after_creature.health,
                        });
                    }

                    // Attack/stats changed beyond health
                    if before_creature.attack != after_creature.attack {
                        events.push(GameEvent::CreatureStatsChanged {
                            player,
                            slot: before_creature.slot,
                            old_attack: before_creature.attack,
                            new_attack: after_creature.attack,
                            old_health: before_creature.health,
                            new_health: after_creature.health,
                        });
                    }
                }

                // Check for keyword changes
                if before_creature.keywords != after_creature.keywords {
                    events.push(GameEvent::CreatureKeywordsChanged {
                        player,
                        slot: before_creature.slot,
                        old_keywords: Keywords(before_creature.keywords),
                        new_keywords: Keywords(after_creature.keywords),
                    });
                }
            }
        }
    }
}

/// Diff supports between two player snapshots.
fn diff_supports(
    player: PlayerId,
    before: &PlayerSnapshot,
    after: &PlayerSnapshot,
    events: &mut Vec<GameEvent>,
) {
    use crate::core::types::CardId;

    // Check for new supports (placed)
    for after_support in &after.supports {
        if before.get_support(after_support.slot).is_none() {
            events.push(GameEvent::SupportPlaced {
                player,
                slot: after_support.slot,
                card_id: CardId(after_support.card_id),
                durability: after_support.durability,
            });
        }
    }

    // Check for removed supports and durability changes
    for before_support in &before.supports {
        match after.get_support(before_support.slot) {
            None => {
                // Support removed
                events.push(GameEvent::SupportRemoved {
                    player,
                    slot: before_support.slot,
                    card_id: CardId(before_support.card_id),
                    reason: SupportRemovalReason::DurabilityDepleted,
                });
            }
            Some(after_support) => {
                // Check for durability changes
                if before_support.durability != after_support.durability {
                    events.push(GameEvent::SupportDurabilityChanged {
                        player,
                        slot: before_support.slot,
                        old_durability: before_support.durability,
                        new_durability: after_support.durability,
                    });
                }
            }
        }
    }
}

/// Check if a card was drawn by comparing hand and deck sizes.
pub fn detect_card_drawn(before: &PlayerSnapshot, after: &PlayerSnapshot) -> bool {
    after.hand_size > before.hand_size && after.deck_size < before.deck_size
}

/// Check if a card was discarded (overdraw).
pub fn detect_overdraw(before: &PlayerSnapshot, after: &PlayerSnapshot) -> bool {
    after.deck_size < before.deck_size && after.hand_size == before.hand_size
}
