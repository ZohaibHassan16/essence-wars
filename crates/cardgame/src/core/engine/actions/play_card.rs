//! PlayCard action execution.
//!
//! Handles playing creatures, spells, and supports with full effect queue integration.

use crate::core::cards::{AbilityDefinition, CardType, ConditionalEffectGroup, EffectDefinition};
use crate::core::effects::{EffectSource, TargetingRule, Trigger};
use crate::core::engine::effect_convert::{
    effect_def_to_effect_with_target, effect_def_to_triggered_effect, resolve_spell_target,
};
use crate::core::engine::passive::{
    apply_all_support_passives_to_creature, apply_commander_passive_from_cache,
    apply_support_passives_to_all_creatures, collect_commander_creature_played_effects,
    support_effect_def_to_effect,
};
use crate::core::state::{Creature, CreatureStatus, Support};
use crate::core::types::Slot;

use super::ActionContext;

/// Execute a PlayCard action.
///
/// Handles creatures, spells, and supports with full effect queue integration:
/// - Creatures: Place on board with summoning sickness (unless Rush), trigger OnPlay
/// - Spells: Resolve targeting, queue effects, process queue
/// - Supports: Place in support slot, trigger OnPlay if present
pub fn execute_play_card(ctx: &mut ActionContext, hand_index: usize, slot: Slot) -> Result<(), String> {
    let current_player = ctx.state.active_player;

    // Get the card from hand
    if hand_index >= ctx.state.players[current_player.index()].hand.len() {
        return Err("Invalid hand index".to_string());
    }
    let card_instance = ctx.state.players[current_player.index()].hand.remove(hand_index);
    let card_id = card_instance.card_id;

    // Look up card definition
    let card_def = ctx
        .card_db
        .get(card_id)
        .ok_or_else(|| "Card not found in database".to_string())?;

    // Check and deduct costs
    // Per design: playing a card costs 1 AP + card's Essence cost
    let player_state = &mut ctx.state.players[current_player.index()];

    // Check AP (1 AP per card play)
    if player_state.action_points < 1 {
        return Err("Not enough AP".to_string());
    }

    // Check Essence (card's cost)
    if card_def.cost > player_state.current_essence {
        return Err("Not enough Essence".to_string());
    }

    // Deduct 1 AP for the action
    player_state.action_points -= 1;

    // Deduct Essence equal to card cost
    player_state.current_essence -= card_def.cost;

    match &card_def.card_type {
        CardType::Creature { attack, health, abilities, .. } => {
            play_creature(ctx, card_id, slot, *attack, *health, abilities)?;
        }
        CardType::Spell { targeting, effects, conditional_effects } => {
            play_spell(ctx, card_id, slot, targeting, effects, conditional_effects)?;
        }
        CardType::Support { durability, triggered_effects, .. } => {
            play_support(ctx, card_id, slot, *durability, triggered_effects)?;
        }
    }

    Ok(())
}

