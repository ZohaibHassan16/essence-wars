//! Combat resolution module.
//!
//! This module handles all combat resolution including keyword interactions.
//! Combat flow follows the rules specified in DESIGN.md:
//!
//! 1. Attacker declares attack on a slot
//! 2. Check if slot has defender creature or is empty
//! 3. If empty slot: deal damage directly to enemy player (face damage)
//! 4. If defender present: resolve creature combat with keyword interactions
//!
//! Keywords are resolved in this order:
//! 1. QUICK - Attacker with Quick deals damage first
//! 2. SHIELD - First damage instance is absorbed
//! 3. RANGED - Attacker takes no counter-attack damage
//! 4. PIERCING - Excess damage dealt to enemy player
//! 5. LETHAL - Any non-zero damage kills the target
//! 6. LIFESTEAL - Attacker's controller heals for damage dealt

use crate::core::cards::CardDatabase;
use crate::core::effects::{EffectSource, Trigger};
use crate::core::engine::EffectQueue;
use crate::core::keywords::Keywords;
use crate::core::state::GameState;
use crate::core::types::{PlayerId, Slot};

/// Result of combat resolution
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CombatResult {
    /// Damage the attacker dealt to the defender (or face)
    pub attacker_damage_dealt: u8,
    /// Damage the defender dealt to the attacker (counter-attack)
    pub defender_damage_dealt: u8,
    /// Whether the attacker died in combat
    pub attacker_died: bool,
    /// Whether the defender died in combat
    pub defender_died: bool,
    /// Face damage dealt to the defending player (from face attack or piercing overflow)
    pub face_damage: u8,
    /// Amount the attacker's controller healed from Lifesteal
    pub attacker_healed: u8,
}

/// Resolve combat between attacker and target slot.
///
/// This is the main entry point for combat resolution. It handles:
/// - Face damage when attacking an empty slot
/// - Creature vs creature combat with all keyword interactions
///
/// # Arguments
/// * `state` - The current game state (will be mutated)
/// * `card_db` - Card database for looking up card definitions
/// * `effect_queue` - Effect queue for triggering death effects
/// * `attacker_player` - The player who is attacking
/// * `attacker_slot` - The slot of the attacking creature
/// * `defender_slot` - The target slot being attacked
///
/// # Returns
/// A `CombatResult` containing details about what happened in combat
pub fn resolve_combat(
    state: &mut GameState,
    card_db: &CardDatabase,
    effect_queue: &mut EffectQueue,
    attacker_player: PlayerId,
    attacker_slot: Slot,
    defender_slot: Slot,
) -> CombatResult {
    let defender_player = attacker_player.opponent();

    // Get attacker creature - must exist
    let attacker = state.players[attacker_player.index()]
        .get_creature(attacker_slot)
        .expect("Attacker must exist");

    let attacker_attack = attacker.attack.max(0) as u8;
    let attacker_keywords = attacker.keywords;

    // Check if defender slot is empty (face damage)
    let defender_exists = state.players[defender_player.index()]
        .get_creature(defender_slot)
        .is_some();

    if !defender_exists {
        // Face damage - attack goes directly to enemy player
        return resolve_face_attack(
            state,
            card_db,
            effect_queue,
            attacker_player,
            attacker_slot,
            defender_player,
            attacker_attack,
            attacker_keywords,
        );
    }

    // Creature vs creature combat
    resolve_creature_combat(
        state,
        card_db,
        effect_queue,
        attacker_player,
        attacker_slot,
        defender_player,
        defender_slot,
    )
}

/// Resolve a face attack (attacking an empty slot).
fn resolve_face_attack(
    state: &mut GameState,
    card_db: &CardDatabase,
    effect_queue: &mut EffectQueue,
    attacker_player: PlayerId,
    attacker_slot: Slot,
    defender_player: PlayerId,
    attacker_attack: u8,
    attacker_keywords: Keywords,
) -> CombatResult {
    let damage = attacker_attack;

    // Deal face damage
    state.players[defender_player.index()].life =
        state.players[defender_player.index()].life.saturating_sub(damage as i16);

    // Track total damage dealt
    state.players[attacker_player.index()].total_damage_dealt += damage as u16;

    // Apply Lifesteal
    let healed = if attacker_keywords.has_lifesteal() && damage > 0 {
        // Heal attacker's controller, capped at 30 (max life per DESIGN.md)
        let current_life = state.players[attacker_player.index()].life;
        let new_life = (current_life + damage as i16).min(30);
        let actual_heal = (new_life - current_life) as u8;
        state.players[attacker_player.index()].life = new_life;
        actual_heal
    } else {
        0
    };

    // Mark attacker as having attacked
    mark_attacked(state, attacker_player, attacker_slot);

    // Trigger OnAttack effect (if creature has one)
    trigger_on_attack(state, card_db, effect_queue, attacker_player, attacker_slot);

    // Check for game over
    check_game_over(state);

    CombatResult {
        attacker_damage_dealt: damage,
        defender_damage_dealt: 0,
        attacker_died: false,
        defender_died: false,
        face_damage: damage,
        attacker_healed: healed,
    }
}

