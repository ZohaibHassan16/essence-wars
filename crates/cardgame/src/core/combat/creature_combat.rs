//! Creature vs creature combat resolution.
//!
//! Handles combat between two creatures including all keyword interactions:
//! - Quick (first strike)
//! - Shield (damage absorption)
//! - Ranged (no counter-attack)
//! - Piercing (overflow damage to face)
//! - Lethal (instant kill on any damage)
//! - Lifesteal (heal for damage dealt)
//! - Fortify (damage reduction)

use crate::core::cards::CardDatabase;
use crate::core::effects::Trigger;
use crate::core::engine::EffectQueue;
use crate::core::keywords::Keywords;
use crate::core::state::GameState;
use crate::core::tracing::{CombatTrace, CombatTracer};
use crate::core::types::{PlayerId, Slot};

use super::death::{check_game_over, process_creature_death};
use super::face_attack::mark_attacked;
use super::triggers::trigger_creature_ability;
use super::CombatResult;

/// Resolve creature vs creature combat with all keyword interactions.
#[allow(clippy::too_many_arguments)]
pub fn resolve_creature_combat(
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
    let attacker_stats = get_creature_stats(state, attacker_player, attacker_slot);
    let defender_stats = get_creature_stats(state, defender_player, defender_slot);

    // Create trace if enabled
    let mut trace = create_combat_trace(
        tracer.as_ref(),
        attacker_player,
        attacker_slot,
        &attacker_stats,
        defender_player,
        defender_slot,
        &defender_stats,
    );

    // Log keyword checks
    log_keyword_checks(&mut trace, &attacker_stats, &defender_stats);

    // Trigger OnAttack effect before combat damage
    trigger_creature_ability(state, card_db, effect_queue, attacker_player, attacker_slot, Trigger::OnAttack);

    // Resolve combat damage based on Quick keyword
    let mut result = resolve_combat_damage(
        state,
        attacker_player,
        attacker_slot,
        defender_player,
        defender_slot,
        &attacker_stats,
        &defender_stats,
    );

    // Log damage dealt
    if let Some(ref mut t) = trace {
        t.log_damage(result.attacker_damage_dealt, result.defender_damage_dealt);
    }

    // Apply Piercing: excess damage to face when defender dies
    apply_piercing(state, &attacker_stats, &defender_stats, &mut result, &mut trace);

    // Apply Lifesteal: heal for damage actually dealt to creatures
    apply_lifesteal_creature(state, attacker_player, &attacker_stats, &defender_stats, &mut result, &mut trace);

    // Track damage dealt to creatures
    state.players[attacker_player.index()].total_damage_dealt += result.attacker_damage_dealt as u16;

    // Mark attacker as having attacked
    mark_attacked(state, attacker_player, attacker_slot);

    // Trigger damage-related effects
    trigger_damage_effects(
        state,
        card_db,
        effect_queue,
        attacker_player,
        attacker_slot,
        defender_player,
        defender_slot,
        &result,
    );

    // Process deaths and trigger OnDeath/OnKill effects
    process_combat_deaths(
        state,
        card_db,
        effect_queue,
        attacker_player,
        attacker_slot,
        defender_player,
        defender_slot,
        &result,
    );

    // Check for game over
    check_game_over(state);

    // Log completion and add trace to tracer
    if let Some(ref mut t) = trace {
        t.log_complete(result.attacker_died, result.defender_died);
    }
    if let (Some(tracer), Some(trace)) = (tracer, trace) {
        tracer.add_trace(trace);
    }

    result
}

/// Creature stats needed for combat resolution.
struct CreatureStats {
    attack: u8,
    health: i8,
    keywords: Keywords,
}

impl CreatureStats {
    fn has_quick(&self) -> bool {
        self.keywords.has_quick()
    }

    fn has_shield(&self) -> bool {
        self.keywords.has_shield()
    }

    fn has_ranged(&self) -> bool {
        self.keywords.has_ranged()
    }

    fn has_piercing(&self) -> bool {
        self.keywords.has_piercing()
    }

