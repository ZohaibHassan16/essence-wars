//! Stat modification effect handlers (buff, debuff, set stats).

use crate::core::effects::{CreatureFilter, EffectTarget};
use crate::core::types::{PlayerId, Slot};

use super::super::effect_context::{EffectContext, TargetResolver};
use super::EffectHandler;

/// Handler for BuffStats effects.
pub struct BuffStatsHandler {
    pub target: EffectTarget,
    pub attack: i8,
    pub health: i8,
    pub filter: Option<CreatureFilter>,
}

impl EffectHandler for BuffStatsHandler {
    fn apply(&self, ctx: &mut EffectContext) -> bool {
        let targets = TargetResolver::collect_creature_targets(&self.target, self.filter.as_ref(), ctx.state);

        for (owner, slot) in targets {
            buff_creature(ctx, owner, slot, self.attack, self.health);
        }
        true
    }
}

/// Handler for SetStats effects.
pub struct SetStatsHandler {
    pub target: EffectTarget,
    pub attack: u8,
    pub health: u8,
}

impl EffectHandler for SetStatsHandler {
    fn apply(&self, ctx: &mut EffectContext) -> bool {
        if let EffectTarget::Creature { owner, slot } = self.target {
            if let Some(creature) = ctx.state.players[owner.index()].get_creature_mut(slot) {
                creature.attack = self.attack as i8;
                creature.current_health = self.health as i8;
                creature.max_health = self.health as i8;

                // Check for death
                if creature.current_health <= 0 {
                    ctx.mark_for_death(owner, slot);
                }
            }
        }
        true
    }
}

/// Buff a specific creature's stats.
fn buff_creature(ctx: &mut EffectContext, owner: PlayerId, slot: Slot, attack: i8, health: i8) {
    if let Some(creature) = ctx.state.players[owner.index()].get_creature_mut(slot) {
        // Use saturating arithmetic to prevent overflow
        creature.attack = creature.attack.saturating_add(attack);
        // Cap health at 0 to prevent negative values (debuffs can reduce health)
        creature.current_health = creature.current_health.saturating_add(health).max(0);
        if health > 0 {
            creature.max_health = creature.max_health.saturating_add(health);
        }

        // Check for death from negative health buff
        if creature.current_health == 0 {
            ctx.mark_for_death(owner, slot);
        }
    }
}