/// Play a creature card to the battlefield.
fn play_creature(
    ctx: &mut ActionContext,
    card_id: crate::core::types::CardId,
    slot: Slot,
    attack: u8,
    health: u8,
    abilities: &[AbilityDefinition],
) -> Result<(), String> {
    let current_player = ctx.state.active_player;
    let current_turn = ctx.state.current_turn;

    // Look up card definition for keywords
    let card_def = ctx.card_db.get(card_id).ok_or("Card not found")?;
    let keywords = card_def.keywords();

    // Create creature instance
    let instance_id = ctx.state.next_creature_instance_id();

    let creature = Creature {
        instance_id,
        card_id,
        owner: current_player,
        slot,
        attack: attack as i8,
        current_health: health as i8,
        max_health: health as i8,
        base_attack: attack,
        base_health: health,
        keywords,
        status: CreatureStatus::default(),
        turn_played: current_turn,
        frenzy_stacks: 0,
        token_abilities: None, // Regular creatures don't have token abilities
        token_name: None, // Regular creatures get name from card database
    };

    // Add creature to board
    ctx.state.players[current_player.index()].creatures.push(creature);

    // Apply passive effects from existing supports to the new creature
    let supports: Vec<Support> = ctx.state.players[current_player.index()]
        .supports.iter().cloned().collect();
    if let Some(new_creature) = ctx.state.players[current_player.index()]
        .get_creature_mut(slot) {
        apply_all_support_passives_to_creature(new_creature, &supports, ctx.card_db);
    }

    // Apply commander passive effects to the new creature (using cached passive)
    let commander_passive = ctx.state.players[current_player.index()].commander_passive;
    if let Some(new_creature) = ctx.state.players[current_player.index()]
        .get_creature_mut(slot) {
        apply_commander_passive_from_cache(new_creature, &commander_passive);
    }

    // Queue OnPlay triggered effects
    for ability in abilities {
        if ability.trigger == Trigger::OnPlay {
            let source = EffectSource::Creature { owner: current_player, slot };
            for effect_def in &ability.effects {
                if let Some(effect) = effect_def_to_triggered_effect(
                    effect_def,
                    current_player,
                    slot,
                    ability,
                ) {
                    ctx.effect_queue.push(effect, source);
                }
            }
        }
    }

    // Check for OnAllyPlayed triggers on other friendly creatures
    let other_creatures: Vec<(Slot, crate::core::types::CardId)> = ctx.state.players[current_player.index()]
        .creatures
        .iter()
        .filter(|c| c.slot != slot && !c.status.is_silenced())
        .map(|c| (c.slot, c.card_id))
        .collect();

    for (ally_slot, ally_card_id) in other_creatures {
        if let Some(ally_card_def) = ctx.card_db.get(ally_card_id) {
            if let Some(ally_abilities) = ally_card_def.creature_abilities() {
                for ability in ally_abilities {
                    if ability.trigger == Trigger::OnAllyPlayed {
                        let source = EffectSource::Creature {
                            owner: current_player,
                            slot: ally_slot,
                        };
                        for effect_def in &ability.effects {
                            if let Some(effect) = effect_def_to_triggered_effect(
                                effect_def,
                                current_player,
                                ally_slot,
                                ability,
                            ) {
                                ctx.effect_queue.push(effect, source);
                            }
                        }
                    }
                }
            }
        }
    }

    // Check for OnCreaturePlayed commander trigger (e.g., The Broodmother)
    for (effect, source) in collect_commander_creature_played_effects(
        ctx.state,
        current_player,
        keywords,
        ctx.card_db,
    ) {
        ctx.effect_queue.push(effect, source);
    }

    // Process all queued effects
    ctx.process_effects();

    Ok(())
}

/// Cast a spell card.
fn play_spell(
    ctx: &mut ActionContext,
    card_id: crate::core::types::CardId,
    slot: Slot,
    targeting: &TargetingRule,
    effects: &[EffectDefinition],
    conditional_effects: &[ConditionalEffectGroup],
) -> Result<(), String> {
    let current_player = ctx.state.active_player;

    // Resolve the spell target based on targeting rule and slot parameter
    let target = resolve_spell_target(targeting, slot, current_player)?;
    let source = EffectSource::Card(card_id);

    // Queue all spell effects with the resolved target
    for effect_def in effects {
        if let Some(effect) = effect_def_to_effect_with_target(
            effect_def,
            target,
            current_player,
        ) {
            ctx.effect_queue.push(effect, source);
        }
    }

    // If spell has conditional effects, process primary effects first and check conditions
    if !conditional_effects.is_empty() {
        // Reset accumulated result before processing
        ctx.effect_queue.reset_accumulated_result();

        // Process primary effects
        ctx.process_effects();

        // Check conditions and queue bonus effects (clone to avoid borrow issues)
        let result = ctx.effect_queue.accumulated_result().clone();
        for cond_group in conditional_effects {
            if result.check(&cond_group.condition) {
                for effect_def in &cond_group.effects {
                    if let Some(effect) = effect_def_to_effect_with_target(
                        effect_def,
                        target,
                        current_player,
                    ) {
                        ctx.effect_queue.push(effect, source);
                    }
                }
            }
        }
    }

    // Process any remaining effects (or all effects if no conditional)
    ctx.process_effects();

    Ok(())
}

/// Play a support card.
fn play_support(
    ctx: &mut ActionContext,
    card_id: crate::core::types::CardId,
    slot: Slot,
    durability: u8,
    triggered_effects: &[AbilityDefinition],
) -> Result<(), String> {
    let current_player = ctx.state.active_player;

    // Create support instance
    let support = Support {
        card_id,
        owner: current_player,
        slot,
        current_durability: durability,
    };

    // Add support to board
    ctx.state.players[current_player.index()].supports.push(support);

    // Apply passive effects from this support to all existing creatures
    apply_support_passives_to_all_creatures(
        card_id,
        &mut ctx.state.players[current_player.index()].creatures,
        ctx.card_db,
    );

    // Queue OnPlay triggered effects for support
    for ability in triggered_effects {
        if ability.trigger == Trigger::OnPlay {
            let source = EffectSource::Support { owner: current_player, slot };
            for effect_def in &ability.effects {
                // Use support-specific effect conversion
                if let Some(effect) = support_effect_def_to_effect(
                    effect_def,
                    current_player,
                    ability,
                ) {
                    ctx.effect_queue.push(effect, source);
                }
            }
        }
    }

    // Process all queued effects
    ctx.process_effects();

    Ok(())
}