    fn has_lethal(&self) -> bool {
        self.keywords.has_lethal()
    }

    fn has_lifesteal(&self) -> bool {
        self.keywords.has_lifesteal()
    }
}

/// Get creature stats for combat.
fn get_creature_stats(state: &GameState, player: PlayerId, slot: Slot) -> CreatureStats {
    let creature = state.players[player.index()]
        .get_creature(slot)
        .expect("Creature must exist");
    CreatureStats {
        attack: creature.attack.max(0) as u8,
        health: creature.current_health,
        keywords: creature.keywords,
    }
}

/// Create a combat trace if tracing is enabled.
fn create_combat_trace(
    tracer: Option<&&mut CombatTracer>,
    attacker_player: PlayerId,
    attacker_slot: Slot,
    attacker_stats: &CreatureStats,
    defender_player: PlayerId,
    defender_slot: Slot,
    defender_stats: &CreatureStats,
) -> Option<CombatTrace> {
    tracer.and_then(|t| {
        if t.is_enabled() {
            Some(CombatTrace::new_creature_combat(
                attacker_player,
                attacker_slot,
                attacker_stats.attack as i8,
                attacker_stats.health,
                attacker_stats.keywords,
                defender_player,
                defender_slot,
                defender_stats.attack as i8,
                defender_stats.health,
                defender_stats.keywords,
            ))
        } else {
            None
        }
    })
}

/// Log keyword checks to the trace.
fn log_keyword_checks(
    trace: &mut Option<CombatTrace>,
    attacker_stats: &CreatureStats,
    defender_stats: &CreatureStats,
) {
    if let Some(ref mut t) = trace {
        t.log_quick_check(attacker_stats.has_quick(), defender_stats.has_quick());
        if attacker_stats.has_ranged() {
            t.log_ranged();
        }
        if attacker_stats.has_shield() {
            t.log_shield_check("Attacker", true);
        }
        if defender_stats.has_shield() {
            t.log_shield_check("Defender", true);
        }
    }
}

/// Resolve combat damage based on Quick keyword timing.
fn resolve_combat_damage(
    state: &mut GameState,
    attacker_player: PlayerId,
    attacker_slot: Slot,
    defender_player: PlayerId,
    defender_slot: Slot,
    attacker_stats: &CreatureStats,
    defender_stats: &CreatureStats,
) -> CombatResult {
    let mut result = CombatResult::default();

    let attacker_has_quick = attacker_stats.has_quick();
    let defender_has_quick = defender_stats.has_quick();
    let attacker_has_ranged = attacker_stats.has_ranged();

    if attacker_has_quick && !defender_has_quick {
        // Attacker strikes first
        resolve_attacker_first(
            state,
            attacker_player,
            attacker_slot,
            defender_player,
            defender_slot,
            attacker_stats,
            defender_stats,
            attacker_has_ranged,
            &mut result,
        );
    } else if defender_has_quick && !attacker_has_quick {
        // Defender strikes first
        resolve_defender_first(
            state,
            attacker_player,
            attacker_slot,
            defender_player,
            defender_slot,
            attacker_stats,
            defender_stats,
            attacker_has_ranged,
            &mut result,
        );
    } else {
        // Simultaneous damage
        resolve_simultaneous(
            state,
            attacker_player,
            attacker_slot,
            defender_player,
            defender_slot,
            attacker_stats,
            defender_stats,
            attacker_has_ranged,
            &mut result,
        );
    }

    result
}

