//! Death processing and game over checking.
//!
//! Handles creature death effects including:
//! - Volatile keyword damage
//! - OnDeath triggers
//! - OnAllyDeath triggers
//! - Commander death triggers
//! - Creature removal

use crate::core::cards::CardDatabase;
use crate::core::effects::{EffectSource, Trigger};
use crate::core::engine::{
    collect_commander_ally_death_effects, collect_commander_any_death_effects,
    collect_commander_enemy_death_effects, EffectQueue,
};
use crate::core::state::{GamePhase, GameResult, GameState, WinReason};
use crate::core::types::{PlayerId, Slot};

use super::triggers::trigger_ally_death_for_allies;

/// Process a creature's death, triggering OnDeath and OnAllyDeath effects.
///
/// This handles:
/// 1. Volatile keyword damage to enemy creatures
/// 2. OnDeath trigger for the dying creature
/// 3. OnAllyDeath triggers for friendly creatures
/// 4. Commander death triggers (ally/enemy/any)
/// 5. Removing the creature from the board
/// 6. Recursive death processing for Volatile kills
pub fn process_creature_death(
    state: &mut GameState,
    card_db: &CardDatabase,
    effect_queue: &mut EffectQueue,
    player: PlayerId,
    slot: Slot,
) {
    // Get creature info before removal
    let (card_id, is_silenced, has_volatile) = match state.players[player.index()].get_creature(slot) {
        Some(c) => (c.card_id, c.status.is_silenced(), c.keywords.has_volatile()),
        None => return,
    };

    // VOLATILE: Deal 2 damage to all enemy creatures on death (if not silenced)
    let enemies_killed = process_volatile(state, player, has_volatile, is_silenced);

    // Trigger OnDeath effects (if not silenced)
    if !is_silenced {
        trigger_on_death(state, card_db, effect_queue, player, slot, card_id);
    }

    // Trigger OnAllyDeath for other friendly creatures
    trigger_ally_death_for_allies(state, card_db, effect_queue, player, slot);

    // Trigger commander death effects
    trigger_commander_death_effects(state, card_db, effect_queue, player);

    // Remove the dead creature from the board
    state.players[player.index()].creatures.retain(|c| c.slot != slot);

    // Process deaths of enemy creatures killed by Volatile (can chain!)
    let enemy_player = player.opponent();
    for enemy_slot in enemies_killed {
        process_creature_death(state, card_db, effect_queue, enemy_player, enemy_slot);
    }
}

/// Process Volatile keyword damage.
///
/// Returns a list of enemy slots that were killed by Volatile damage.
fn process_volatile(
    state: &mut GameState,
    player: PlayerId,
    has_volatile: bool,
    is_silenced: bool,
) -> Vec<Slot> {
    const VOLATILE_DAMAGE: i8 = 2;
    let enemy_player = player.opponent();
    let mut enemies_killed: Vec<Slot> = Vec::new();

    if has_volatile && !is_silenced {
        // Collect enemy creature slots first to avoid borrow issues
        let enemy_slots: Vec<Slot> = state.players[enemy_player.index()]
            .creatures
            .iter()
            .map(|c| c.slot)
            .collect();

        // Deal 2 damage to each enemy creature (cap at 0 to prevent negative health)
        for enemy_slot in enemy_slots {
            if let Some(enemy) = state.players[enemy_player.index()].get_creature_mut(enemy_slot) {
                enemy.current_health = (enemy.current_health - VOLATILE_DAMAGE).max(0);
                if enemy.current_health == 0 {
                    enemies_killed.push(enemy_slot);
                }
            }
        }
    }

    enemies_killed
}

/// Trigger OnDeath effects for a creature.
fn trigger_on_death(
    _state: &GameState,
    card_db: &CardDatabase,
    effect_queue: &mut EffectQueue,
    player: PlayerId,
    slot: Slot,
    card_id: crate::types::CardId,
) {
    if let Some(card_def) = card_db.get(card_id) {
        if let Some(abilities) = card_def.creature_abilities() {
            for ability in abilities {
                if ability.trigger == Trigger::OnDeath {
                    let source = EffectSource::Creature { owner: player, slot };
                    for effect_def in &ability.effects {
                        if let Some(effect) = crate::engine::effect_def_to_triggered_effect(
                            effect_def,
                            player,
                            slot,
                            ability,
                        ) {
                            effect_queue.push(effect, source);
                        }
                    }
                }
            }
        }
    }
}

/// Trigger all commander death-related effects.
fn trigger_commander_death_effects(
    state: &GameState,
    card_db: &CardDatabase,
    effect_queue: &mut EffectQueue,
    player: PlayerId,
) {
    // Trigger OnAllyDeath commander trigger (e.g., Plague Sovereign)
    for (effect, source) in collect_commander_ally_death_effects(state, player, card_db) {
        effect_queue.push(effect, source);
    }

    // Trigger OnEnemyDeath commander trigger
    // When a creature dies, the opponent's commander may have OnEnemyDeath trigger
    for (effect, source) in collect_commander_enemy_death_effects(state, player, card_db) {
        effect_queue.push(effect, source);
    }

    // Trigger OnAnyDeath commander triggers for both players
    for (effect, source) in collect_commander_any_death_effects(state, player, card_db) {
        effect_queue.push(effect, source);
    }
    for (effect, source) in collect_commander_any_death_effects(state, player.opponent(), card_db) {
        effect_queue.push(effect, source);
    }
}

/// Check if the game is over due to a player reaching 0 life.
pub fn check_game_over(state: &mut GameState) {
    let p1_dead = state.players[0].life <= 0;
    let p2_dead = state.players[1].life <= 0;

    if p1_dead && p2_dead {
        // Both dead simultaneously = draw
        state.result = Some(GameResult::Draw);
        state.phase = GamePhase::Ended;
    } else if p1_dead {
        state.result = Some(GameResult::Win {
            winner: PlayerId::PLAYER_TWO,
            reason: WinReason::LifeReachedZero,
        });
        state.phase = GamePhase::Ended;
    } else if p2_dead {
        state.result = Some(GameResult::Win {
            winner: PlayerId::PLAYER_ONE,
            reason: WinReason::LifeReachedZero,
        });
        state.phase = GamePhase::Ended;
    }
}
