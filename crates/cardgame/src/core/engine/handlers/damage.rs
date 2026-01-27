//! Damage and healing effect handlers.

use crate::core::effects::{CreatureFilter, EffectTarget, Trigger};
use crate::core::keywords::Keywords;
use crate::core::state::{GameResult, WinReason};
use crate::core::types::{PlayerId, Slot};

use super::super::effect_context::{EffectContext, ResolvedTargets, TargetResolver};
use super::super::triggers::check_creature_triggers;
use super::EffectHandler;

/// Handler for Damage effects.
pub struct DamageHandler {
    pub target: EffectTarget,
    pub amount: u8,
    pub filter: Option<CreatureFilter>,
}

impl EffectHandler for DamageHandler {
    fn apply(&self, ctx: &mut EffectContext) -> bool {
        let targets = TargetResolver::resolve(&self.target, self.filter.as_ref(), ctx.state);

        match targets {
            ResolvedTargets::Single { owner, slot } => {
                damage_creature(ctx, owner, slot, self.amount);
            }
            ResolvedTargets::Multiple(creatures) => {
                for (owner, slot) in creatures {
                    damage_creature(ctx, owner, slot, self.amount);
                }
            }
            ResolvedTargets::Player(player) => {
                damage_player(ctx, player, self.amount);
            }
            ResolvedTargets::None => {}
        }
        true
    }
}

/// Handler for Heal effects.
pub struct HealHandler {
    pub target: EffectTarget,
    pub amount: u8,
    pub filter: Option<CreatureFilter>,
}

impl EffectHandler for HealHandler {
    fn apply(&self, ctx: &mut EffectContext) -> bool {
        let targets = TargetResolver::resolve(&self.target, self.filter.as_ref(), ctx.state);

        match targets {
            ResolvedTargets::Single { owner, slot } => {
                heal_creature(ctx, owner, slot, self.amount);
            }
            ResolvedTargets::Multiple(creatures) => {
                for (owner, slot) in creatures {
                    heal_creature(ctx, owner, slot, self.amount);
                }
            }
            ResolvedTargets::Player(player) => {
                // Heal player (no max cap in current design per DESIGN.md)
                ctx.state.players[player.index()].life += self.amount as i16;
            }
            ResolvedTargets::None => {}
        }
        true
    }
}

/// Deal damage to a specific creature.
///
/// Handles Shield absorption, Fortify damage reduction, and death marking.
fn damage_creature(ctx: &mut EffectContext, owner: PlayerId, slot: Slot, amount: u8) {
    // First check if creature exists and has shield/fortify, get necessary info
    let creature_info = {
        let creature = match ctx.state.players[owner.index()].get_creature(slot) {
            Some(c) => c,
            None => return,
        };
        (creature.keywords.has_shield(), creature.keywords.has_fortify(), creature.current_health)
    };

    let (has_shield, has_fortify, old_health) = creature_info;

    if has_shield {
        // Shield absorbs the damage, remove shield
        if let Some(creature) = ctx.state.players[owner.index()].get_creature_mut(slot) {
            creature.keywords.remove(Keywords::SHIELD);
        }
        // No damage dealt, no triggers for OnTakeDamage with 0 damage
        return;
    }

    // FORTIFY: Reduce damage by 1 (minimum 1 damage still dealt)
    let actual_amount = if has_fortify && amount > 1 {
        amount - 1
    } else {
        amount
    };

    // Apply damage (cap at 0 to prevent negative health)
    let damage_dealt = actual_amount.min(old_health.max(0) as u8);
    let new_health = {
        let creature = ctx.state.players[owner.index()].get_creature_mut(slot).unwrap();
        creature.current_health = (creature.current_health - actual_amount as i8).max(0);
        creature.current_health
    };

    // Queue OnTakeDamage triggers
    if damage_dealt > 0 {
        check_creature_triggers(ctx, Trigger::OnTakeDamage, owner, slot);
    }

    // Check for death
    if new_health == 0 {
        ctx.mark_for_death(owner, slot);
    }
}

/// Deal damage to a player (commander).
fn damage_player(ctx: &mut EffectContext, player: PlayerId, amount: u8) {
    use crate::core::state::GamePhase;

    ctx.state.players[player.index()].life -= amount as i16;

    // Track total damage dealt
    let opponent = player.opponent();
    ctx.state.players[opponent.index()].total_damage_dealt += amount as u16;

    // Check for game over
    if ctx.state.players[player.index()].life <= 0 {
        ctx.state.result = Some(GameResult::Win {
            winner: player.opponent(),
            reason: WinReason::LifeReachedZero,
        });
        ctx.state.phase = GamePhase::Ended;
    }
}

/// Heal a specific creature.
fn heal_creature(ctx: &mut EffectContext, owner: PlayerId, slot: Slot, amount: u8) {
    if let Some(creature) = ctx.state.players[owner.index()].get_creature_mut(slot) {
        // Heal up to max health (use i16 to avoid overflow)
        // Cap at 0 minimum in case max_health went negative from passive removal
        creature.current_health = ((creature.current_health as i16) + (amount as i16))
            .min(creature.max_health as i16)
            .max(0) as i8;
    }
}