/// Resolve combat when attacker strikes first (has Quick, defender doesn't).
#[allow(clippy::too_many_arguments)]
fn resolve_attacker_first(
    state: &mut GameState,
    attacker_player: PlayerId,
    attacker_slot: Slot,
    defender_player: PlayerId,
    defender_slot: Slot,
    attacker_stats: &CreatureStats,
    defender_stats: &CreatureStats,
    attacker_has_ranged: bool,
    result: &mut CombatResult,
) {
    let (damage_dealt, _, defender_died) = apply_combat_damage(
        state,
        defender_player,
        defender_slot,
        attacker_stats.attack,
        attacker_stats.has_lethal(),
        defender_stats.has_shield(),
    );

    result.attacker_damage_dealt = damage_dealt;
    result.defender_died = defender_died;

    // If defender survived, they counter-attack (unless attacker has Ranged)
    if !defender_died && !attacker_has_ranged {
        let (damage_dealt, _, attacker_died) = apply_combat_damage(
            state,
            attacker_player,
            attacker_slot,
            defender_stats.attack,
            defender_stats.has_lethal(),
            attacker_stats.has_shield(),
        );

        result.defender_damage_dealt = damage_dealt;
        result.attacker_died = attacker_died;
    }
}

/// Resolve combat when defender strikes first (has Quick, attacker doesn't).
#[allow(clippy::too_many_arguments)]
fn resolve_defender_first(
    state: &mut GameState,
    attacker_player: PlayerId,
    attacker_slot: Slot,
    defender_player: PlayerId,
    defender_slot: Slot,
    attacker_stats: &CreatureStats,
    defender_stats: &CreatureStats,
    attacker_has_ranged: bool,
    result: &mut CombatResult,
) {
    // Defender strikes first, but only if attacker doesn't have Ranged
    if !attacker_has_ranged {
        let (damage_dealt, _, attacker_died) = apply_combat_damage(
            state,
            attacker_player,
            attacker_slot,
            defender_stats.attack,
            defender_stats.has_lethal(),
            attacker_stats.has_shield(),
        );

        result.defender_damage_dealt = damage_dealt;
        result.attacker_died = attacker_died;
    }

    // If attacker survived, they deal damage
    if !result.attacker_died {
        let (damage_dealt, _, defender_died) = apply_combat_damage(
            state,
            defender_player,
            defender_slot,
            attacker_stats.attack,
            attacker_stats.has_lethal(),
            defender_stats.has_shield(),
        );

        result.attacker_damage_dealt = damage_dealt;
        result.defender_died = defender_died;
    }
}

/// Resolve simultaneous combat damage (both have Quick or neither has Quick).
#[allow(clippy::too_many_arguments)]
fn resolve_simultaneous(
    state: &mut GameState,
    attacker_player: PlayerId,
    attacker_slot: Slot,
    defender_player: PlayerId,
    defender_slot: Slot,
    attacker_stats: &CreatureStats,
    defender_stats: &CreatureStats,
    attacker_has_ranged: bool,
    result: &mut CombatResult,
) {
    // Apply attacker's damage to defender
    let (atk_damage_dealt, _, defender_died) = apply_combat_damage(
        state,
        defender_player,
        defender_slot,
        attacker_stats.attack,
        attacker_stats.has_lethal(),
        defender_stats.has_shield(),
    );

    result.attacker_damage_dealt = atk_damage_dealt;
    result.defender_died = defender_died;

    // Apply defender's counter-attack damage to attacker (unless Ranged)
    if !attacker_has_ranged {
        let (def_damage_dealt, _, attacker_died) = apply_combat_damage(
            state,
            attacker_player,
            attacker_slot,
            defender_stats.attack,
            defender_stats.has_lethal(),
            attacker_stats.has_shield(),
        );

        result.defender_damage_dealt = def_damage_dealt;
        result.attacker_died = attacker_died;
    }
}

