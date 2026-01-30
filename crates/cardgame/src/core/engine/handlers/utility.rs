//! Utility effect handlers (draw, essence, refresh, bounce, destroy).

use crate::core::effects::{CreatureFilter, EffectTarget};
use crate::core::state::CardInstance;
use crate::core::types::{PlayerId, Slot};

use super::super::effect_context::{EffectContext, TargetResolver};
use super::EffectHandler;

/// Handler for Draw effects.
pub struct DrawHandler {
    pub player: PlayerId,
    pub count: u8,
}

impl EffectHandler for DrawHandler {
    fn apply(&self, ctx: &mut EffectContext) -> bool {
        let player_state = &mut ctx.state.players[self.player.index()];

        for _ in 0..self.count {
            if player_state.deck.is_empty() {
                // No cards to draw
                break;
            }

            let card = player_state.deck.remove(0);

            if player_state.is_hand_full() {
                // Hand is full, card is burned (discarded)
            } else {
                player_state.hand.push(card);
            }
        }
        true
    }

    fn blocked_by_ward(&self) -> bool {
        false // Draw doesn't target creatures
    }
}

/// Handler for GainEssence effects.
pub struct GainEssenceHandler {
    pub player: PlayerId,
    pub amount: u8,
}

impl EffectHandler for GainEssenceHandler {
    fn apply(&self, ctx: &mut EffectContext) -> bool {
        ctx.state.players[self.player.index()].current_essence =
            ctx.state.players[self.player.index()].current_essence.saturating_add(self.amount);
        true
    }

    fn blocked_by_ward(&self) -> bool {
        false // GainEssence doesn't target creatures
    }
}

/// Handler for RefreshCreature effects.
pub struct RefreshCreatureHandler {
    pub target: EffectTarget,
}

impl EffectHandler for RefreshCreatureHandler {
    fn apply(&self, ctx: &mut EffectContext) -> bool {
        if let EffectTarget::Creature { owner, slot } = self.target {
            if let Some(creature) = ctx.state.players[owner.index()].get_creature_mut(slot) {
                creature.status.set_exhausted(false);
            }
        }
        true
    }
}

/// Handler for Bounce effects (return creature to hand).
pub struct BounceHandler {
    pub target: EffectTarget,
    pub filter: Option<CreatureFilter>,
}

impl EffectHandler for BounceHandler {
    fn apply(&self, ctx: &mut EffectContext) -> bool {
        match &self.target {
            EffectTarget::Creature { owner, slot } => {
                bounce_creature(ctx, *owner, *slot, self.filter.as_ref());
            }
            EffectTarget::AllEnemyCreatures(player) => {
                let enemy = player.opponent();
                let creatures: Vec<_> = ctx.state.players[enemy.index()]
                    .creatures
                    .iter()
                    .filter(|c| self.filter.as_ref().is_none_or(|f| f.matches(c.current_health, c.keywords.0)))
                    .map(|c| c.slot)
                    .collect();
                for slot in creatures {
                    bounce_creature(ctx, enemy, slot, None);
                }
            }
            EffectTarget::AllAllyCreatures(player) => {
                let creatures: Vec<_> = ctx.state.players[player.index()]
                    .creatures
                    .iter()
                    .filter(|c| self.filter.as_ref().is_none_or(|f| f.matches(c.current_health, c.keywords.0)))
                    .map(|c| c.slot)
                    .collect();
                for slot in creatures {
                    bounce_creature(ctx, *player, slot, None);
                }
            }
            EffectTarget::AllCreatures => {
                for player in [PlayerId::PLAYER_ONE, PlayerId::PLAYER_TWO] {
                    let creatures: Vec<_> = ctx.state.players[player.index()]
                        .creatures
                        .iter()
                        .filter(|c| self.filter.as_ref().is_none_or(|f| f.matches(c.current_health, c.keywords.0)))
                        .map(|c| c.slot)
                        .collect();
                    for slot in creatures {
                        bounce_creature(ctx, player, slot, None);
                    }
                }
            }
            _ => {} // Player targets don't make sense for bounce
        }
        true
    }
}

/// Bounce a single creature (return to owner's hand).
fn bounce_creature(
    ctx: &mut EffectContext,
    owner: PlayerId,
    slot: Slot,
    filter: Option<&CreatureFilter>,
) {
    let player_state = &mut ctx.state.players[owner.index()];
    if let Some(creature) = player_state.get_creature(slot) {
        // Apply filter if present
        if let Some(f) = filter {
            if !f.matches(creature.current_health, creature.keywords.0) {
                return;
            }
        }
        let card_id = creature.card_id;
        // Remove creature from board
        player_state.creatures.retain(|c| c.slot != slot);
        // Add card back to hand if not full
        if !player_state.is_hand_full() {
            player_state.hand.push(CardInstance::new(card_id));
        }
        // Note: If hand is full, the card is simply lost
    }
}

/// Handler for Destroy effects.
pub struct DestroyHandler {
    pub target: EffectTarget,
    pub filter: Option<CreatureFilter>,
}

impl EffectHandler for DestroyHandler {
    fn apply(&self, ctx: &mut EffectContext) -> bool {
        let targets = TargetResolver::collect_creature_targets(&self.target, self.filter.as_ref(), ctx.state);

        for (owner, slot) in targets {
            if ctx.state.players[owner.index()].get_creature(slot).is_some() {
                ctx.mark_for_death(owner, slot);
            }
        }
        true
    }
}

/// Handler for DestroySelf effects (self-sacrifice for token abilities).
pub struct DestroySelfHandler {
    pub owner: PlayerId,
    pub slot: Slot,
}

impl EffectHandler for DestroySelfHandler {
    fn apply(&self, ctx: &mut EffectContext) -> bool {
        // Mark the creature for death (will trigger OnDeath effects)
        if ctx.state.players[self.owner.index()].get_creature(self.slot).is_some() {
            ctx.mark_for_death(self.owner, self.slot);
            true
        } else {
            false
        }
    }

    fn blocked_by_ward(&self) -> bool {
        false // Self-sacrifice cannot be warded
    }
}
