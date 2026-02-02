//! Summon effect handlers (summon, token, transform, copy).

use crate::core::cards::CardType;
use crate::core::effects::{EffectTarget, TokenDefinition, Trigger};
use crate::core::keywords::Keywords;
use crate::core::state::Creature;
use crate::core::types::{CardId, PlayerId, Slot};

use super::super::effect_context::EffectContext;
use super::super::passive::apply_commander_passive_from_cache;
use super::super::triggers::check_creature_triggers;
use super::EffectHandler;

/// Handler for Summon effects (summon from card database).
pub struct SummonHandler {
    pub owner: PlayerId,
    pub card_id: CardId,
    pub slot: Option<Slot>,
}

impl EffectHandler for SummonHandler {
    fn apply(&self, ctx: &mut EffectContext) -> bool {
        // Find the slot to use
        let target_slot = self.slot.or_else(|| ctx.state.players[self.owner.index()].find_empty_creature_slot());

        let Some(target_slot) = target_slot else {
            // No empty slot available
            return false;
        };

        // Check if slot is already occupied
        if ctx.state.players[self.owner.index()].get_creature(target_slot).is_some() {
            return false;
        }

        // Get card definition
        let Some(card_def) = ctx.card_db.get(self.card_id) else {
            return false;
        };

        // Must be a creature card
        let CardType::Creature { attack, health, .. } = &card_def.card_type else {
            return false;
        };

        // Create creature instance
        let instance_id = ctx.state.next_creature_instance_id();
        let keywords = card_def.keywords();

        let creature = Creature {
            instance_id,
            card_id: self.card_id,
            owner: self.owner,
            slot: target_slot,
            attack: *attack as i8,
            current_health: *health as i8,
            max_health: *health as i8,
            base_attack: *attack,
            base_health: *health,
            keywords,
            status: Default::default(),
            turn_played: ctx.state.current_turn,
            frenzy_stacks: 0,
            token_abilities: None, // Regular creatures don't have token abilities
            token_name: None, // Regular creatures get name from card database
        };

        ctx.state.players[self.owner.index()].creatures.push(creature);

        // Apply commander passive effects to the summoned creature
        let commander_passive = ctx.state.players[self.owner.index()].commander_passive;
        if let Some(creature) = ctx.state.players[self.owner.index()].get_creature_mut(target_slot) {
            apply_commander_passive_from_cache(creature, &commander_passive);
        }

        // Check if the summoned creature has <= 0 health (can happen with auras/debuffs)
        if let Some(creature) = ctx.state.players[self.owner.index()].get_creature(target_slot) {
            if creature.current_health <= 0 {
                ctx.mark_for_death(self.owner, target_slot);
            }
        }

        // Queue OnPlay triggers
        check_creature_triggers(ctx, Trigger::OnPlay, self.owner, target_slot);
        true
    }

    fn blocked_by_ward(&self) -> bool {
        false // Summon doesn't target existing creatures
    }
}

/// Handler for SummonToken effects (inline token definition).
pub struct SummonTokenHandler {
    pub owner: PlayerId,
    pub token: TokenDefinition,
    pub slot: Option<Slot>,
}

impl EffectHandler for SummonTokenHandler {
    fn apply(&self, ctx: &mut EffectContext) -> bool {
        // Find the slot to use
        let target_slot = self.slot.or_else(|| ctx.state.players[self.owner.index()].find_empty_creature_slot());

        let Some(target_slot) = target_slot else {
            // No empty slot available
            return false;
        };

        // Check if slot is already occupied
        if ctx.state.players[self.owner.index()].get_creature(target_slot).is_some() {
            return false;
        }

        // Create token creature instance
        // Use CardId(0) as a sentinel value for tokens (not a real card)
        let instance_id = ctx.state.next_creature_instance_id();
        let keywords = Keywords(self.token.keywords);

        // Convert token abilities if present
        let token_abilities = if self.token.abilities.is_empty() {
            None
        } else {
            Some(self.token.abilities.clone())
        };

        let creature = Creature {
            instance_id,
            card_id: CardId(0), // Token marker
            owner: self.owner,
            slot: target_slot,
            attack: self.token.attack as i8,
            current_health: self.token.health as i8,
            max_health: self.token.health as i8,
            base_attack: self.token.attack,
            base_health: self.token.health,
            keywords,
            status: Default::default(),
            turn_played: ctx.state.current_turn,
            frenzy_stacks: 0,
            token_abilities,
            token_name: Some(self.token.name.clone()), // Store token name for UI display
        };

        ctx.state.players[self.owner.index()].creatures.push(creature);

        // Apply commander passive effects to the summoned token
        let commander_passive = ctx.state.players[self.owner.index()].commander_passive;
        if let Some(creature) = ctx.state.players[self.owner.index()].get_creature_mut(target_slot) {
            apply_commander_passive_from_cache(creature, &commander_passive);
        }

        // Check if the summoned token has <= 0 health (edge case for 0-health tokens)
        if let Some(creature) = ctx.state.players[self.owner.index()].get_creature(target_slot) {
            if creature.current_health <= 0 {
                ctx.mark_for_death(self.owner, target_slot);
            }
        }

        // Note: Tokens don't trigger OnPlay since they're not "played from hand"
        true
    }