/// Resolve creature vs creature combat with all keyword interactions.
fn resolve_creature_combat(
    state: &mut GameState,
    card_db: &CardDatabase,
    effect_queue: &mut EffectQueue,
    attacker_player: PlayerId,
    attacker_slot: Slot,
    defender_player: PlayerId,
    defender_slot: Slot,
) -> CombatResult {
    // Gather all creature stats and keywords before combat
    let (
        attacker_attack,
        attacker_health,
        attacker_keywords,
    ) = {
        let attacker = state.players[attacker_player.index()]
            .get_creature(attacker_slot)
            .expect("Attacker must exist");
        (
            attacker.attack.max(0) as u8,
            attacker.current_health,
            attacker.keywords,
        )
    };

    let (
        defender_attack,
        defender_health,
        defender_keywords,
    ) = {
        let defender = state.players[defender_player.index()]
            .get_creature(defender_slot)
            .expect("Defender must exist");
        (
            defender.attack.max(0) as u8,
            defender.current_health,
            defender.keywords,
        )
    };

    // Extract keyword flags
    let attacker_has_quick = attacker_keywords.has_quick();
    let attacker_has_shield = attacker_keywords.has_shield();
    let attacker_has_ranged = attacker_keywords.has_ranged();
    let attacker_has_piercing = attacker_keywords.has_piercing();
    let attacker_has_lethal = attacker_keywords.has_lethal();
    let attacker_has_lifesteal = attacker_keywords.has_lifesteal();

    let defender_has_quick = defender_keywords.has_quick();
    let defender_has_shield = defender_keywords.has_shield();
    let defender_has_lethal = defender_keywords.has_lethal();

    // Trigger OnAttack effect before combat damage
    trigger_on_attack(state, card_db, effect_queue, attacker_player, attacker_slot);

    // Determine combat order based on Quick keyword
    // - If attacker has Quick and defender doesn't: attacker strikes first
    // - If defender has Quick and attacker doesn't: defender strikes first
    // - If both or neither have Quick: simultaneous damage

    let mut result = CombatResult::default();
    let mut attacker_died = false;
    let mut defender_died = false;
    let mut attacker_took_damage = false;
    let mut defender_took_damage = false;
    let mut actual_damage_to_defender: u8 = 0;
    let mut actual_damage_to_attacker: u8 = 0;

    if attacker_has_quick && !defender_has_quick {
        // Attacker strikes first
        let (damage_dealt, _blocked_by_shield, target_died) = apply_combat_damage(
            state,
            defender_player,
            defender_slot,
            attacker_attack,
            attacker_has_lethal,
            defender_has_shield,
            defender_health,
        );

        actual_damage_to_defender = damage_dealt;
        defender_took_damage = damage_dealt > 0;
        defender_died = target_died;
        result.attacker_damage_dealt = damage_dealt;

        // If defender survived, they counter-attack (unless attacker has Ranged)
        if !defender_died && !attacker_has_ranged {
            let (damage_dealt, _blocked_by_shield, target_died) = apply_combat_damage(
                state,
                attacker_player,
                attacker_slot,
                defender_attack,
                defender_has_lethal,
                attacker_has_shield,
                attacker_health,
            );

            actual_damage_to_attacker = damage_dealt;
            attacker_took_damage = damage_dealt > 0;
            attacker_died = target_died;
            result.defender_damage_dealt = damage_dealt;
        }
    } else if defender_has_quick && !attacker_has_quick {
        // Defender strikes first (counter-attack happens first)
        // But only if attacker doesn't have Ranged
        if !attacker_has_ranged {
            let (damage_dealt, _blocked_by_shield, target_died) = apply_combat_damage(
                state,
                attacker_player,
                attacker_slot,
                defender_attack,
                defender_has_lethal,
                attacker_has_shield,
                attacker_health,
            );

            actual_damage_to_attacker = damage_dealt;
            attacker_took_damage = damage_dealt > 0;
            attacker_died = target_died;
            result.defender_damage_dealt = damage_dealt;
        }

        // If attacker survived, they deal damage
        if !attacker_died {
            let (damage_dealt, _blocked_by_shield, target_died) = apply_combat_damage(
                state,
                defender_player,
                defender_slot,
                attacker_attack,
                attacker_has_lethal,
                defender_has_shield,
                defender_health,
            );

            actual_damage_to_defender = damage_dealt;
            defender_took_damage = damage_dealt > 0;
            defender_died = target_died;
            result.attacker_damage_dealt = damage_dealt;
        }
    } else {
        // Simultaneous damage (both have Quick or neither has Quick)

        // Apply attacker's damage to defender
        let (atk_damage_dealt, _atk_blocked, def_would_die) = apply_combat_damage(
            state,
            defender_player,
            defender_slot,
            attacker_attack,
            attacker_has_lethal,
            defender_has_shield,
            defender_health,
        );

        actual_damage_to_defender = atk_damage_dealt;
        defender_took_damage = atk_damage_dealt > 0;
        defender_died = def_would_die;
        result.attacker_damage_dealt = atk_damage_dealt;

        // Apply defender's counter-attack damage to attacker (unless Ranged)
        if !attacker_has_ranged {
            let (def_damage_dealt, _def_blocked, atk_would_die) = apply_combat_damage(
                state,
                attacker_player,
                attacker_slot,
                defender_attack,
                defender_has_lethal,
                attacker_has_shield,
                attacker_health,
            );

            actual_damage_to_attacker = def_damage_dealt;
            attacker_took_damage = def_damage_dealt > 0;
            attacker_died = atk_would_die;
            result.defender_damage_dealt = def_damage_dealt;
        }
    }

    result.attacker_died = attacker_died;
    result.defender_died = defender_died;

    // Apply Piercing: excess damage to face when defender dies
    if attacker_has_piercing && defender_died && actual_damage_to_defender > 0 {
        // Calculate excess damage: attack power minus defender's remaining health before death
        // The defender died, so we need to figure out how much "overkill" damage there was
        let defender_health_before = defender_health as u8;
        if attacker_attack > defender_health_before {
            let excess = attacker_attack - defender_health_before;
            state.players[defender_player.index()].life =
                state.players[defender_player.index()].life.saturating_sub(excess as i16);
            result.face_damage = excess;

            // Track piercing damage dealt
            state.players[attacker_player.index()].total_damage_dealt += excess as u16;
        }
    }

    // Apply Lifesteal: heal for damage actually dealt to creatures
    if attacker_has_lifesteal && actual_damage_to_defender > 0 {
        // Heal for damage dealt to the creature (not piercing overflow)
        // Cap the heal at the defender's health before combat
        let heal_amount = actual_damage_to_defender.min(defender_health.max(0) as u8);
        if heal_amount > 0 {
            let current_life = state.players[attacker_player.index()].life;
            let new_life = (current_life + heal_amount as i16).min(30);
            let actual_heal = (new_life - current_life) as u8;
            state.players[attacker_player.index()].life = new_life;
            result.attacker_healed = actual_heal;
        }
    }

    // Track damage dealt to creatures
    state.players[attacker_player.index()].total_damage_dealt += actual_damage_to_defender as u16;

    // Mark attacker as having attacked
    mark_attacked(state, attacker_player, attacker_slot);

    // Trigger OnDealDamage effects
    if actual_damage_to_defender > 0 {
        trigger_on_deal_damage(state, card_db, effect_queue, attacker_player, attacker_slot);
    }
    if actual_damage_to_attacker > 0 {
        trigger_on_deal_damage(state, card_db, effect_queue, defender_player, defender_slot);
    }

    // Trigger OnTakeDamage effects
    if defender_took_damage {
        trigger_on_take_damage(state, card_db, effect_queue, defender_player, defender_slot);
    }
    if attacker_took_damage {
        trigger_on_take_damage(state, card_db, effect_queue, attacker_player, attacker_slot);
    }

    // Process deaths and trigger OnDeath/OnKill effects
    if defender_died {
        trigger_on_kill(state, card_db, effect_queue, attacker_player, attacker_slot);
        process_creature_death(state, card_db, effect_queue, defender_player, defender_slot);
    }
    if attacker_died {
        // Note: defender doesn't get OnKill trigger since they're defending
        process_creature_death(state, card_db, effect_queue, attacker_player, attacker_slot);
    }

    // Check for game over
    check_game_over(state);

    result
}

