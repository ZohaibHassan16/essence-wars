//! Keyword manipulation effect handlers (grant, remove, silence).

use crate::core::effects::{CreatureFilter, EffectTarget};

use super::super::effect_context::EffectContext;
use super::EffectHandler;

/// Handler for GrantKeyword effects.
pub struct GrantKeywordHandler {
    pub target: EffectTarget,
    pub keyword: u16,
    pub filter: Option<CreatureFilter>,
}

impl EffectHandler for GrantKeywordHandler {
    fn apply(&self, ctx: &mut EffectContext) -> bool {
        match &self.target {
            EffectTarget::Creature { owner, slot } => {
                if let Some(creature) = ctx.state.players[owner.index()].get_creature_mut(*slot) {
                    creature.keywords.add(self.keyword);
                }
            }
            EffectTarget::AllCreatures => {
                for player in &mut ctx.state.players {
                    for creature in &mut player.creatures {
                        if self.filter.as_ref().is_none_or(|f| f.matches(creature.current_health, creature.keywords.0)) {
                            creature.keywords.add(self.keyword);
                        }
                    }
                }
            }
            EffectTarget::AllAllyCreatures(player) => {
                for creature in &mut ctx.state.players[player.index()].creatures {
                    if self.filter.as_ref().is_none_or(|f| f.matches(creature.current_health, creature.keywords.0)) {
                        creature.keywords.add(self.keyword);
                    }
                }
            }
            EffectTarget::AllEnemyCreatures(player) => {
                let enemy = player.opponent();
                for creature in &mut ctx.state.players[enemy.index()].creatures {
                    if self.filter.as_ref().is_none_or(|f| f.matches(creature.current_health, creature.keywords.0)) {
                        creature.keywords.add(self.keyword);
                    }
                }
            }
            _ => {}
        }
        true
    }
}

/// Handler for RemoveKeyword effects.
pub struct RemoveKeywordHandler {
    pub target: EffectTarget,
    pub keyword: u16,
    pub filter: Option<CreatureFilter>,
}

impl EffectHandler for RemoveKeywordHandler {
    fn apply(&self, ctx: &mut EffectContext) -> bool {
        match &self.target {
            EffectTarget::Creature { owner, slot } => {
                if let Some(creature) = ctx.state.players[owner.index()].get_creature_mut(*slot) {
                    creature.keywords.remove(self.keyword);
                }
            }
            EffectTarget::AllCreatures => {
                for player in &mut ctx.state.players {
                    for creature in &mut player.creatures {
                        if self.filter.as_ref().is_none_or(|f| f.matches(creature.current_health, creature.keywords.0)) {
                            creature.keywords.remove(self.keyword);
                        }
                    }
                }
            }
            _ => {}
        }
        true
    }
}

/// Handler for Silence effects.
pub struct SilenceHandler {
    pub target: EffectTarget,
    /// Filter is accepted for API consistency but not used for single-target silence.
    #[allow(dead_code)]
    pub filter: Option<CreatureFilter>,
}

impl EffectHandler for SilenceHandler {
    fn apply(&self, ctx: &mut EffectContext) -> bool {
        // Note: filter not used for single-target silence, but accepted for API consistency
        if let EffectTarget::Creature { owner, slot } = self.target {
            if let Some(creature) = ctx.state.players[owner.index()].get_creature_mut(slot) {
                creature.keywords.clear();
                creature.status.set_silenced(true);
            }
        }
        true
    }
}