    fn blocked_by_ward(&self) -> bool {
        false // SummonToken doesn't target existing creatures
    }
}

/// Handler for Transform effects.
pub struct TransformHandler {
    pub target: EffectTarget,
    pub into: TokenDefinition,
}

impl EffectHandler for TransformHandler {
    fn apply(&self, ctx: &mut EffectContext) -> bool {
        let EffectTarget::Creature { owner, slot } = self.target else {
            return false;
        };

        // Remove the existing creature
        let existed = ctx.state.players[owner.index()].get_creature(slot).is_some();
        if !existed {
            return false;
        }
        ctx.state.players[owner.index()].creatures.retain(|c| c.slot != slot);

        // Create the transformed creature in the same slot
        let instance_id = ctx.state.next_creature_instance_id();
        let keywords = Keywords(self.into.keywords);

        // Convert token abilities if present
        let token_abilities = if self.into.abilities.is_empty() {
            None
        } else {
            Some(self.into.abilities.clone())
        };

        let creature = Creature {
            instance_id,
            card_id: CardId(0), // Token marker
            owner,
            slot,
            attack: self.into.attack as i8,
            current_health: self.into.health as i8,
            max_health: self.into.health as i8,
            base_attack: self.into.attack,
            base_health: self.into.health,
            keywords,
            status: Default::default(),
            turn_played: ctx.state.current_turn,
            frenzy_stacks: 0,
            token_abilities,
            token_name: Some(self.into.name.clone()), // Store transformed token name
        };

        // Check if the transformed creature has <= 0 health before adding (edge case)
        let is_dead = creature.current_health <= 0;

        ctx.state.players[owner.index()].creatures.push(creature);

        // Apply commander passive effects to the transformed creature
        let commander_passive = ctx.state.players[owner.index()].commander_passive;
        if let Some(creature) = ctx.state.players[owner.index()].get_creature_mut(slot) {
            apply_commander_passive_from_cache(creature, &commander_passive);
        }

        if is_dead {
            ctx.mark_for_death(owner, slot);
        }

        // Note: Transform doesn't trigger OnDeath or OnPlay
        true
    }
}

/// Handler for Copy effects.
pub struct CopyHandler {
    pub target: EffectTarget,
    pub owner: PlayerId,
}

impl EffectHandler for CopyHandler {
    fn apply(&self, ctx: &mut EffectContext) -> bool {
        let EffectTarget::Creature { owner: source_owner, slot: source_slot } = self.target else {
            return false;
        };

        // Get the source creature's stats
        let source = match ctx.state.players[source_owner.index()].get_creature(source_slot) {
            Some(c) => c,
            None => return false,
        };

        // Copy base stats (not current stats)
        let card_id = source.card_id;
        let base_attack = source.base_attack;
        let base_health = source.base_health;
        let keywords = source.keywords;
        let token_abilities = source.token_abilities.clone();
        let token_name = source.token_name.clone();

        // Find empty slot for the copy
        let target_slot = match ctx.state.players[self.owner.index()].find_empty_creature_slot() {
            Some(s) => s,
            None => return false, // No empty slot
        };

        // Create the copy
        let instance_id = ctx.state.next_creature_instance_id();

        let creature = Creature {
            instance_id,
            card_id,
            owner: self.owner,
            slot: target_slot,
            attack: base_attack as i8,
            current_health: base_health as i8,
            max_health: base_health as i8,
            base_attack,
            base_health,
            keywords,
            status: Default::default(),
            turn_played: ctx.state.current_turn,
            frenzy_stacks: 0,
            token_abilities,
            token_name, // Copy token name from source (if it was a token)
        };

        ctx.state.players[self.owner.index()].creatures.push(creature);

        // Apply commander passive effects to the copied creature
        let commander_passive = ctx.state.players[self.owner.index()].commander_passive;
        if let Some(creature) = ctx.state.players[self.owner.index()].get_creature_mut(target_slot) {
            apply_commander_passive_from_cache(creature, &commander_passive);
        }

        // Check if the copied creature has <= 0 health (can happen if copying damaged creature)
        if let Some(creature) = ctx.state.players[self.owner.index()].get_creature(target_slot) {
            if creature.current_health <= 0 {
                ctx.mark_for_death(self.owner, target_slot);
            }
        }

        // Note: Copy doesn't trigger OnPlay since it's not played from hand
        true
    }
}