/// Apply combat damage to a creature, handling Shield and Lethal keywords.
///
/// Returns (damage_dealt, was_blocked_by_shield, creature_died)
fn apply_combat_damage(
    state: &mut GameState,
    target_player: PlayerId,
    target_slot: Slot,
    damage: u8,
    attacker_has_lethal: bool,
    target_has_shield: bool,
    _target_health_before: i8,
) -> (u8, bool, bool) {
    if damage == 0 {
        return (0, false, false);
    }

    let creature = match state.players[target_player.index()].get_creature_mut(target_slot) {
        Some(c) => c,
        None => return (0, false, false),
    };

    if target_has_shield {
        // Shield absorbs the damage completely
        creature.keywords.remove(Keywords::SHIELD);
        // No damage dealt, Lethal doesn't trigger
        return (0, true, false);
    }

    // Apply damage
    creature.current_health -= damage as i8;
    let died = creature.current_health <= 0;

    // Apply Lethal: any non-zero damage kills
    if attacker_has_lethal && damage > 0 && !died {
        creature.current_health = 0;
        return (damage, false, true);
    }

    (damage, false, died)
}

/// Mark a creature as having attacked this turn.
fn mark_attacked(state: &mut GameState, player: PlayerId, slot: Slot) {
    if let Some(creature) = state.players[player.index()].get_creature_mut(slot) {
        creature.status.set_exhausted(true);
    }
}

