//! Module for the main game engine logic.
//!
//! The engine orchestrates game flow, processes actions, and manages
//! the game loop including turn structure and priority.
//!
//! This module also includes the Effect Queue System for processing game effects
//! without recursion, enabling clean triggered ability handling.

use std::collections::VecDeque;
use crate::actions::Action;
use crate::cards::{CardDatabase, CardType, EffectDefinition, AbilityDefinition};
use crate::effects::{Effect, EffectTarget, EffectSource, PendingEffect, Trigger, TargetingRule};
use crate::keywords::Keywords;
use crate::legal::legal_actions;
use crate::state::{CardInstance, Creature, CreatureStatus, GamePhase, GameResult, GameState, Support, WinReason};
use crate::types::{CardId, PlayerId, Slot};

/// Maximum number of turns before the game ends in a tiebreaker
pub const MAX_TURNS: u16 = 30;

/// Starting action points per turn
pub const AP_PER_TURN: u8 = 3;

/// Initial hand size (cards drawn before turn 1)
pub const INITIAL_HAND_SIZE: usize = 3;

// =============================================================================
// EFFECT QUEUE SYSTEM
// =============================================================================

/// Effect queue for processing game effects in FIFO order.
///
/// When an effect triggers another effect, the new effect goes to the back
/// of the queue. This avoids recursion and makes resolution predictable.
#[derive(Debug, Default)]
pub struct EffectQueue {
    queue: VecDeque<PendingEffect>,
    /// Creatures marked for death (processed after each effect)
    pending_deaths: Vec<(PlayerId, Slot)>,
}

