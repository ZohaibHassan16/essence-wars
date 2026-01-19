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
use crate::core::tracing::{CombatTrace, CombatTracer};
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
/// * `tracer` - Optional combat tracer for debugging
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
    tracer: Option<&mut CombatTracer>,
) -> CombatResult {
    let defender_player = attacker_player.opponent();

    // Get attacker creature - must exist
    let attacker = state.players[attacker_player.index()]
        .get_creature(attacker_slot)
        .expect("Attacker must exist");

    // CHARGE BONUS: +2 attack when attacking
    const CHARGE_BONUS: u8 = 2;
    let base_attack = attacker.attack.max(0) as u8;

    // Apply combat bonuses: Charge (+2) and Frenzy (accumulated stacks from prior attacks this turn)
    let mut attacker_attack = base_attack;
    if attacker.keywords.has_charge() {
        attacker_attack = attacker_attack.saturating_add(CHARGE_BONUS);
    }
    // FRENZY BONUS: +1 attack per stack (earned from previous attacks this turn)
    if attacker.keywords.has_frenzy() {
        attacker_attack = attacker_attack.saturating_add(attacker.frenzy_stacks);
    }
    let attacker_keywords = attacker.keywords;

    // STEALTH BREAK: When a creature attacks, it loses Stealth
    if attacker_keywords.has_stealth() {
        if let Some(attacker_mut) = state.players[attacker_player.index()]
            .get_creature_mut(attacker_slot)
        {
            attacker_mut.keywords.remove(Keywords::STEALTH);
        }
    }

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
            defender_slot,
            attacker_attack,
            attacker_keywords,
            tracer,
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
        tracer,
    )
}

/// Resolve a face attack (attacking an empty slot).
#[allow(clippy::too_many_arguments)]
fn resolve_face_attack(
    state: &mut GameState,
    card_db: &CardDatabase,
    effect_queue: &mut EffectQueue,
    attacker_player: PlayerId,
    attacker_slot: Slot,
    defender_player: PlayerId,
    defender_slot: Slot,
    attacker_attack: u8,
    attacker_keywords: Keywords,
    tracer: Option<&mut CombatTracer>,
) -> CombatResult {
    // Get attacker health for trace
    let attacker_health = state.players[attacker_player.index()]
        .get_creature(attacker_slot)
        .map(|c| c.current_health)
        .unwrap_or(0);

    // Create trace if enabled
    let mut trace = tracer.as_ref().and_then(|t| {
        if t.is_enabled() {
            Some(CombatTrace::new_face_attack(
                attacker_player,
                attacker_slot,
                attacker_attack as i8,
                attacker_health,
                attacker_keywords,
                defender_slot,
            ))
        } else {
            None
        }
    });

    let damage = attacker_attack;

    // Deal face damage
    state.players[defender_player.index()].life =
        state.players[defender_player.index()].life.saturating_sub(damage as i16);

    // Track total damage dealt
    state.players[attacker_player.index()].total_damage_dealt += damage as u16;

    // Log face damage
    if let Some(ref mut t) = trace {
        t.log_face_damage(damage);
    }

    // Apply Lifesteal
    let healed = if attacker_keywords.has_lifesteal() && damage > 0 {
        // Heal attacker's controller, capped at 30 (max life per DESIGN.md)
        let current_life = state.players[attacker_player.index()].life;
        let new_life = (current_life + damage as i16).min(30);
        let actual_heal = (new_life - current_life) as u8;
        state.players[attacker_player.index()].life = new_life;

        if let Some(ref mut t) = trace {
            t.log_lifesteal(actual_heal);
        }

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

    // Log completion
    if let Some(ref mut t) = trace {
        t.log_complete(false, false);
    }

    // Add trace to tracer
    if let (Some(tracer), Some(trace)) = (tracer, trace) {
        tracer.add_trace(trace);
    }

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
#[allow(clippy::too_many_arguments)]
fn resolve_creature_combat(
    state: &mut GameState,
    card_db: &CardDatabase,
    effect_queue: &mut EffectQueue,
    attacker_player: PlayerId,
    attacker_slot: Slot,
    defender_player: PlayerId,
    defender_slot: Slot,
    tracer: Option<&mut CombatTracer>,
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

    // Create trace if enabled
    let mut trace = tracer.as_ref().and_then(|t| {
        if t.is_enabled() {
            Some(CombatTrace::new_creature_combat(
                attacker_player,
                attacker_slot,
                attacker_attack as i8,
                attacker_health,
                attacker_keywords,
                defender_player,
                defender_slot,
                defender_attack as i8,
                defender_health,
                defender_keywords,
            ))
        } else {
            None
        }
    });

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

    // Log Quick check
    if let Some(ref mut t) = trace {
        t.log_quick_check(attacker_has_quick, defender_has_quick);
        if attacker_has_ranged {
            t.log_ranged();
        }
        if attacker_has_shield {
            t.log_shield_check("Attacker", true);
        }
        if defender_has_shield {
            t.log_shield_check("Defender", true);
        }
    }

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

    // Log damage dealt
    if let Some(ref mut t) = trace {
        t.log_damage(actual_damage_to_defender, actual_damage_to_attacker);
    }

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

            // Log piercing
            if let Some(ref mut t) = trace {
                t.log_piercing(excess);
            }
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

            // Log lifesteal
            if let Some(ref mut t) = trace {
                t.log_lifesteal(actual_heal);
            }
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

    // Log completion and add trace to tracer
    if let Some(ref mut t) = trace {
        t.log_complete(attacker_died, defender_died);
    }
    if let (Some(tracer), Some(trace)) = (tracer, trace) {
        tracer.add_trace(trace);
    }

    result
}

/// Apply combat damage to a creature, handling Shield, Fortify, and Lethal keywords.
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

    // FORTIFY: Reduce damage by 1 (minimum 1 damage still dealt)
    let actual_damage = if creature.keywords.has_fortify() && damage > 1 {
        damage - 1
    } else {
        damage
    };

    // Apply damage (cap at 0 to prevent negative health)
    creature.current_health = (creature.current_health - actual_damage as i8).max(0);
    let died = creature.current_health == 0;

    // Apply Lethal: any non-zero damage kills
    if attacker_has_lethal && actual_damage > 0 && !died {
        creature.current_health = 0;
        return (actual_damage, false, true);
    }

    (actual_damage, false, died)
}

/// Mark a creature as having attacked this turn.
/// Also increments frenzy_stacks if the creature has Frenzy keyword.
fn mark_attacked(state: &mut GameState, player: PlayerId, slot: Slot) {
    if let Some(creature) = state.players[player.index()].get_creature_mut(slot) {
        creature.status.set_exhausted(true);

        // FRENZY: Gain +1 attack stack for subsequent attacks this turn
        if creature.keywords.has_frenzy() {
            creature.frenzy_stacks = creature.frenzy_stacks.saturating_add(1);
        }
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
    let (card_id, is_silenced, has_volatile) = match state.players[player.index()].get_creature(slot) {
        Some(c) => (c.card_id, c.status.is_silenced(), c.keywords.has_volatile()),
        None => return,
    };

    // VOLATILE: Deal 2 damage to all enemy creatures on death (if not silenced)
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

    // Process deaths of enemy creatures killed by Volatile (can chain!)
    for enemy_slot in enemies_killed {
        process_creature_death(state, card_db, effect_queue, enemy_player, enemy_slot);
    }
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