/// Trigger OnAttack effects for a creature.
fn trigger_on_attack(
    state: &mut GameState,
    card_db: &CardDatabase,
    effect_queue: &mut EffectQueue,
    player: PlayerId,
    slot: Slot,
) {
    let creature = match state.players[player.index()].get_creature(slot) {
        Some(c) => c,
        None => return,
    };

    // Don't trigger if silenced
    if creature.status.is_silenced() {
        return;
    }

    let card_id = creature.card_id;
    let card_def = match card_db.get(card_id) {
        Some(c) => c,
        None => return,
    };

    let abilities = match card_def.creature_abilities() {
        Some(a) => a,
        None => return,
    };

    for ability in abilities {
        if ability.trigger == Trigger::OnAttack {
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

/// Trigger OnDealDamage effects for a creature.
fn trigger_on_deal_damage(
    state: &mut GameState,
    card_db: &CardDatabase,
    effect_queue: &mut EffectQueue,
    player: PlayerId,
    slot: Slot,
) {
    let creature = match state.players[player.index()].get_creature(slot) {
        Some(c) => c,
        None => return,
    };

    if creature.status.is_silenced() {
        return;
    }

    let card_id = creature.card_id;
    let card_def = match card_db.get(card_id) {
        Some(c) => c,
        None => return,
    };

    let abilities = match card_def.creature_abilities() {
        Some(a) => a,
        None => return,
    };

    for ability in abilities {
        if ability.trigger == Trigger::OnDealDamage {
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

/// Trigger OnTakeDamage effects for a creature.
fn trigger_on_take_damage(
    state: &mut GameState,
    card_db: &CardDatabase,
    effect_queue: &mut EffectQueue,
    player: PlayerId,
    slot: Slot,
) {
    let creature = match state.players[player.index()].get_creature(slot) {
        Some(c) => c,
        None => return,
    };

    if creature.status.is_silenced() {
        return;
    }

    let card_id = creature.card_id;
    let card_def = match card_db.get(card_id) {
        Some(c) => c,
        None => return,
    };

    let abilities = match card_def.creature_abilities() {
        Some(a) => a,
        None => return,
    };

    for ability in abilities {
        if ability.trigger == Trigger::OnTakeDamage {
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

/// Trigger OnKill effects for a creature.
fn trigger_on_kill(
    state: &mut GameState,
    card_db: &CardDatabase,
    effect_queue: &mut EffectQueue,
    player: PlayerId,
    slot: Slot,
) {
    let creature = match state.players[player.index()].get_creature(slot) {
        Some(c) => c,
        None => return,
    };

    if creature.status.is_silenced() {
        return;
    }

    let card_id = creature.card_id;
    let card_def = match card_db.get(card_id) {
        Some(c) => c,
        None => return,
    };

    let abilities = match card_def.creature_abilities() {
        Some(a) => a,
        None => return,
    };

    for ability in abilities {
        if ability.trigger == Trigger::OnKill {
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

/// Process a creature's death, triggering OnDeath and OnAllyDeath effects.
fn process_creature_death(
    state: &mut GameState,
    card_db: &CardDatabase,
    effect_queue: &mut EffectQueue,
    player: PlayerId,
    slot: Slot,
) {
    // Get creature info before removal
    let (card_id, is_silenced) = match state.players[player.index()].get_creature(slot) {
        Some(c) => (c.card_id, c.status.is_silenced()),
        None => return,
    };

    // Trigger OnDeath effects (if not silenced)
    if !is_silenced {
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

    // Trigger OnAllyDeath for other friendly creatures
    let ally_creatures: Vec<(Slot, crate::types::CardId, bool)> = state.players[player.index()]
        .creatures
        .iter()
        .filter(|c| c.slot != slot)
        .map(|c| (c.slot, c.card_id, c.status.is_silenced()))
        .collect();

    for (ally_slot, ally_card_id, ally_silenced) in ally_creatures {
        if ally_silenced {
            continue;
        }

        if let Some(card_def) = card_db.get(ally_card_id) {
            if let Some(abilities) = card_def.creature_abilities() {
                for ability in abilities {
                    if ability.trigger == Trigger::OnAllyDeath {
                        let source = EffectSource::Creature { owner: player, slot: ally_slot };
                        for effect_def in &ability.effects {
                            if let Some(effect) = crate::engine::effect_def_to_triggered_effect(
                                effect_def,
                                player,
                                ally_slot,
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

    // Remove the dead creature from the board
    state.players[player.index()].creatures.retain(|c| c.slot != slot);
}

/// Check if the game is over due to a player reaching 0 life.
fn check_game_over(state: &mut GameState) {
    use crate::core::state::{GameResult, WinReason};

    let p1_dead = state.players[0].life <= 0;
    let p2_dead = state.players[1].life <= 0;

    if p1_dead && p2_dead {
        // Both dead simultaneously = draw
        state.result = Some(GameResult::Draw);
    } else if p1_dead {
        state.result = Some(GameResult::Win {
            winner: PlayerId::PLAYER_TWO,
            reason: WinReason::LifeReachedZero,
        });
    } else if p2_dead {
        state.result = Some(GameResult::Win {
            winner: PlayerId::PLAYER_ONE,
            reason: WinReason::LifeReachedZero,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::cards::{CardDatabase, CardDefinition, CardType};
    use crate::core::state::{Creature, CreatureStatus};
    use crate::core::types::{CardId, CreatureInstanceId, Rarity};

    /// Create a minimal test card database
    fn test_card_db() -> CardDatabase {
        let cards = vec![
            // Basic creature (no keywords)
            CardDefinition {
                id: 1,
                name: "Basic Creature".to_string(),
                cost: 2,
                card_type: CardType::Creature {
                    attack: 3,
                    health: 4,
                    keywords: vec![],
                    abilities: vec![],
                },
                rarity: Rarity::Common,
                tags: vec![],
            },
            // Creature with Quick
            CardDefinition {
                id: 2,
                name: "Quick Creature".to_string(),
                cost: 3,
                card_type: CardType::Creature {
                    attack: 2,
                    health: 2,
                    keywords: vec!["Quick".to_string()],
                    abilities: vec![],
                },
                rarity: Rarity::Common,
                tags: vec![],
            },
            // Creature with Shield
            CardDefinition {
                id: 3,
                name: "Shielded Creature".to_string(),
                cost: 3,
                card_type: CardType::Creature {
                    attack: 2,
                    health: 3,
                    keywords: vec!["Shield".to_string()],
                    abilities: vec![],
                },
                rarity: Rarity::Common,
                tags: vec![],
            },
            // Creature with Ranged
            CardDefinition {
                id: 4,
                name: "Ranged Creature".to_string(),
                cost: 3,
                card_type: CardType::Creature {
                    attack: 4,
                    health: 2,
                    keywords: vec!["Ranged".to_string()],
                    abilities: vec![],
                },
                rarity: Rarity::Common,
                tags: vec![],
            },
            // Creature with Piercing
            CardDefinition {
                id: 5,
                name: "Piercing Creature".to_string(),
                cost: 3,
                card_type: CardType::Creature {
                    attack: 5,
                    health: 3,
                    keywords: vec!["Piercing".to_string()],
                    abilities: vec![],
                },
                rarity: Rarity::Common,
                tags: vec![],
            },
            // Creature with Lethal
            CardDefinition {
                id: 6,
                name: "Lethal Creature".to_string(),
                cost: 3,
                card_type: CardType::Creature {
                    attack: 1,
                    health: 1,
                    keywords: vec!["Lethal".to_string()],
                    abilities: vec![],
                },
                rarity: Rarity::Common,
                tags: vec![],
            },
            // Creature with Lifesteal
            CardDefinition {
                id: 7,
                name: "Lifesteal Creature".to_string(),
                cost: 3,
                card_type: CardType::Creature {
                    attack: 4,
                    health: 4,
                    keywords: vec!["Lifesteal".to_string()],
                    abilities: vec![],
                },
                rarity: Rarity::Common,
                tags: vec![],
            },
            // Creature with Quick + Lethal
            CardDefinition {
                id: 8,
                name: "Quick Lethal Creature".to_string(),
                cost: 5,
                card_type: CardType::Creature {
                    attack: 1,
                    health: 1,
                    keywords: vec!["Quick".to_string(), "Lethal".to_string()],
                    abilities: vec![],
                },
                rarity: Rarity::Rare,
                tags: vec![],
            },
        ];
        CardDatabase::new(cards)
    }

    /// Create a test game state with two empty player boards
    fn test_game_state() -> GameState {
        let mut state = GameState::new();
        state.players[0].life = 30;
        state.players[1].life = 30;
        state.current_turn = 2; // Turn 2 so creatures can attack (no summoning sickness)
        state
    }

    /// Helper to create a creature for testing
    fn create_test_creature(
        instance_id: u32,
        card_id: u16,
        owner: PlayerId,
        slot: Slot,
        attack: i8,
        health: i8,
        keywords: Keywords,
    ) -> Creature {
        Creature {
            instance_id: CreatureInstanceId(instance_id),
            card_id: CardId(card_id),
            owner,
            slot,
            attack,
            current_health: health,
            max_health: health,
            base_attack: attack as u8,
            base_health: health as u8,
            keywords,
            status: CreatureStatus::default(),
            turn_played: 1, // Played last turn, so no summoning sickness
        }
    }

    #[test]
    fn test_basic_combat() {
        let card_db = test_card_db();
        let mut state = test_game_state();
        let mut effect_queue = EffectQueue::new();

        // Player 1 has a 3/4 creature in slot 0
        state.players[0].creatures.push(create_test_creature(
            0, 1, PlayerId::PLAYER_ONE, Slot(0), 3, 4, Keywords::none()
        ));

        // Player 2 has a 2/3 creature in slot 0
        state.players[1].creatures.push(create_test_creature(
            1, 1, PlayerId::PLAYER_TWO, Slot(0), 2, 3, Keywords::none()
        ));

        let result = resolve_combat(
            &mut state,
            &card_db,
            &mut effect_queue,
            PlayerId::PLAYER_ONE,
            Slot(0),
            Slot(0),
        );

        // Attacker deals 3 damage, defender deals 2 damage
        assert_eq!(result.attacker_damage_dealt, 3);
        assert_eq!(result.defender_damage_dealt, 2);

        // Defender dies (3/3 -> 3/0), attacker survives (3/4 -> 3/2)
        assert!(!result.attacker_died);
        assert!(result.defender_died);

        // Verify state
        let attacker = state.players[0].get_creature(Slot(0)).unwrap();
        assert_eq!(attacker.current_health, 2);
        assert!(state.players[1].get_creature(Slot(0)).is_none()); // Defender removed
    }

    #[test]
    fn test_face_damage() {
        let card_db = test_card_db();
        let mut state = test_game_state();
        let mut effect_queue = EffectQueue::new();

        // Player 1 has a 3/4 creature in slot 0
        state.players[0].creatures.push(create_test_creature(
            0, 1, PlayerId::PLAYER_ONE, Slot(0), 3, 4, Keywords::none()
        ));

        // No defender in slot 0

        let result = resolve_combat(
            &mut state,
            &card_db,
            &mut effect_queue,
            PlayerId::PLAYER_ONE,
            Slot(0),
            Slot(0),
        );

        // Face damage dealt
        assert_eq!(result.attacker_damage_dealt, 3);
        assert_eq!(result.face_damage, 3);
        assert_eq!(result.defender_damage_dealt, 0);
        assert!(!result.attacker_died);
        assert!(!result.defender_died);

        // Player 2's life reduced
        assert_eq!(state.players[1].life, 27);
    }

    #[test]
    fn test_quick_kills_before_counter() {
        let card_db = test_card_db();
        let mut state = test_game_state();
        let mut effect_queue = EffectQueue::new();

        // Player 1 has a 2/2 Quick creature in slot 0
        state.players[0].creatures.push(create_test_creature(
            0, 2, PlayerId::PLAYER_ONE, Slot(0), 2, 2, Keywords::none().with_quick()
        ));

        // Player 2 has a 5/2 creature in slot 0 (would kill attacker if it could counter)
        state.players[1].creatures.push(create_test_creature(
            1, 1, PlayerId::PLAYER_TWO, Slot(0), 5, 2, Keywords::none()
        ));

        let result = resolve_combat(
            &mut state,
            &card_db,
            &mut effect_queue,
            PlayerId::PLAYER_ONE,
            Slot(0),
            Slot(0),
        );

        // Quick creature kills defender before counter-attack
        assert!(!result.attacker_died);
        assert!(result.defender_died);
        assert_eq!(result.defender_damage_dealt, 0); // No counter-attack

        // Attacker still at full health
        let attacker = state.players[0].get_creature(Slot(0)).unwrap();
        assert_eq!(attacker.current_health, 2);
    }

    #[test]
    fn test_shield_absorbs_damage() {
        let card_db = test_card_db();
        let mut state = test_game_state();
        let mut effect_queue = EffectQueue::new();

        // Player 1 has a 3/4 creature in slot 0
        state.players[0].creatures.push(create_test_creature(
            0, 1, PlayerId::PLAYER_ONE, Slot(0), 3, 4, Keywords::none()
        ));

        // Player 2 has a 2/3 shielded creature in slot 0
        state.players[1].creatures.push(create_test_creature(
            1, 3, PlayerId::PLAYER_TWO, Slot(0), 2, 3, Keywords::none().with_shield()
        ));

        let result = resolve_combat(
            &mut state,
            &card_db,
            &mut effect_queue,
            PlayerId::PLAYER_ONE,
            Slot(0),
            Slot(0),
        );

        // Shield absorbed damage, defender survives
        assert_eq!(result.attacker_damage_dealt, 0); // Blocked by shield
        assert!(!result.defender_died);

        // Defender lost shield but kept health
        let defender = state.players[1].get_creature(Slot(0)).unwrap();
        assert_eq!(defender.current_health, 3);
        assert!(!defender.keywords.has_shield());

        // Attacker took counter-attack damage
        let attacker = state.players[0].get_creature(Slot(0)).unwrap();
        assert_eq!(attacker.current_health, 2); // 4 - 2 = 2
    }

    #[test]
    fn test_ranged_no_counter_attack() {
        let card_db = test_card_db();
        let mut state = test_game_state();
        let mut effect_queue = EffectQueue::new();

        // Player 1 has a 4/2 ranged creature in slot 0
        state.players[0].creatures.push(create_test_creature(
            0, 4, PlayerId::PLAYER_ONE, Slot(0), 4, 2, Keywords::none().with_ranged()
        ));

        // Player 2 has a 3/5 creature in slot 0
        state.players[1].creatures.push(create_test_creature(
            1, 1, PlayerId::PLAYER_TWO, Slot(0), 3, 5, Keywords::none()
        ));

        let result = resolve_combat(
            &mut state,
            &card_db,
            &mut effect_queue,
            PlayerId::PLAYER_ONE,
            Slot(0),
            Slot(0),
        );

        // Ranged creature deals damage but takes none
        assert_eq!(result.attacker_damage_dealt, 4);
        assert_eq!(result.defender_damage_dealt, 0);
        assert!(!result.attacker_died);
        assert!(!result.defender_died);

        // Attacker still at full health
        let attacker = state.players[0].get_creature(Slot(0)).unwrap();
        assert_eq!(attacker.current_health, 2);

        // Defender took damage
        let defender = state.players[1].get_creature(Slot(0)).unwrap();
        assert_eq!(defender.current_health, 1); // 5 - 4 = 1
    }

    #[test]
    fn test_piercing_overflow() {
        let card_db = test_card_db();
        let mut state = test_game_state();
        let mut effect_queue = EffectQueue::new();

        // Player 1 has a 5/3 piercing creature in slot 0
        state.players[0].creatures.push(create_test_creature(
            0, 5, PlayerId::PLAYER_ONE, Slot(0), 5, 3, Keywords::none().with_piercing()
        ));

        // Player 2 has a 1/2 creature in slot 0
        state.players[1].creatures.push(create_test_creature(
            1, 1, PlayerId::PLAYER_TWO, Slot(0), 1, 2, Keywords::none()
        ));

        let result = resolve_combat(
            &mut state,
            &card_db,
            &mut effect_queue,
            PlayerId::PLAYER_ONE,
            Slot(0),
            Slot(0),
        );

        // 5 attack - 2 health = 3 overflow damage
        assert_eq!(result.face_damage, 3);
        assert!(result.defender_died);

        // Player 2's life reduced by overflow
        assert_eq!(state.players[1].life, 27); // 30 - 3 = 27
    }

    #[test]
    fn test_lethal_kills_regardless_of_health() {
        let card_db = test_card_db();
        let mut state = test_game_state();
        let mut effect_queue = EffectQueue::new();

        // Player 1 has a 1/1 lethal creature in slot 0
        state.players[0].creatures.push(create_test_creature(
            0, 6, PlayerId::PLAYER_ONE, Slot(0), 1, 1, Keywords::none().with_lethal()
        ));

        // Player 2 has a 1/10 creature in slot 0 (very high health)
        state.players[1].creatures.push(create_test_creature(
            1, 1, PlayerId::PLAYER_TWO, Slot(0), 1, 10, Keywords::none()
        ));

        let result = resolve_combat(
            &mut state,
            &card_db,
            &mut effect_queue,
            PlayerId::PLAYER_ONE,
            Slot(0),
            Slot(0),
        );

        // Lethal killed the high-health defender
        assert!(result.defender_died);
        // Both die (simultaneous damage)
        assert!(result.attacker_died);
    }

    #[test]
    fn test_lifesteal_heals() {
        let card_db = test_card_db();
        let mut state = test_game_state();
        let mut effect_queue = EffectQueue::new();

        // Set player 1's life to 20
        state.players[0].life = 20;

        // Player 1 has a 4/4 lifesteal creature in slot 0
        state.players[0].creatures.push(create_test_creature(
            0, 7, PlayerId::PLAYER_ONE, Slot(0), 4, 4, Keywords::none().with_lifesteal()
        ));

        // Player 2 has a 2/3 creature in slot 0
        state.players[1].creatures.push(create_test_creature(
            1, 1, PlayerId::PLAYER_TWO, Slot(0), 2, 3, Keywords::none()
        ));

        let result = resolve_combat(
            &mut state,
            &card_db,
            &mut effect_queue,
            PlayerId::PLAYER_ONE,
            Slot(0),
            Slot(0),
        );

        // Lifesteal heals for damage dealt (3, the defender's health)
        assert_eq!(result.attacker_healed, 3);

        // Player 1's life increased
        assert_eq!(state.players[0].life, 23); // 20 + 3 = 23
    }

    #[test]
    fn test_quick_plus_lethal_combo() {
        let card_db = test_card_db();
        let mut state = test_game_state();
        let mut effect_queue = EffectQueue::new();

        // Player 1 has a 1/1 quick+lethal creature in slot 0
        state.players[0].creatures.push(create_test_creature(
            0, 8, PlayerId::PLAYER_ONE, Slot(0), 1, 1,
            Keywords::none().with_quick().with_lethal()
        ));

        // Player 2 has a 10/10 creature in slot 0
        state.players[1].creatures.push(create_test_creature(
            1, 1, PlayerId::PLAYER_TWO, Slot(0), 10, 10, Keywords::none()
        ));

        let result = resolve_combat(
            &mut state,
            &card_db,
            &mut effect_queue,
            PlayerId::PLAYER_ONE,
            Slot(0),
            Slot(0),
        );

        // Quick + Lethal kills the 10/10 before it can counter-attack
        assert!(!result.attacker_died);
        assert!(result.defender_died);
        assert_eq!(result.defender_damage_dealt, 0);

        // Attacker survives at full health
        let attacker = state.players[0].get_creature(Slot(0)).unwrap();
        assert_eq!(attacker.current_health, 1);
    }

    #[test]
    fn test_shield_vs_piercing() {
        let card_db = test_card_db();
        let mut state = test_game_state();
        let mut effect_queue = EffectQueue::new();

        // Player 1 has a 5/3 piercing creature in slot 0
        state.players[0].creatures.push(create_test_creature(
            0, 5, PlayerId::PLAYER_ONE, Slot(0), 5, 3, Keywords::none().with_piercing()
        ));

        // Player 2 has a 2/3 shielded creature in slot 0
        state.players[1].creatures.push(create_test_creature(
            1, 3, PlayerId::PLAYER_TWO, Slot(0), 2, 3, Keywords::none().with_shield()
        ));

        let result = resolve_combat(
            &mut state,
            &card_db,
            &mut effect_queue,
            PlayerId::PLAYER_ONE,
            Slot(0),
            Slot(0),
        );

        // Shield absorbs damage, no piercing overflow
        assert_eq!(result.attacker_damage_dealt, 0);
        assert_eq!(result.face_damage, 0);
        assert!(!result.defender_died);

        // Defender lost shield but took no damage
        let defender = state.players[1].get_creature(Slot(0)).unwrap();
        assert_eq!(defender.current_health, 3);
        assert!(!defender.keywords.has_shield());
    }

    #[test]
    fn test_shield_vs_lethal() {
        let card_db = test_card_db();
        let mut state = test_game_state();
        let mut effect_queue = EffectQueue::new();

        // Player 1 has a 1/1 lethal creature in slot 0
        state.players[0].creatures.push(create_test_creature(
            0, 6, PlayerId::PLAYER_ONE, Slot(0), 1, 1, Keywords::none().with_lethal()
        ));

        // Player 2 has a 2/3 shielded creature in slot 0
        state.players[1].creatures.push(create_test_creature(
            1, 3, PlayerId::PLAYER_TWO, Slot(0), 2, 3, Keywords::none().with_shield()
        ));

        let result = resolve_combat(
            &mut state,
            &card_db,
            &mut effect_queue,
            PlayerId::PLAYER_ONE,
            Slot(0),
            Slot(0),
        );

        // Shield absorbs damage, lethal doesn't trigger
        assert_eq!(result.attacker_damage_dealt, 0);
        assert!(!result.defender_died);

        // Defender lost shield but survived
        let defender = state.players[1].get_creature(Slot(0)).unwrap();
        assert_eq!(defender.current_health, 3);
        assert!(!defender.keywords.has_shield());

        // Attacker died from counter-attack
        assert!(result.attacker_died);
    }

    #[test]
    fn test_both_have_shield() {
        let card_db = test_card_db();
        let mut state = test_game_state();
        let mut effect_queue = EffectQueue::new();

        // Player 1 has a 3/3 shielded creature in slot 0
        state.players[0].creatures.push(create_test_creature(
            0, 3, PlayerId::PLAYER_ONE, Slot(0), 3, 3, Keywords::none().with_shield()
        ));

        // Player 2 has a 3/3 shielded creature in slot 0
        state.players[1].creatures.push(create_test_creature(
            1, 3, PlayerId::PLAYER_TWO, Slot(0), 3, 3, Keywords::none().with_shield()
        ));

        let result = resolve_combat(
            &mut state,
            &card_db,
            &mut effect_queue,
            PlayerId::PLAYER_ONE,
            Slot(0),
            Slot(0),
        );

        // Both shields absorbed damage
        assert_eq!(result.attacker_damage_dealt, 0);
        assert_eq!(result.defender_damage_dealt, 0);
        assert!(!result.attacker_died);
        assert!(!result.defender_died);

        // Both lost shields but survived at full health
        let attacker = state.players[0].get_creature(Slot(0)).unwrap();
        assert_eq!(attacker.current_health, 3);
        assert!(!attacker.keywords.has_shield());

        let defender = state.players[1].get_creature(Slot(0)).unwrap();
        assert_eq!(defender.current_health, 3);
        assert!(!defender.keywords.has_shield());
    }

    #[test]
    fn test_shield_vs_lifesteal() {
        let card_db = test_card_db();
        let mut state = test_game_state();
        let mut effect_queue = EffectQueue::new();

        // Set player 1's life to 20
        state.players[0].life = 20;

        // Player 1 has a 4/4 lifesteal creature in slot 0
        state.players[0].creatures.push(create_test_creature(
            0, 7, PlayerId::PLAYER_ONE, Slot(0), 4, 4, Keywords::none().with_lifesteal()
        ));

        // Player 2 has a 2/3 shielded creature in slot 0
        state.players[1].creatures.push(create_test_creature(
            1, 3, PlayerId::PLAYER_TWO, Slot(0), 2, 3, Keywords::none().with_shield()
        ));

        let result = resolve_combat(
            &mut state,
            &card_db,
            &mut effect_queue,
            PlayerId::PLAYER_ONE,
            Slot(0),
            Slot(0),
        );

        // Shield blocked damage, no lifesteal healing
        assert_eq!(result.attacker_damage_dealt, 0);
        assert_eq!(result.attacker_healed, 0);

        // Player 1's life unchanged (counter-attack damages creature, not player)
        assert_eq!(state.players[0].life, 20);

        // Attacker took counter-attack damage (2 damage from defender)
        let attacker = state.players[0].get_creature(Slot(0)).unwrap();
        assert_eq!(attacker.current_health, 2); // 4 - 2 = 2
    }

    #[test]
    fn test_lifesteal_face_attack() {
        let card_db = test_card_db();
        let mut state = test_game_state();
        let mut effect_queue = EffectQueue::new();

        // Set player 1's life to 20
        state.players[0].life = 20;

        // Player 1 has a 4/4 lifesteal creature in slot 0
        state.players[0].creatures.push(create_test_creature(
            0, 7, PlayerId::PLAYER_ONE, Slot(0), 4, 4, Keywords::none().with_lifesteal()
        ));

        // No defender

        let result = resolve_combat(
            &mut state,
            &card_db,
            &mut effect_queue,
            PlayerId::PLAYER_ONE,
            Slot(0),
            Slot(0),
        );

        // Lifesteal heals for face damage
        assert_eq!(result.attacker_healed, 4);

        // Player 1's life increased
        assert_eq!(state.players[0].life, 24); // 20 + 4 = 24
    }

    #[test]
    fn test_defender_has_quick() {
        let card_db = test_card_db();
        let mut state = test_game_state();
        let mut effect_queue = EffectQueue::new();

        // Player 1 has a 3/3 creature in slot 0 (no quick)
        state.players[0].creatures.push(create_test_creature(
            0, 1, PlayerId::PLAYER_ONE, Slot(0), 3, 3, Keywords::none()
        ));

        // Player 2 has a 3/3 quick creature in slot 0
        state.players[1].creatures.push(create_test_creature(
            1, 2, PlayerId::PLAYER_TWO, Slot(0), 3, 3, Keywords::none().with_quick()
        ));

        let result = resolve_combat(
            &mut state,
            &card_db,
            &mut effect_queue,
            PlayerId::PLAYER_ONE,
            Slot(0),
            Slot(0),
        );

        // Defender has Quick, so they strike first
        // Attacker dies before dealing damage
        assert!(result.attacker_died);
        assert!(!result.defender_died);
        assert_eq!(result.attacker_damage_dealt, 0);
        assert_eq!(result.defender_damage_dealt, 3);

        // Defender survives at full health
        let defender = state.players[1].get_creature(Slot(0)).unwrap();
        assert_eq!(defender.current_health, 3);
    }

    #[test]
    fn test_both_have_quick() {
        let card_db = test_card_db();
        let mut state = test_game_state();
        let mut effect_queue = EffectQueue::new();

        // Player 1 has a 2/4 quick creature in slot 0
        state.players[0].creatures.push(create_test_creature(
            0, 2, PlayerId::PLAYER_ONE, Slot(0), 2, 4, Keywords::none().with_quick()
        ));

        // Player 2 has a 2/4 quick creature in slot 0
        state.players[1].creatures.push(create_test_creature(
            1, 2, PlayerId::PLAYER_TWO, Slot(0), 2, 4, Keywords::none().with_quick()
        ));

        let result = resolve_combat(
            &mut state,
            &card_db,
            &mut effect_queue,
            PlayerId::PLAYER_ONE,
            Slot(0),
            Slot(0),
        );

        // Both have Quick = simultaneous damage
        // Both deal 2 damage, both survive
        assert_eq!(result.attacker_damage_dealt, 2);
        assert_eq!(result.defender_damage_dealt, 2);
        assert!(!result.attacker_died);
        assert!(!result.defender_died);

        // Both at 2 health
        let attacker = state.players[0].get_creature(Slot(0)).unwrap();
        assert_eq!(attacker.current_health, 2);
        let defender = state.players[1].get_creature(Slot(0)).unwrap();
        assert_eq!(defender.current_health, 2);
    }

    #[test]
    fn test_attacker_exhausted_after_combat() {
        let card_db = test_card_db();
        let mut state = test_game_state();
        let mut effect_queue = EffectQueue::new();

        // Player 1 has a 3/4 creature in slot 0
        state.players[0].creatures.push(create_test_creature(
            0, 1, PlayerId::PLAYER_ONE, Slot(0), 3, 4, Keywords::none()
        ));

        // No defender (face attack)

        let _result = resolve_combat(
            &mut state,
            &card_db,
            &mut effect_queue,
            PlayerId::PLAYER_ONE,
            Slot(0),
            Slot(0),
        );

        // Attacker should be exhausted
        let attacker = state.players[0].get_creature(Slot(0)).unwrap();
        assert!(attacker.status.is_exhausted());
    }

    #[test]
    fn test_game_over_on_lethal_face_damage() {
        let card_db = test_card_db();
        let mut state = test_game_state();
        let mut effect_queue = EffectQueue::new();

        // Set player 2's life to 3
        state.players[1].life = 3;

        // Player 1 has a 5/4 creature in slot 0
        state.players[0].creatures.push(create_test_creature(
            0, 1, PlayerId::PLAYER_ONE, Slot(0), 5, 4, Keywords::none()
        ));

        // No defender

        let _result = resolve_combat(
            &mut state,
            &card_db,
            &mut effect_queue,
            PlayerId::PLAYER_ONE,
            Slot(0),
            Slot(0),
        );

        // Game should be over
        assert!(state.is_terminal());
        assert!(matches!(
            state.result,
            Some(crate::state::GameResult::Win {
                winner: PlayerId::PLAYER_ONE,
                ..
            })
        ));
    }
}