impl EffectQueue {
    /// Create a new empty effect queue
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
            pending_deaths: Vec::new(),
        }
    }

    /// Add an effect to the back of the queue
    pub fn push(&mut self, effect: Effect, source: EffectSource) {
        self.queue.push_back(PendingEffect::new(effect, source));
    }

    /// Add a pending effect to the back of the queue
    pub fn push_pending(&mut self, pending: PendingEffect) {
        self.queue.push_back(pending);
    }

    /// Add multiple effects to the queue
    pub fn push_all(&mut self, effects: impl IntoIterator<Item = PendingEffect>) {
        self.queue.extend(effects);
    }

    /// Check if the queue is empty
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    /// Get the number of effects in the queue
    pub fn len(&self) -> usize {
        self.queue.len()
    }

    /// Process all effects in the queue until empty
    pub fn process_all(
        &mut self,
        state: &mut GameState,
        card_db: &CardDatabase,
    ) {
        while let Some(pending) = self.queue.pop_front() {
            self.resolve_effect(pending, state, card_db);

            // Process any pending deaths after each effect
            self.process_deaths(state, card_db);

            // Check for game over conditions
            if state.is_terminal() {
                // Clear remaining effects if game is over
                self.queue.clear();
                return;
            }
        }
    }

    /// Resolve a single effect, potentially queuing more effects
    fn resolve_effect(
        &mut self,
        pending: PendingEffect,
        state: &mut GameState,
        card_db: &CardDatabase,
    ) {
        let source_player = match pending.source {
            EffectSource::Card(_) => state.active_player,
            EffectSource::Creature { owner, .. } => owner,
            EffectSource::Support { owner, .. } => owner,
            EffectSource::System => state.active_player,
        };

        match pending.effect {
            Effect::Damage { target, amount } => {
                self.apply_damage(target, amount, source_player, state, card_db);
            }
            Effect::Heal { target, amount } => {
                self.apply_heal(target, amount, state, card_db);
            }
            Effect::Draw { player, count } => {
                self.apply_draw(player, count, state);
            }
            Effect::BuffStats { target, attack, health } => {
                self.apply_buff(target, attack, health, state);
            }
            Effect::SetStats { target, attack, health } => {
                self.apply_set_stats(target, attack, health, state);
            }
            Effect::Destroy { target } => {
                self.apply_destroy(target, state);
            }
            Effect::Summon { owner, card_id, slot } => {
                self.apply_summon(owner, card_id, slot, state, card_db);
            }
            Effect::GrantKeyword { target, keyword } => {
                self.apply_grant_keyword(target, keyword, source_player, state);
            }
            Effect::RemoveKeyword { target, keyword } => {
                self.apply_remove_keyword(target, keyword, state);
            }
            Effect::Silence { target } => {
                self.apply_silence(target, state);
            }
            Effect::GainEssence { player, amount } => {
                self.apply_gain_essence(player, amount, state);
            }
            Effect::RefreshCreature { target } => {
                self.apply_refresh_creature(target, state);
            }
        }
    }

    /// Apply damage to a target
    fn apply_damage(
        &mut self,
        target: EffectTarget,
        amount: u8,
        _source_player: PlayerId,
        state: &mut GameState,
        card_db: &CardDatabase,
    ) {
        match target {
            EffectTarget::Creature { owner, slot } => {
                self.damage_creature(owner, slot, amount, state, card_db);
            }
            EffectTarget::Player(player) => {
                self.damage_player(player, amount, state);
            }
            EffectTarget::AllCreatures => {
                // Collect all creature positions first to avoid borrow issues
                let creatures: Vec<_> = state.players.iter()
                    .enumerate()
                    .flat_map(|(i, p)| {
                        let owner = PlayerId(i as u8);
                        p.creatures.iter().map(move |c| (owner, c.slot))
                    })
                    .collect();

                for (owner, slot) in creatures {
                    self.damage_creature(owner, slot, amount, state, card_db);
                }
            }
            EffectTarget::AllAllyCreatures(player) => {
                let creatures: Vec<_> = state.players[player.index()]
                    .creatures.iter()
                    .map(|c| c.slot)
                    .collect();

                for slot in creatures {
                    self.damage_creature(player, slot, amount, state, card_db);
                }
            }
            EffectTarget::AllEnemyCreatures(player) => {
                let enemy = player.opponent();
                let creatures: Vec<_> = state.players[enemy.index()]
                    .creatures.iter()
                    .map(|c| c.slot)
                    .collect();

                for slot in creatures {
                    self.damage_creature(enemy, slot, amount, state, card_db);
                }
            }
            EffectTarget::TriggerSource | EffectTarget::None => {
                // No target to damage
            }
        }
    }

    /// Deal damage to a specific creature
    fn damage_creature(
        &mut self,
        owner: PlayerId,
        slot: Slot,
        amount: u8,
        state: &mut GameState,
        card_db: &CardDatabase,
    ) {
        // First check if creature exists and has shield, get necessary info
        let creature_info = {
            let creature = match state.players[owner.index()].get_creature(slot) {
                Some(c) => c,
                None => return,
            };
            (creature.keywords.has_shield(), creature.current_health)
        };

        let (has_shield, old_health) = creature_info;

        if has_shield {
            // Shield absorbs the damage, remove shield
            if let Some(creature) = state.players[owner.index()].get_creature_mut(slot) {
                creature.keywords.remove(Keywords::SHIELD);
            }
            // No damage dealt, no triggers for OnTakeDamage with 0 damage
            return;
        }

        // Apply damage
        let damage_dealt = amount.min(old_health.max(0) as u8);
        let new_health = {
            let creature = state.players[owner.index()].get_creature_mut(slot).unwrap();
            creature.current_health -= amount as i8;
            creature.current_health
        };

        // Queue OnTakeDamage triggers
        if damage_dealt > 0 {
            self.check_creature_triggers(
                Trigger::OnTakeDamage,
                owner,
                slot,
                state,
                card_db,
            );
        }

        // Check for death
        if new_health <= 0 {
            // Mark for death processing
            if !self.pending_deaths.contains(&(owner, slot)) {
                self.pending_deaths.push((owner, slot));
            }
        }
    }

    /// Deal damage to a player
    fn damage_player(&mut self, player: PlayerId, amount: u8, state: &mut GameState) {
        state.players[player.index()].life -= amount as i16;

        // Track total damage dealt
        let opponent = player.opponent();
        state.players[opponent.index()].total_damage_dealt += amount as u16;

        // Check for game over
        if state.players[player.index()].life <= 0 {
            state.result = Some(GameResult::Win {
                winner: player.opponent(),
                reason: WinReason::LifeReachedZero,
            });
        }
    }

    /// Apply healing to a target
    fn apply_heal(
        &mut self,
        target: EffectTarget,
        amount: u8,
        state: &mut GameState,
        card_db: &CardDatabase,
    ) {
        match target {
            EffectTarget::Creature { owner, slot } => {
                self.heal_creature(owner, slot, amount, state, card_db);
            }
            EffectTarget::Player(player) => {
                // Heal player (no max cap in current design per DESIGN.md)
                state.players[player.index()].life += amount as i16;
            }
            EffectTarget::AllCreatures => {
                let creatures: Vec<_> = state.players.iter()
                    .enumerate()
                    .flat_map(|(i, p)| {
                        let owner = PlayerId(i as u8);
                        p.creatures.iter().map(move |c| (owner, c.slot))
                    })
                    .collect();

                for (owner, slot) in creatures {
                    self.heal_creature(owner, slot, amount, state, card_db);
                }
            }
            EffectTarget::AllAllyCreatures(player) => {
                let creatures: Vec<_> = state.players[player.index()]
                    .creatures.iter()
                    .map(|c| c.slot)
                    .collect();

                for slot in creatures {
                    self.heal_creature(player, slot, amount, state, card_db);
                }
            }
            EffectTarget::AllEnemyCreatures(player) => {
                let enemy = player.opponent();
                let creatures: Vec<_> = state.players[enemy.index()]
                    .creatures.iter()
                    .map(|c| c.slot)
                    .collect();

                for slot in creatures {
                    self.heal_creature(enemy, slot, amount, state, card_db);
                }
            }
            EffectTarget::TriggerSource | EffectTarget::None => {
                // No target to heal
            }
        }
    }

    /// Heal a specific creature
    fn heal_creature(
        &mut self,
        owner: PlayerId,
        slot: Slot,
        amount: u8,
        state: &mut GameState,
        _card_db: &CardDatabase,
    ) {
        if let Some(creature) = state.players[owner.index()].get_creature_mut(slot) {
            // Heal up to max health
            creature.current_health = (creature.current_health + amount as i8)
                .min(creature.max_health);
        }
    }

    /// Apply draw effect
    fn apply_draw(&mut self, player: PlayerId, count: u8, state: &mut GameState) {
        let player_state = &mut state.players[player.index()];

        for _ in 0..count {
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
    }

    /// Apply buff/debuff to stats
    fn apply_buff(
        &mut self,
        target: EffectTarget,
        attack: i8,
        health: i8,
        state: &mut GameState,
    ) {
        match target {
            EffectTarget::Creature { owner, slot } => {
                self.buff_creature(owner, slot, attack, health, state);
            }
            EffectTarget::AllCreatures => {
                let creatures: Vec<_> = state.players.iter()
                    .enumerate()
                    .flat_map(|(i, p)| {
                        let owner = PlayerId(i as u8);
                        p.creatures.iter().map(move |c| (owner, c.slot))
                    })
                    .collect();

                for (owner, slot) in creatures {
                    self.buff_creature(owner, slot, attack, health, state);
                }
            }
            EffectTarget::AllAllyCreatures(player) => {
                let creatures: Vec<_> = state.players[player.index()]
                    .creatures.iter()
                    .map(|c| c.slot)
                    .collect();

                for slot in creatures {
                    self.buff_creature(player, slot, attack, health, state);
                }
            }
            EffectTarget::AllEnemyCreatures(player) => {
                let enemy = player.opponent();
                let creatures: Vec<_> = state.players[enemy.index()]
                    .creatures.iter()
                    .map(|c| c.slot)
                    .collect();

                for slot in creatures {
                    self.buff_creature(enemy, slot, attack, health, state);
                }
            }
            _ => {}
        }
    }

    /// Buff a specific creature
    fn buff_creature(
        &mut self,
        owner: PlayerId,
        slot: Slot,
        attack: i8,
        health: i8,
        state: &mut GameState,
    ) {
        if let Some(creature) = state.players[owner.index()].get_creature_mut(slot) {
            creature.attack += attack;
            creature.current_health += health;
            if health > 0 {
                creature.max_health += health;
            }

            // Check for death from negative health buff
            if creature.current_health <= 0 {
                if !self.pending_deaths.contains(&(owner, slot)) {
                    self.pending_deaths.push((owner, slot));
                }
            }
        }
    }

    /// Set creature stats to specific values
    fn apply_set_stats(
        &mut self,
        target: EffectTarget,
        attack: u8,
        health: u8,
        state: &mut GameState,
    ) {
        if let EffectTarget::Creature { owner, slot } = target {
            if let Some(creature) = state.players[owner.index()].get_creature_mut(slot) {
                creature.attack = attack as i8;
                creature.current_health = health as i8;
                creature.max_health = health as i8;

                // Check for death
                if creature.current_health <= 0 {
                    if !self.pending_deaths.contains(&(owner, slot)) {
                        self.pending_deaths.push((owner, slot));
                    }
                }
            }
        }
    }

    /// Destroy a target
    fn apply_destroy(
        &mut self,
        target: EffectTarget,
        state: &mut GameState,
    ) {
        match target {
            EffectTarget::Creature { owner, slot } => {
                if state.players[owner.index()].get_creature(slot).is_some() {
                    if !self.pending_deaths.contains(&(owner, slot)) {
                        self.pending_deaths.push((owner, slot));
                    }
                }
            }
            EffectTarget::AllCreatures => {
                let creatures: Vec<_> = state.players.iter()
                    .enumerate()
                    .flat_map(|(i, p)| {
                        let owner = PlayerId(i as u8);
                        p.creatures.iter().map(move |c| (owner, c.slot))
                    })
                    .collect();

                for (owner, slot) in creatures {
                    if !self.pending_deaths.contains(&(owner, slot)) {
                        self.pending_deaths.push((owner, slot));
                    }
                }
            }
            EffectTarget::AllAllyCreatures(player) => {
                let creatures: Vec<_> = state.players[player.index()]
                    .creatures.iter()
                    .map(|c| c.slot)
                    .collect();

                for slot in creatures {
                    if !self.pending_deaths.contains(&(player, slot)) {
                        self.pending_deaths.push((player, slot));
                    }
                }
            }
            EffectTarget::AllEnemyCreatures(player) => {
                let enemy = player.opponent();
                let creatures: Vec<_> = state.players[enemy.index()]
                    .creatures.iter()
                    .map(|c| c.slot)
                    .collect();

                for slot in creatures {
                    if !self.pending_deaths.contains(&(enemy, slot)) {
                        self.pending_deaths.push((enemy, slot));
                    }
                }
            }
            _ => {}
        }
    }

    /// Summon a creature
    fn apply_summon(
        &mut self,
        owner: PlayerId,
        card_id: CardId,
        slot: Option<Slot>,
        state: &mut GameState,
        card_db: &CardDatabase,
    ) {
        // Find the slot to use
        let target_slot = slot.or_else(|| state.players[owner.index()].find_empty_creature_slot());

        let Some(target_slot) = target_slot else {
            // No empty slot available
            return;
        };

        // Check if slot is already occupied
        if state.players[owner.index()].get_creature(target_slot).is_some() {
            return;
        }

        // Get card definition
        let Some(card_def) = card_db.get(card_id) else {
            return;
        };

        // Must be a creature card
        let CardType::Creature { attack, health, .. } = &card_def.card_type else {
            return;
        };

        // Create creature instance
        let instance_id = state.next_creature_instance_id();
        let keywords = card_def.keywords();

        let creature = Creature {
            instance_id,
            card_id,
            owner,
            slot: target_slot,
            attack: *attack as i8,
            current_health: *health as i8,
            max_health: *health as i8,
            base_attack: *attack,
            base_health: *health,
            keywords,
            status: CreatureStatus::default(),
            turn_played: state.current_turn,
        };

        state.players[owner.index()].creatures.push(creature);

        // Queue OnPlay triggers
        self.check_creature_triggers(Trigger::OnPlay, owner, target_slot, state, card_db);
    }

    /// Grant a keyword to a target
    fn apply_grant_keyword(
        &mut self,
        target: EffectTarget,
        keyword: u8,
        _source_player: PlayerId,
        state: &mut GameState,
    ) {
        match target {
            EffectTarget::Creature { owner, slot } => {
                if let Some(creature) = state.players[owner.index()].get_creature_mut(slot) {
                    creature.keywords.add(keyword);
                }
            }
            EffectTarget::AllCreatures => {
                for player in &mut state.players {
                    for creature in &mut player.creatures {
                        creature.keywords.add(keyword);
                    }
                }
            }
            EffectTarget::AllAllyCreatures(player) => {
                for creature in &mut state.players[player.index()].creatures {
                    creature.keywords.add(keyword);
                }
            }
            EffectTarget::AllEnemyCreatures(player) => {
                let enemy = player.opponent();
                for creature in &mut state.players[enemy.index()].creatures {
                    creature.keywords.add(keyword);
                }
            }
            _ => {}
        }
    }

    /// Remove a keyword from a target
    fn apply_remove_keyword(
        &mut self,
        target: EffectTarget,
        keyword: u8,
        state: &mut GameState,
    ) {
        match target {
            EffectTarget::Creature { owner, slot } => {
                if let Some(creature) = state.players[owner.index()].get_creature_mut(slot) {
                    creature.keywords.remove(keyword);
                }
            }
            EffectTarget::AllCreatures => {
                for player in &mut state.players {
                    for creature in &mut player.creatures {
                        creature.keywords.remove(keyword);
                    }
                }
            }
            _ => {}
        }
    }

    /// Silence a target (remove all keywords and set silenced flag)
    fn apply_silence(
        &mut self,
        target: EffectTarget,
        state: &mut GameState,
    ) {
        if let EffectTarget::Creature { owner, slot } = target {
            if let Some(creature) = state.players[owner.index()].get_creature_mut(slot) {
                creature.keywords.clear();
                creature.status.set_silenced(true);
            }
        }
    }

    /// Gain essence this turn
    fn apply_gain_essence(&mut self, player: PlayerId, amount: u8, state: &mut GameState) {
        state.players[player.index()].current_essence =
            state.players[player.index()].current_essence.saturating_add(amount);
    }

    /// Refresh a creature (remove exhausted status)
    fn apply_refresh_creature(
        &mut self,
        target: EffectTarget,
        state: &mut GameState,
    ) {
        if let EffectTarget::Creature { owner, slot } = target {
            if let Some(creature) = state.players[owner.index()].get_creature_mut(slot) {
                creature.status.set_exhausted(false);
            }
        }
    }

    /// Check for and queue triggered abilities on a creature
    fn check_creature_triggers(
        &mut self,
        trigger: Trigger,
        owner: PlayerId,
        slot: Slot,
        state: &GameState,
        card_db: &CardDatabase,
    ) {
        let Some(creature) = state.players[owner.index()].get_creature(slot) else {
            return;
        };

        // Silenced creatures don't trigger abilities
        if creature.status.is_silenced() {
            return;
        }

        let Some(card_def) = card_db.get(creature.card_id) else {
            return;
        };

        let Some(abilities) = card_def.creature_abilities() else {
            return;
        };

        for ability in abilities {
            if ability.trigger == trigger {
                // Convert ability effects to Effect enum and queue them
                let source = EffectSource::Creature { owner, slot };
                for effect_def in &ability.effects {
                    if let Some(effect) = self.effect_def_to_effect(effect_def, owner, slot) {
                        self.push(effect, source);
                    }
                }
            }
        }
    }

    /// Convert an EffectDefinition to an Effect enum
    fn effect_def_to_effect(
        &self,
        def: &EffectDefinition,
        source_owner: PlayerId,
        source_slot: Slot,
    ) -> Option<Effect> {
        match def {
            EffectDefinition::Damage { amount } => {
                // Default to targeting all enemy creatures for triggered abilities
                Some(Effect::Damage {
                    target: EffectTarget::AllEnemyCreatures(source_owner),
                    amount: *amount,
                })
            }
            EffectDefinition::Heal { amount } => {
                Some(Effect::Heal {
                    target: EffectTarget::Creature { owner: source_owner, slot: source_slot },
                    amount: *amount,
                })
            }
            EffectDefinition::Draw { count } => {
                Some(Effect::Draw {
                    player: source_owner,
                    count: *count,
                })
            }
            EffectDefinition::BuffStats { attack, health } => {
                Some(Effect::BuffStats {
                    target: EffectTarget::Creature { owner: source_owner, slot: source_slot },
                    attack: *attack,
                    health: *health,
                })
            }
            EffectDefinition::Destroy => {
                // Would need targeting info from ability definition
                None
            }
            EffectDefinition::GrantKeyword { keyword } => {
                let kw = Keywords::from_names(&[keyword.as_str()]);
                Some(Effect::GrantKeyword {
                    target: EffectTarget::Creature { owner: source_owner, slot: source_slot },
                    keyword: kw.0,
                })
            }
            EffectDefinition::RemoveKeyword { keyword } => {
                let kw = Keywords::from_names(&[keyword.as_str()]);
                Some(Effect::RemoveKeyword {
                    target: EffectTarget::Creature { owner: source_owner, slot: source_slot },
                    keyword: kw.0,
                })
            }
            EffectDefinition::Silence => {
                Some(Effect::Silence {
                    target: EffectTarget::Creature { owner: source_owner, slot: source_slot },
                })
            }
            EffectDefinition::GainEssence { amount } => {
                Some(Effect::GainEssence {
                    player: source_owner,
                    amount: *amount,
                })
            }
            EffectDefinition::RefreshCreature => {
                Some(Effect::RefreshCreature {
                    target: EffectTarget::Creature { owner: source_owner, slot: source_slot },
                })
            }
        }
    }

    /// Process all pending deaths
    fn process_deaths(&mut self, state: &mut GameState, card_db: &CardDatabase) {
        // Process deaths in the order they occurred
        while let Some((owner, slot)) = self.pending_deaths.pop() {
            // Check creature still exists
            let creature_info = state.players[owner.index()]
                .get_creature(slot)
                .map(|c| (c.card_id, c.status.is_silenced()));

            if let Some((card_id, is_silenced)) = creature_info {
                // Queue OnDeath triggers before removing
                if !is_silenced {
                    if let Some(card_def) = card_db.get(card_id) {
                        if let Some(abilities) = card_def.creature_abilities() {
                            for ability in abilities {
                                if ability.trigger == Trigger::OnDeath {
                                    let source = EffectSource::Creature { owner, slot };
                                    for effect_def in &ability.effects {
                                        if let Some(effect) = self.effect_def_to_effect(
                                            effect_def,
                                            owner,
                                            slot,
                                        ) {
                                            self.push(effect, source);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // Queue OnAllyDeath triggers for other friendly creatures
                let ally_creatures: Vec<Slot> = state.players[owner.index()]
                    .creatures.iter()
                    .filter(|c| c.slot != slot && !c.status.is_silenced())
                    .map(|c| c.slot)
                    .collect();

                for ally_slot in ally_creatures {
                    if let Some(ally) = state.players[owner.index()].get_creature(ally_slot) {
                        if let Some(card_def) = card_db.get(ally.card_id) {
                            if let Some(abilities) = card_def.creature_abilities() {
                                for ability in abilities {
                                    if ability.trigger == Trigger::OnAllyDeath {
                                        let source = EffectSource::Creature {
                                            owner,
                                            slot: ally_slot
                                        };
                                        for effect_def in &ability.effects {
                                            if let Some(effect) = self.effect_def_to_effect(
                                                effect_def,
                                                owner,
                                                ally_slot,
                                            ) {
                                                self.push(effect, source);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // Remove the creature from the board
                state.players[owner.index()].creatures.retain(|c| c.slot != slot);
            }
        }
    }
}

// =============================================================================
// HELPER FUNCTIONS FOR CARD PLAYING
// =============================================================================

/// Resolve the target for a spell based on its targeting rule and the slot parameter.
///
/// Target encoding in slot parameter:
/// - For NoTarget spells: slot is ignored
/// - For creature-targeting spells: slot 0-4 indicates enemy creature slot, slot + 5 for friendly
/// - For TargetPlayer: slot 0 = enemy, slot 1 = self
/// - For TargetAny: slot 0-4 enemy creature, 5-9 friendly creature, 10 enemy player, 11 self
pub fn resolve_spell_target(
    targeting: &TargetingRule,
    slot: Slot,
    caster: PlayerId,
) -> Result<EffectTarget, String> {
    match targeting {
        TargetingRule::NoTarget => {
            Ok(EffectTarget::None)
        }
        TargetingRule::TargetCreature(_) => {
            // slot 0-4 = enemy creature, slot 5-9 = friendly creature
            if slot.0 < 5 {
                Ok(EffectTarget::Creature {
                    owner: caster.opponent(),
                    slot: Slot(slot.0),
                })
            } else if slot.0 < 10 {
                Ok(EffectTarget::Creature {
                    owner: caster,
                    slot: Slot(slot.0 - 5),
                })
            } else {
                Err("Invalid target slot for creature targeting".to_string())
            }
        }
        TargetingRule::TargetEnemyCreature => {
            // slot 0-4 = enemy creature slot
            if slot.0 < 5 {
                Ok(EffectTarget::Creature {
                    owner: caster.opponent(),
                    slot: Slot(slot.0),
                })
            } else {
                Err("Invalid target slot for enemy creature targeting".to_string())
            }
        }
        TargetingRule::TargetAllyCreature => {
            // slot 0-4 = friendly creature slot
            if slot.0 < 5 {
                Ok(EffectTarget::Creature {
                    owner: caster,
                    slot: Slot(slot.0),
                })
            } else {
                Err("Invalid target slot for ally creature targeting".to_string())
            }
        }
        TargetingRule::TargetPlayer => {
            // slot 0 = enemy, slot 1 = self
            if slot.0 == 0 {
                Ok(EffectTarget::Player(caster.opponent()))
            } else {
                Ok(EffectTarget::Player(caster))
            }
        }
        TargetingRule::TargetEnemyPlayer => {
            Ok(EffectTarget::Player(caster.opponent()))
        }
        TargetingRule::TargetAny => {
            // slot 0-4 enemy creature, 5-9 friendly creature, 10 enemy player, 11 self
            if slot.0 < 5 {
                Ok(EffectTarget::Creature {
                    owner: caster.opponent(),
                    slot: Slot(slot.0),
                })
            } else if slot.0 < 10 {
                Ok(EffectTarget::Creature {
                    owner: caster,
                    slot: Slot(slot.0 - 5),
                })
            } else if slot.0 == 10 {
                Ok(EffectTarget::Player(caster.opponent()))
            } else {
                Ok(EffectTarget::Player(caster))
            }
        }
        TargetingRule::TargetSlot => {
            // For summoning effects - just return the slot info
            if slot.0 < 5 {
                Ok(EffectTarget::Creature {
                    owner: caster,
                    slot: Slot(slot.0),
                })
            } else {
                Err("Invalid target slot".to_string())
            }
        }
    }
}

/// Convert an EffectDefinition to an Effect enum with a specific target.
/// Used primarily for spell effects where the target is resolved beforehand.
pub fn effect_def_to_effect_with_target(
    def: &EffectDefinition,
    target: EffectTarget,
    source_player: PlayerId,
) -> Option<Effect> {
    match def {
        EffectDefinition::Damage { amount } => {
            Some(Effect::Damage {
                target,
                amount: *amount,
            })
        }
        EffectDefinition::Heal { amount } => {
            Some(Effect::Heal {
                target,
                amount: *amount,
            })
        }
        EffectDefinition::Draw { count } => {
            // Draw always affects the caster
            Some(Effect::Draw {
                player: source_player,
                count: *count,
            })
        }
        EffectDefinition::BuffStats { attack, health } => {
            Some(Effect::BuffStats {
                target,
                attack: *attack,
                health: *health,
            })
        }
        EffectDefinition::Destroy => {
            Some(Effect::Destroy { target })
        }
        EffectDefinition::GrantKeyword { keyword } => {
            let kw = Keywords::from_names(&[keyword.as_str()]);
            Some(Effect::GrantKeyword {
                target,
                keyword: kw.0,
            })
        }
        EffectDefinition::RemoveKeyword { keyword } => {
            let kw = Keywords::from_names(&[keyword.as_str()]);
            Some(Effect::RemoveKeyword {
                target,
                keyword: kw.0,
            })
        }
        EffectDefinition::Silence => {
            Some(Effect::Silence { target })
        }
        EffectDefinition::GainEssence { amount } => {
            Some(Effect::GainEssence {
                player: source_player,
                amount: *amount,
            })
        }
        EffectDefinition::RefreshCreature => {
            Some(Effect::RefreshCreature { target })
        }
    }
}

/// Convert an EffectDefinition to an Effect for triggered abilities.
/// The target is inferred based on the effect type and trigger context.
pub fn effect_def_to_triggered_effect(
    def: &EffectDefinition,
    source_owner: PlayerId,
    source_slot: Slot,
    ability: &AbilityDefinition,
) -> Option<Effect> {
    // Determine the target based on the ability's targeting rule and effect type
    let default_target = EffectTarget::Creature {
        owner: source_owner,
        slot: source_slot,
    };

    match def {
        EffectDefinition::Damage { amount } => {
            // For damage, check the targeting rule to determine who gets hit
            let target = match &ability.targeting {
                TargetingRule::NoTarget => {
                    // Default to all enemy creatures for AoE damage
                    EffectTarget::AllEnemyCreatures(source_owner)
                }
                TargetingRule::TargetEnemyCreature => {
                    // This should be resolved at cast time, but for triggers,
                    // we default to all enemy creatures
                    EffectTarget::AllEnemyCreatures(source_owner)
                }
                TargetingRule::TargetAllyCreature => {
                    EffectTarget::AllAllyCreatures(source_owner)
                }
                TargetingRule::TargetEnemyPlayer => {
                    EffectTarget::Player(source_owner.opponent())
                }
                _ => EffectTarget::AllEnemyCreatures(source_owner),
            };
            Some(Effect::Damage {
                target,
                amount: *amount,
            })
        }
        EffectDefinition::Heal { amount } => {
            // Heal typically targets self or allies
            let target = match &ability.targeting {
                TargetingRule::TargetAllyCreature | TargetingRule::NoTarget => {
                    default_target
                }
                TargetingRule::TargetPlayer => {
                    EffectTarget::Player(source_owner)
                }
                _ => default_target,
            };
            Some(Effect::Heal {
                target,
                amount: *amount,
            })
        }
        EffectDefinition::Draw { count } => {
            Some(Effect::Draw {
                player: source_owner,
                count: *count,
            })
        }
        EffectDefinition::BuffStats { attack, health } => {
            // Buff typically targets self
            Some(Effect::BuffStats {
                target: default_target,
                attack: *attack,
                health: *health,
            })
        }
        EffectDefinition::Destroy => {
            // Destroy needs a specific target - this should be resolved differently
            // For now, return None as it needs targeting
            None
        }
        EffectDefinition::GrantKeyword { keyword } => {
            let kw = Keywords::from_names(&[keyword.as_str()]);
            Some(Effect::GrantKeyword {
                target: default_target,
                keyword: kw.0,
            })
        }
        EffectDefinition::RemoveKeyword { keyword } => {
            let kw = Keywords::from_names(&[keyword.as_str()]);
            Some(Effect::RemoveKeyword {
                target: default_target,
                keyword: kw.0,
            })
        }
        EffectDefinition::Silence => {
            Some(Effect::Silence {
                target: default_target,
            })
        }
        EffectDefinition::GainEssence { amount } => {
            Some(Effect::GainEssence {
                player: source_owner,
                amount: *amount,
            })
        }
        EffectDefinition::RefreshCreature => {
            Some(Effect::RefreshCreature {
                target: default_target,
            })
        }
    }
}

// =============================================================================
// GAME ENGINE
// =============================================================================

/// Seeded shuffle using Linear Congruential Generator for deterministic results.
/// Uses the same constants as PCG for good statistical properties.
pub fn seeded_shuffle<T>(items: &mut [T], seed: u64) {
    let mut rng = seed;
    for i in (1..items.len()).rev() {
        // LCG: next = (a * current + c) mod m
        rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let j = (rng as usize) % (i + 1);
        items.swap(i, j);
    }
}

/// Game engine that manages turn flow and action execution
pub struct GameEngine<'a> {
    pub state: GameState,
    card_db: &'a CardDatabase,
}

impl<'a> GameEngine<'a> {
    /// Create a new game engine with the given card database
    pub fn new(card_db: &'a CardDatabase) -> Self {
        Self {
            state: GameState::new(),
            card_db,
        }
    }

    /// Initialize a new game with the given decks.
    /// Shuffles decks using the provided seed, draws initial hands,
    /// and starts Player 1's first turn.
    pub fn start_game(&mut self, deck1: Vec<CardId>, deck2: Vec<CardId>, seed: u64) {
        // Reset state
        self.state = GameState::new();
        self.state.rng_state = seed;

        // Set up player 1's deck
        let mut deck1_cards: Vec<CardInstance> = deck1.into_iter().map(CardInstance::new).collect();
        seeded_shuffle(&mut deck1_cards, seed);
        for card in deck1_cards {
            if self.state.players[0].deck.len() < 30 {
                self.state.players[0].deck.push(card);
            }
        }

        // Set up player 2's deck (use a different seed derived from the original)
        let seed2 = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        let mut deck2_cards: Vec<CardInstance> = deck2.into_iter().map(CardInstance::new).collect();
        seeded_shuffle(&mut deck2_cards, seed2);
        for card in deck2_cards {
            if self.state.players[1].deck.len() < 30 {
                self.state.players[1].deck.push(card);
            }
        }

        // Draw initial hands (3 cards each)
        for _ in 0..INITIAL_HAND_SIZE {
            self.draw_card(PlayerId::PLAYER_ONE);
            self.draw_card(PlayerId::PLAYER_TWO);
        }

        // Set up initial game state
        self.state.current_turn = 0; // Will be incremented to 1 in start_turn
        self.state.active_player = PlayerId::PLAYER_ONE;
        self.state.phase = GamePhase::Main;

        // Start Player 1's first turn
        self.start_turn();
    }

    /// Draw a card for the specified player.
    /// If deck is empty, nothing happens (no fatigue damage in this game).
    /// If hand is full (10 cards), the drawn card is discarded (overdraw).
    fn draw_card(&mut self, player: PlayerId) {
        let player_state = &mut self.state.players[player.index()];

        // Try to draw from deck
        if let Some(card) = player_state.deck.pop() {
            // Add to hand if not full
            if !player_state.is_hand_full() {
                player_state.hand.push(card);
            }
            // If hand is full, card is simply discarded (overdraw)
        }
    }

    /// Start the current player's turn.
    /// - Increment turn counter
    /// - Draw a card
    /// - Restore AP to 3
    /// - Reset creature attack flags (clear exhausted status)
    fn start_turn(&mut self) {
        // Increment turn counter
        self.state.current_turn += 1;

        // Check for turn limit win condition
        if self.state.current_turn > MAX_TURNS {
            self.check_turn_limit_victory();
            return;
        }

        let current_player = self.state.active_player;

        // Draw a card
        self.draw_card(current_player);

        // Restore AP to 3
        self.state.players[current_player.index()].action_points = AP_PER_TURN;

        // Reset creature attack flags (clear exhausted status)
        // Creatures that survived a full round can now attack
        for creature in &mut self.state.players[current_player.index()].creatures {
            creature.status.set_exhausted(false);
        }
    }

    /// End the current player's turn.
    /// - Process end-of-turn effects (placeholder for now)
    /// - Switch to other player
    /// - Call start_turn for new player
    fn end_turn(&mut self) {
        // Process end-of-turn effects (placeholder for future implementation)
        // self.process_end_of_turn_effects();

        // Switch to opponent
        self.state.active_player = self.state.active_player.opponent();

        // Start opponent's turn
        self.start_turn();
    }

    /// Check for turn limit victory condition.
    /// Player with higher life wins. On tie, Player 1 wins.
    fn check_turn_limit_victory(&mut self) {
        let p1_life = self.state.players[0].life;
        let p2_life = self.state.players[1].life;

        let winner = if p1_life >= p2_life {
            PlayerId::PLAYER_ONE
        } else {
            PlayerId::PLAYER_TWO
        };

        self.state.result = Some(GameResult::Win {
            winner,
            reason: WinReason::TurnLimitHigherLife,
        });
        self.state.phase = GamePhase::Ended;
    }

    /// Check if a player has lost due to life reaching 0.
    pub fn check_life_victory(&mut self) {
        for player_idx in 0..2 {
            if self.state.players[player_idx].life <= 0 {
                let loser = PlayerId(player_idx as u8);
                let winner = loser.opponent();
                self.state.result = Some(GameResult::Win {
                    winner,
                    reason: WinReason::LifeReachedZero,
                });
                self.state.phase = GamePhase::Ended;
                return;
            }
        }
    }

    /// Apply an action to the game state.
    /// Returns Ok(()) on success, Err with description on illegal action.
    pub fn apply_action(&mut self, action: Action) -> Result<(), String> {
        // Check if game is over
        if self.state.is_terminal() {
            return Err("Game is already over".to_string());
        }

        // Validate action is legal
        let legal = legal_actions(&self.state, self.card_db);
        if !legal.contains(&action) {
            return Err(format!("Illegal action: {:?}", action));
        }

        match action {
            Action::PlayCard { hand_index, slot } => {
                self.execute_play_card(hand_index as usize, slot)?;
            }
            Action::Attack { attacker, defender } => {
                self.execute_attack(attacker, defender)?;
            }
            Action::UseAbility {
                slot,
                ability_index,
                target,
            } => {
                self.execute_use_ability(slot, ability_index, target)?;
            }
            Action::EndTurn => {
                self.end_turn();
            }
        }

        // Check for victory after each action
        self.check_life_victory();

        Ok(())
    }

    /// Execute a PlayCard action.
    ///
    /// Handles creatures, spells, and supports with full effect queue integration:
    /// - Creatures: Place on board with summoning sickness (unless Rush), trigger OnPlay
    /// - Spells: Resolve targeting, queue effects, process queue
    /// - Supports: Place in support slot, trigger OnPlay if present
    pub fn execute_play_card(&mut self, hand_index: usize, slot: Slot) -> Result<(), String> {
        let current_player = self.state.active_player;

        // Get the card from hand
        if hand_index >= self.state.players[current_player.index()].hand.len() {
            return Err("Invalid hand index".to_string());
        }
        let card_instance = self.state.players[current_player.index()].hand.remove(hand_index);
        let card_id = card_instance.card_id;

        // Look up card definition
        let card_def = self
            .card_db
            .get(card_id)
            .ok_or_else(|| "Card not found in database".to_string())?;

        // Deduct AP cost
        let player_state = &mut self.state.players[current_player.index()];
        if card_def.cost > player_state.action_points {
            return Err("Not enough AP".to_string());
        }
        player_state.action_points -= card_def.cost;

        // Create effect queue for triggered effects
        let mut effect_queue = EffectQueue::new();

        match &card_def.card_type {
            CardType::Creature { attack, health, abilities, .. } => {
                // Create creature instance
                let instance_id = self.state.next_creature_instance_id();
                let keywords = card_def.keywords();

                let creature = Creature {
                    instance_id,
                    card_id,
                    owner: current_player,
                    slot,
                    attack: *attack as i8,
                    current_health: *health as i8,
                    max_health: *health as i8,
                    base_attack: *attack,
                    base_health: *health,
                    keywords,
                    status: CreatureStatus::default(),
                    turn_played: self.state.current_turn,
                };

                // Add creature to board
                self.state.players[current_player.index()].creatures.push(creature);

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
                                effect_queue.push(effect, source);
                            }
                        }
                    }
                }

                // Check for OnAllyPlayed triggers on other friendly creatures
                let other_creatures: Vec<(Slot, CardId)> = self.state.players[current_player.index()]
                    .creatures
                    .iter()
                    .filter(|c| c.slot != slot && !c.status.is_silenced())
                    .map(|c| (c.slot, c.card_id))
                    .collect();

                for (ally_slot, ally_card_id) in other_creatures {
                    if let Some(ally_card_def) = self.card_db.get(ally_card_id) {
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
                                            effect_queue.push(effect, source);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            CardType::Spell { targeting, effects } => {
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
                        effect_queue.push(effect, source);
                    }
                }

                // Spell is consumed (already removed from hand, no discard pile tracking)
            }
            CardType::Support { durability, triggered_effects, .. } => {
                // Create support instance
                let support = Support {
                    card_id,
                    owner: current_player,
                    slot,
                    current_durability: *durability,
                };

                // Add support to board
                self.state.players[current_player.index()].supports.push(support);

                // Queue OnPlay triggered effects for support
                for ability in triggered_effects {
                    if ability.trigger == Trigger::OnPlay {
                        let source = EffectSource::Support { owner: current_player, slot };
                        for effect_def in &ability.effects {
                            if let Some(effect) = effect_def_to_triggered_effect(
                                effect_def,
                                current_player,
                                slot,
                                ability,
                            ) {
                                effect_queue.push(effect, source);
                            }
                        }
                    }
                }

                // Note: Passive effects from supports are applied continuously and
                // handled elsewhere (e.g., during stat calculations). They are not
                // queued effects that resolve once.
            }
        }

        // Process all queued effects
        effect_queue.process_all(&mut self.state, self.card_db);

        Ok(())
    }

    /// Execute an Attack action.
    fn execute_attack(&mut self, attacker_slot: Slot, defender_slot: Slot) -> Result<(), String> {
        let current_player = self.state.active_player;
        let opponent = current_player.opponent();

        // Get attacker
        let attacker = self
            .state
            .players[current_player.index()]
            .get_creature(attacker_slot)
            .ok_or_else(|| "No creature at attacker slot".to_string())?;

        // Validate attacker can attack
        if !attacker.can_attack(self.state.current_turn) {
            return Err("Creature cannot attack".to_string());
        }

        let attacker_damage = attacker.attack.max(0) as u8;
        let attacker_has_piercing = attacker.keywords.has_piercing();
        let attacker_has_lifesteal = attacker.keywords.has_lifesteal();
        let attacker_has_lethal = attacker.keywords.has_lethal();

        // Check if there's a defender creature
        let defender_exists = self.state.players[opponent.index()]
            .get_creature(defender_slot)
            .is_some();

        if defender_exists {
            // Combat between creatures
            let defender = self.state.players[opponent.index()]
                .get_creature(defender_slot)
                .unwrap();

            let defender_damage = defender.attack.max(0) as u8;
            let defender_has_lethal = defender.keywords.has_lethal();
            let defender_has_shield = defender.keywords.has_shield();
            let attacker_has_shield = self.state.players[current_player.index()]
                .get_creature(attacker_slot)
                .unwrap()
                .keywords
                .has_shield();

            // Apply damage to defender
            let defender = self.state.players[opponent.index()]
                .get_creature_mut(defender_slot)
                .unwrap();

            if defender_has_shield && attacker_damage > 0 {
                // Shield absorbs damage and is consumed
                defender.keywords.remove(crate::keywords::Keywords::SHIELD);
            } else {
                defender.current_health -= attacker_damage as i8;
                // Apply lethal keyword
                if attacker_has_lethal && attacker_damage > 0 {
                    defender.current_health = 0;
                }
            }

            // Apply damage to attacker (from defender)
            let attacker = self.state.players[current_player.index()]
                .get_creature_mut(attacker_slot)
                .unwrap();

            if attacker_has_shield && defender_damage > 0 {
                // Shield absorbs damage and is consumed
                attacker.keywords.remove(crate::keywords::Keywords::SHIELD);
            } else {
                attacker.current_health -= defender_damage as i8;
                // Apply lethal keyword
                if defender_has_lethal && defender_damage > 0 {
                    attacker.current_health = 0;
                }
            }

            // Mark attacker as exhausted
            attacker.status.set_exhausted(true);

            // Piercing: excess damage goes to opponent face
            if attacker_has_piercing {
                let defender = self.state.players[opponent.index()]
                    .get_creature(defender_slot)
                    .unwrap();
                if defender.current_health < 0 {
                    let excess = (-defender.current_health) as i16;
                    self.state.players[opponent.index()].life -= excess;
                }
            }

            // Lifesteal: heal attacker's owner
            if attacker_has_lifesteal && attacker_damage > 0 {
                let heal_amount = attacker_damage as i16;
                self.state.players[current_player.index()].life =
                    (self.state.players[current_player.index()].life + heal_amount).min(30);
            }

            // Track damage dealt
            self.state.players[current_player.index()].total_damage_dealt += attacker_damage as u16;

            // Remove dead creatures
            self.remove_dead_creatures();
        } else {
            // Direct attack to opponent's face
            let damage = attacker_damage as i16;
            self.state.players[opponent.index()].life -= damage;

            // Mark attacker as exhausted
            let attacker = self.state.players[current_player.index()]
                .get_creature_mut(attacker_slot)
                .unwrap();
            attacker.status.set_exhausted(true);

            // Lifesteal: heal attacker's owner
            if attacker_has_lifesteal && damage > 0 {
                self.state.players[current_player.index()].life =
                    (self.state.players[current_player.index()].life + damage).min(30);
            }

            // Track damage dealt
            self.state.players[current_player.index()].total_damage_dealt += damage as u16;
        }

        Ok(())
    }

    /// Execute a UseAbility action.
    fn execute_use_ability(
        &mut self,
        _slot: Slot,
        _ability_index: u8,
        _target: crate::actions::Target,
    ) -> Result<(), String> {
        // TODO: Implement ability execution
        // This is a placeholder for future implementation
        Err("Abilities not yet implemented".to_string())
    }

    /// Remove all dead creatures from the board.
    fn remove_dead_creatures(&mut self) {
        for player_idx in 0..2 {
            self.state.players[player_idx]
                .creatures
                .retain(|c| c.is_alive());
        }
    }

    /// Check if game is over.
    pub fn is_terminal(&self) -> bool {
        self.state.is_terminal()
    }

    /// Get winner (None if game not over, Some(player) if won).
    pub fn winner(&self) -> Option<PlayerId> {
        match &self.state.result {
            Some(GameResult::Win { winner, .. }) => Some(*winner),
            Some(GameResult::Draw) => None,
            None => None,
        }
    }

    /// Get the card database reference.
    pub fn card_db(&self) -> &CardDatabase {
        self.card_db
    }

    // =========================================================================
    // GAME INTERFACE METHODS (Task 6.2)
    // =========================================================================
    // These methods provide a clean interface for AI/external code to interact
    // with the game, particularly useful for MCTS and neural network training.

    /// Get the current state as a tensor for neural network input.
    ///
    /// Returns a fixed-size array of f32 values encoding all relevant game state.
    pub fn get_state_tensor(&self) -> [f32; crate::tensor::STATE_TENSOR_SIZE] {
        crate::tensor::state_to_tensor(&self.state)
    }

    /// Get legal action mask (256 bools as f32).
    ///
    /// Returns 1.0 for legal actions and 0.0 for illegal actions.
    /// This is useful for masking neural network outputs.
    pub fn get_legal_action_mask(&self) -> [f32; 256] {
        crate::tensor::legal_mask_to_tensor(&crate::legal::legal_action_mask(&self.state, self.card_db))
    }

    /// Get list of legal actions.
    ///
    /// Returns a Vec of all currently legal actions for the active player.
    pub fn get_legal_actions(&self) -> Vec<Action> {
        legal_actions(&self.state, self.card_db).to_vec()
    }

    /// Apply an action by index (from neural network output).
    ///
    /// Converts the neural network action index (0-255) to an Action and applies it.
    /// Returns an error if the index is invalid or the action is illegal.
    pub fn apply_action_by_index(&mut self, index: u8) -> Result<(), String> {
        let action = Action::from_index(index)
            .ok_or_else(|| format!("Invalid action index: {}", index))?;
        self.apply_action(action)
    }

    /// Check if game is in terminal state.
    ///
    /// This is an alias for is_terminal() for interface consistency.
    pub fn is_game_over(&self) -> bool {
        self.is_terminal()
    }

    /// Get reward for the specified player.
    ///
    /// Returns:
    /// - 1.0 for win
    /// - -1.0 for loss
    /// - 0.0 for ongoing/draw
    pub fn get_reward(&self, player: PlayerId) -> f32 {
        match self.winner() {
            None => 0.0,
            Some(winner) if winner == player => 1.0,
            Some(_) => -1.0,
        }
    }

    /// Clone the game state for MCTS tree search.
    ///
    /// This is efficient because GameState is designed for fast cloning
    /// (uses ArrayVec for stack allocation, no heap allocations for most data).
    pub fn clone_state(&self) -> GameState {
        self.state.clone()
    }

    /// Create a new engine with a cloned state (for MCTS).
    ///
    /// This creates an independent copy of the game that can be modified
    /// without affecting the original. Useful for tree search algorithms.
    pub fn fork(&self) -> GameEngine<'a> {
        GameEngine {
            state: self.state.clone(),
            card_db: self.card_db,
        }
    }

    /// Get current player.
    pub fn current_player(&self) -> PlayerId {
        self.state.active_player
    }

    /// Get turn number.
    pub fn turn_number(&self) -> u16 {
        self.state.current_turn
    }
}

// =============================================================================
// GAME ENVIRONMENT TRAIT (Task 6.2)
// =============================================================================

/// Trait for game environments (useful for generic AI implementations).
///
/// This trait provides a generic interface that AI algorithms (like MCTS or
/// reinforcement learning) can use to interact with any game, not just this
/// card game. This enables code reuse across different game implementations.
pub trait GameEnvironment {
    /// The state type for this game
    type State;
    /// The action type for this game
    type Action;

    /// Get a reference to the current state
    fn get_state(&self) -> &Self::State;

    /// Get all legal actions for the current state
    fn get_legal_actions(&self) -> Vec<Self::Action>;

    /// Apply an action to the current state
    fn apply_action(&mut self, action: Self::Action) -> Result<(), String>;

    /// Check if the game is in a terminal state
    fn is_terminal(&self) -> bool;

    /// Get the reward for a specific player
    fn get_reward(&self, player: u8) -> f32;

    /// Clone the environment for search (MCTS tree expansion)
    fn clone_for_search(&self) -> Self where Self: Sized;
}

impl<'a> GameEnvironment for GameEngine<'a> {
    type State = GameState;
    type Action = Action;

    fn get_state(&self) -> &Self::State {
        &self.state
    }

    fn get_legal_actions(&self) -> Vec<Self::Action> {
        legal_actions(&self.state, self.card_db).to_vec()
    }

    fn apply_action(&mut self, action: Self::Action) -> Result<(), String> {
        GameEngine::apply_action(self, action)
    }

    fn is_terminal(&self) -> bool {
        self.is_game_over()
    }

    fn get_reward(&self, player: u8) -> f32 {
        GameEngine::get_reward(self, PlayerId(player))
    }

    fn clone_for_search(&self) -> Self {
        self.fork()
    }
}