/// Apply combat damage to a creature, handling Shield, Fortify, and Lethal keywords.
///
/// Returns (damage_dealt, was_blocked_by_shield, creature_died)
pub fn apply_combat_damage(
    state: &mut GameState,
    target_player: PlayerId,
    target_slot: Slot,
    damage: u8,
    attacker_has_lethal: bool,
    target_has_shield: bool,
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

/// Apply Piercing overflow damage to face.
fn apply_piercing(
    state: &mut GameState,
    attacker_stats: &CreatureStats,
    defender_stats: &CreatureStats,
    result: &mut CombatResult,
    trace: &mut Option<CombatTrace>,
) {
    if attacker_stats.has_piercing() && result.defender_died && result.attacker_damage_dealt > 0 {
        let defender_health_before = defender_stats.health as u8;
        if attacker_stats.attack > defender_health_before {
            let excess = attacker_stats.attack - defender_health_before;
            let defender_player_idx = 1; // Defender is always opponent
            state.players[defender_player_idx].life =
                state.players[defender_player_idx].life.saturating_sub(excess as i16);
            result.face_damage = excess;

            // Track piercing damage dealt
            state.players[0].total_damage_dealt += excess as u16;

            if let Some(ref mut t) = trace {
                t.log_piercing(excess);
            }
        }
    }
}

/// Apply Lifesteal healing in creature combat.
fn apply_lifesteal_creature(
    state: &mut GameState,
    attacker_player: PlayerId,
    attacker_stats: &CreatureStats,
    defender_stats: &CreatureStats,
    result: &mut CombatResult,
    trace: &mut Option<CombatTrace>,
) {
    if attacker_stats.has_lifesteal() && result.attacker_damage_dealt > 0 {
        // Heal for damage dealt to the creature (not piercing overflow)
        // Cap the heal at the defender's health before combat
        let heal_amount = result.attacker_damage_dealt.min(defender_stats.health.max(0) as u8);
        if heal_amount > 0 {
            let current_life = state.players[attacker_player.index()].life;
            let new_life = (current_life + heal_amount as i16).min(30);
            let actual_heal = (new_life - current_life) as u8;
            state.players[attacker_player.index()].life = new_life;
            result.attacker_healed = actual_heal;

            if let Some(ref mut t) = trace {
                t.log_lifesteal(actual_heal);
            }
        }
    }
}

/// Trigger damage-related effects after combat.
#[allow(clippy::too_many_arguments)]
fn trigger_damage_effects(
    state: &mut GameState,
    card_db: &CardDatabase,
    effect_queue: &mut EffectQueue,
    attacker_player: PlayerId,
    attacker_slot: Slot,
    defender_player: PlayerId,
    defender_slot: Slot,
    result: &CombatResult,
) {
    // Trigger OnDealDamage effects
    if result.attacker_damage_dealt > 0 {
        trigger_creature_ability(state, card_db, effect_queue, attacker_player, attacker_slot, Trigger::OnDealDamage);
    }
    if result.defender_damage_dealt > 0 {
        trigger_creature_ability(state, card_db, effect_queue, defender_player, defender_slot, Trigger::OnDealDamage);
    }

    // Trigger OnTakeDamage effects
    if result.attacker_damage_dealt > 0 {
        trigger_creature_ability(state, card_db, effect_queue, defender_player, defender_slot, Trigger::OnTakeDamage);
    }
    if result.defender_damage_dealt > 0 {
        trigger_creature_ability(state, card_db, effect_queue, attacker_player, attacker_slot, Trigger::OnTakeDamage);
    }
}

/// Process deaths after combat.
#[allow(clippy::too_many_arguments)]
fn process_combat_deaths(
    state: &mut GameState,
    card_db: &CardDatabase,
    effect_queue: &mut EffectQueue,
    attacker_player: PlayerId,
    attacker_slot: Slot,
    defender_player: PlayerId,
    defender_slot: Slot,
    result: &CombatResult,
) {
    use crate::core::engine::collect_commander_kill_effects;

    if result.defender_died {
        // Trigger commander OnKill effects (e.g., Shadow Emperor Kael)
        for (effect, source) in collect_commander_kill_effects(state, attacker_player, card_db) {
            effect_queue.push(effect, source);
        }
        // Trigger creature OnKill effects
        trigger_creature_ability(state, card_db, effect_queue, attacker_player, attacker_slot, Trigger::OnKill);
        process_creature_death(state, card_db, effect_queue, defender_player, defender_slot);
    }
    if result.attacker_died {
        // Note: defender doesn't get OnKill trigger since they're defending
        process_creature_death(state, card_db, effect_queue, attacker_player, attacker_slot);
    }
}
