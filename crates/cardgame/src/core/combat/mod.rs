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

mod creature_combat;
mod death;
mod face_attack;
mod triggers;

use crate::core::cards::CardDatabase;
use crate::core::engine::EffectQueue;
use crate::core::keywords::Keywords;
use crate::core::state::GameState;
use crate::core::tracing::CombatTracer;
use crate::core::types::{PlayerId, Slot};

// Re-export key items
pub use creature_combat::apply_combat_damage;
pub use death::{check_game_over, process_creature_death};

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
        return face_attack::resolve_face_attack(
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
    creature_combat::resolve_creature_combat(
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
