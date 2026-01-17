//! Effect queue for processing game effects in FIFO order.
//!
//! When an effect triggers another effect, the new effect goes to the back
//! of the queue. This avoids recursion and makes resolution predictable.

use std::collections::VecDeque;
use crate::core::cards::{CardDatabase, CardType, EffectDefinition};
use crate::core::effects::{CreatureFilter, Effect, EffectResult, EffectSource, EffectTarget, PendingEffect, TokenDefinition, Trigger};
use crate::core::keywords::Keywords;
use crate::core::state::{CardInstance, Creature, GameResult, GameState, WinReason};
use crate::core::tracing::EffectTracer;
use crate::core::types::{CardId, PlayerId, Slot};

/// Effect queue for processing game effects in FIFO order.
///
/// When an effect triggers another effect, the new effect goes to the back
/// of the queue. This avoids recursion and makes resolution predictable.
#[derive(Debug, Default)]
pub struct EffectQueue {
    queue: VecDeque<PendingEffect>,
    /// Creatures marked for death (processed after each effect)
    pending_deaths: Vec<(PlayerId, Slot)>,
    /// Accumulated result from effect resolution (for conditional triggers)
    accumulated_result: EffectResult,
}

impl EffectQueue {
    /// Create a new empty effect queue
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
            pending_deaths: Vec::new(),
            accumulated_result: EffectResult::none(),
        }
    }

    /// Reset the accumulated result (call before processing a new spell/ability)
    pub fn reset_accumulated_result(&mut self) {
        self.accumulated_result = EffectResult::none();
    }

    /// Get the accumulated result from effect resolution
    pub fn accumulated_result(&self) -> &EffectResult {
        &self.accumulated_result
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
        self.process_all_with_tracer(state, card_db, None);
    }

    /// Process all effects in the queue with optional tracer
    pub fn process_all_with_tracer(
        &mut self,
        state: &mut GameState,
        card_db: &CardDatabase,
        mut tracer: Option<&mut EffectTracer>,
    ) {
        let initial_queue_size = self.queue.len();
        let mut effects_processed = 0;

        // Log queue start if tracer enabled
        if let Some(ref mut t) = tracer {
            t.log_queue_start(initial_queue_size);
        }

        while let Some(pending) = self.queue.pop_front() {
            // Log effect start
            if let Some(ref mut t) = tracer {
                t.log_effect_start(&pending.effect, self.queue.len());
            }

            // Clone effect for logging completion
            let effect_for_log = pending.effect.clone();

            self.resolve_effect(pending, state, card_db);
            effects_processed += 1;

            // Log effect complete
            if let Some(ref mut t) = tracer {
                t.log_effect_complete(&effect_for_log, vec![]);
            }

            // Process any pending deaths after each effect
            self.process_deaths(state, card_db);

            // Check for game over conditions
            if state.is_terminal() {
                // Clear remaining effects if game is over
                self.queue.clear();
                break;
            }
        }

        // Log queue complete
        if let Some(t) = tracer {
            t.log_queue_complete(effects_processed);
        }
    }

    /// Check if a creature has Ward and consume it. Returns true if Ward was consumed (effect should be blocked).
    /// Ward only blocks single-target effects, not AoE effects.
    fn check_and_consume_ward(state: &mut GameState, owner: PlayerId, slot: Slot) -> bool {
        if let Some(creature) = state.players[owner.index()].get_creature_mut(slot) {
            if creature.keywords.has_ward() {
                creature.keywords.remove(Keywords::WARD);
                return true; // Ward consumed, block the effect
            }
        }
        false
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
            Effect::Damage { target, amount, ref filter } => {
                // Ward blocks single-target damage
                if let EffectTarget::Creature { owner, slot } = target {
                    if Self::check_and_consume_ward(state, owner, slot) {
                        return; // Effect blocked by Ward
                    }
                }
                self.apply_damage(target, amount, filter.as_ref(), source_player, state, card_db);
            }
            Effect::Heal { target, amount, ref filter } => {
                // Ward blocks single-target heal (consistent with other targeted effects)
                if let EffectTarget::Creature { owner, slot } = target {
                    if Self::check_and_consume_ward(state, owner, slot) {
                        return;
                    }
                }
                self.apply_heal(target, amount, filter.as_ref(), state, card_db);
            }
            Effect::Draw { player, count } => {
                self.apply_draw(player, count, state);
            }
            Effect::BuffStats { target, attack, health, ref filter } => {
                // Ward blocks single-target buffs
                if let EffectTarget::Creature { owner, slot } = target {
                    if Self::check_and_consume_ward(state, owner, slot) {
                        return;
                    }
                }
                self.apply_buff(target, attack, health, filter.as_ref(), state);
            }
            Effect::SetStats { target, attack, health } => {
                // Ward blocks single-target set stats
                if let EffectTarget::Creature { owner, slot } = target {
                    if Self::check_and_consume_ward(state, owner, slot) {
                        return;
                    }
                }
                self.apply_set_stats(target, attack, health, state);
            }
            Effect::Destroy { target, ref filter } => {
                // Ward blocks single-target destroy
                if let EffectTarget::Creature { owner, slot } = target {
                    if Self::check_and_consume_ward(state, owner, slot) {
                        return;
                    }
                }
                self.apply_destroy(target, filter.as_ref(), state);
            }
            Effect::Summon { owner, card_id, slot } => {
                self.apply_summon(owner, card_id, slot, state, card_db);
            }
            Effect::GrantKeyword { target, keyword, ref filter } => {
                // Ward blocks single-target grant keyword
                if let EffectTarget::Creature { owner, slot } = target {
                    if Self::check_and_consume_ward(state, owner, slot) {
                        return;
                    }
                }
                self.apply_grant_keyword(target, keyword, filter.as_ref(), source_player, state);
            }
            Effect::RemoveKeyword { target, keyword, ref filter } => {
                // Ward blocks single-target remove keyword
                if let EffectTarget::Creature { owner, slot } = target {
                    if Self::check_and_consume_ward(state, owner, slot) {
                        return;
                    }
                }
                self.apply_remove_keyword(target, keyword, filter.as_ref(), state);
            }
            Effect::Silence { target, ref filter } => {
                // Ward blocks single-target silence
                if let EffectTarget::Creature { owner, slot } = target {
                    if Self::check_and_consume_ward(state, owner, slot) {
                        return;
                    }
                }
                self.apply_silence(target, filter.as_ref(), state);
            }
            Effect::GainEssence { player, amount } => {
                self.apply_gain_essence(player, amount, state);
            }
            Effect::RefreshCreature { target } => {
                self.apply_refresh_creature(target, state);
            }
            Effect::Bounce { target, ref filter } => {
                // Ward blocks single-target bounce
                if let EffectTarget::Creature { owner, slot } = target {
                    if Self::check_and_consume_ward(state, owner, slot) {
                        return;
                    }
                }
                self.apply_bounce(target, filter.as_ref(), state);
            }
            Effect::SummonToken { owner, ref token, slot } => {
                self.apply_summon_token(owner, token, slot, state);
            }
            Effect::Transform { target, ref into } => {
                // Ward blocks single-target transform
                if let EffectTarget::Creature { owner, slot } = target {
                    if Self::check_and_consume_ward(state, owner, slot) {
                        return;
                    }
                }
                self.apply_transform(target, into, state);
            }
            Effect::Copy { target, owner } => {
                self.apply_copy(target, owner, state);
            }
        }
    }

    /// Apply damage to a target
    fn apply_damage(
        &mut self,
        target: EffectTarget,
        amount: u8,
        filter: Option<&CreatureFilter>,
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
                        p.creatures.iter()
                            .filter(|c| filter.is_none_or(|f| f.matches(c.current_health, c.keywords.0)))
                            .map(move |c| (owner, c.slot))
                    })
                    .collect();

                for (owner, slot) in creatures {
                    self.damage_creature(owner, slot, amount, state, card_db);
                }
            }
            EffectTarget::AllAllyCreatures(player) => {
                let creatures: Vec<_> = state.players[player.index()]
                    .creatures.iter()
                    .filter(|c| filter.is_none_or(|f| f.matches(c.current_health, c.keywords.0)))
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
                    .filter(|c| filter.is_none_or(|f| f.matches(c.current_health, c.keywords.0)))
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
        // First check if creature exists and has shield/fortify, get necessary info
        let creature_info = {
            let creature = match state.players[owner.index()].get_creature(slot) {
                Some(c) => c,
                None => return,
            };
            (creature.keywords.has_shield(), creature.keywords.has_fortify(), creature.current_health)
        };

        let (has_shield, has_fortify, old_health) = creature_info;

        if has_shield {
            // Shield absorbs the damage, remove shield
            if let Some(creature) = state.players[owner.index()].get_creature_mut(slot) {
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
            let creature = state.players[owner.index()].get_creature_mut(slot).unwrap();
            creature.current_health = (creature.current_health - actual_amount as i8).max(0);
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
        if new_health == 0 {
            // Mark for death processing
            if !self.pending_deaths.contains(&(owner, slot)) {
                self.pending_deaths.push((owner, slot));
                // Track that a target died for conditional triggers
                self.accumulated_result.target_died = true;
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
        filter: Option<&CreatureFilter>,
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
                        p.creatures.iter()
                            .filter(|c| filter.is_none_or(|f| f.matches(c.current_health, c.keywords.0)))
                            .map(move |c| (owner, c.slot))
                    })
                    .collect();

                for (owner, slot) in creatures {
                    self.heal_creature(owner, slot, amount, state, card_db);
                }
            }
            EffectTarget::AllAllyCreatures(player) => {
                let creatures: Vec<_> = state.players[player.index()]
                    .creatures.iter()
                    .filter(|c| filter.is_none_or(|f| f.matches(c.current_health, c.keywords.0)))
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
                    .filter(|c| filter.is_none_or(|f| f.matches(c.current_health, c.keywords.0)))
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
            // Heal up to max health (use i16 to avoid overflow)
            // Cap at 0 minimum in case max_health went negative from passive removal
            creature.current_health = ((creature.current_health as i16) + (amount as i16))
                .min(creature.max_health as i16)
                .max(0) as i8;
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
        filter: Option<&CreatureFilter>,
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
                        p.creatures.iter()
                            .filter(|c| filter.is_none_or(|f| f.matches(c.current_health, c.keywords.0)))
                            .map(move |c| (owner, c.slot))
                    })
                    .collect();

                for (owner, slot) in creatures {
                    self.buff_creature(owner, slot, attack, health, state);
                }
            }
            EffectTarget::AllAllyCreatures(player) => {
                let creatures: Vec<_> = state.players[player.index()]
                    .creatures.iter()
                    .filter(|c| filter.is_none_or(|f| f.matches(c.current_health, c.keywords.0)))
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
                    .filter(|c| filter.is_none_or(|f| f.matches(c.current_health, c.keywords.0)))
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
            // Use saturating arithmetic to prevent overflow
            creature.attack = creature.attack.saturating_add(attack);
            // Cap health at 0 to prevent negative values (debuffs can reduce health)
            creature.current_health = creature.current_health.saturating_add(health).max(0);
            if health > 0 {
                creature.max_health = creature.max_health.saturating_add(health);
            }

            // Check for death from negative health buff
            if creature.current_health == 0
                && !self.pending_deaths.contains(&(owner, slot)) {
                self.pending_deaths.push((owner, slot));
                // Track that a target died for conditional triggers
                self.accumulated_result.target_died = true;
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
                if creature.current_health <= 0
                    && !self.pending_deaths.contains(&(owner, slot)) {
                    self.pending_deaths.push((owner, slot));
                    // Track that a target died for conditional triggers
                    self.accumulated_result.target_died = true;
                }
            }
        }
    }

    /// Destroy a target
    fn apply_destroy(
        &mut self,
        target: EffectTarget,
        filter: Option<&CreatureFilter>,
        state: &mut GameState,
    ) {
        match target {
            EffectTarget::Creature { owner, slot } => {
                if state.players[owner.index()].get_creature(slot).is_some()
                    && !self.pending_deaths.contains(&(owner, slot)) {
                    self.pending_deaths.push((owner, slot));
                    // Track that a target died for conditional triggers
                    self.accumulated_result.target_died = true;
                }
            }
            EffectTarget::AllCreatures => {
                let creatures: Vec<_> = state.players.iter()
                    .enumerate()
                    .flat_map(|(i, p)| {
                        let owner = PlayerId(i as u8);
                        p.creatures.iter()
                            .filter(|c| filter.is_none_or(|f| f.matches(c.current_health, c.keywords.0)))
                            .map(move |c| (owner, c.slot))
                    })
                    .collect();

                for (owner, slot) in creatures {
                    if !self.pending_deaths.contains(&(owner, slot)) {
                        self.pending_deaths.push((owner, slot));
                        // Track that a target died for conditional triggers
                        self.accumulated_result.target_died = true;
                    }
                }
            }
            EffectTarget::AllAllyCreatures(player) => {
                let creatures: Vec<_> = state.players[player.index()]
                    .creatures.iter()
                    .filter(|c| filter.is_none_or(|f| f.matches(c.current_health, c.keywords.0)))
                    .map(|c| c.slot)
                    .collect();

                for slot in creatures {
                    if !self.pending_deaths.contains(&(player, slot)) {
                        self.pending_deaths.push((player, slot));
                        // Track that a target died for conditional triggers
                        self.accumulated_result.target_died = true;
                    }
                }
            }
            EffectTarget::AllEnemyCreatures(player) => {
                let enemy = player.opponent();
                let creatures: Vec<_> = state.players[enemy.index()]
                    .creatures.iter()
                    .filter(|c| filter.is_none_or(|f| f.matches(c.current_health, c.keywords.0)))
                    .map(|c| c.slot)
                    .collect();

                for slot in creatures {
                    if !self.pending_deaths.contains(&(enemy, slot)) {
                        self.pending_deaths.push((enemy, slot));
                        // Track that a target died for conditional triggers
                        self.accumulated_result.target_died = true;
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
            status: Default::default(),
            turn_played: state.current_turn,
            frenzy_stacks: 0,
        };

        state.players[owner.index()].creatures.push(creature);

        // Check if the summoned creature has <= 0 health (can happen with auras/debuffs)
        if let Some(creature) = state.players[owner.index()].get_creature(target_slot) {
            if creature.current_health <= 0
                && !self.pending_deaths.contains(&(owner, target_slot)) {
                self.pending_deaths.push((owner, target_slot));
                self.accumulated_result.target_died = true;
            }
        }

        // Queue OnPlay triggers
        self.check_creature_triggers(Trigger::OnPlay, owner, target_slot, state, card_db);
    }

    /// Summon a token creature (not from card database)
    fn apply_summon_token(
        &mut self,
        owner: PlayerId,
        token: &TokenDefinition,
        slot: Option<Slot>,
        state: &mut GameState,
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

        // Create token creature instance
        // Use CardId(0) as a sentinel value for tokens (not a real card)
        let instance_id = state.next_creature_instance_id();
        let keywords = Keywords(token.keywords);

        let creature = Creature {
            instance_id,
            card_id: CardId(0), // Token marker
            owner,
            slot: target_slot,
            attack: token.attack as i8,
            current_health: token.health as i8,
            max_health: token.health as i8,
            base_attack: token.attack,
            base_health: token.health,
            keywords,
            status: Default::default(),
            turn_played: state.current_turn,
            frenzy_stacks: 0,
        };

        state.players[owner.index()].creatures.push(creature);
        
        // Check if the summoned token has <= 0 health (edge case for 0-health tokens)
        if let Some(creature) = state.players[owner.index()].get_creature(target_slot) {
            if creature.current_health <= 0
                && !self.pending_deaths.contains(&(owner, target_slot)) {
                self.pending_deaths.push((owner, target_slot));
                self.accumulated_result.target_died = true;
            }
        }
        
        // Note: Tokens don't trigger OnPlay since they're not "played from hand"
    }

    /// Transform target creature into a token
    fn apply_transform(
        &mut self,
        target: EffectTarget,
        into: &TokenDefinition,
        state: &mut GameState,
    ) {
        let EffectTarget::Creature { owner, slot } = target else {
            return;
        };

        // Remove the existing creature
        let existed = state.players[owner.index()].get_creature(slot).is_some();
        if !existed {
            return;
        }
        state.players[owner.index()].creatures.retain(|c| c.slot != slot);

        // Create the transformed creature in the same slot
        let instance_id = state.next_creature_instance_id();
        let keywords = Keywords(into.keywords);

        let creature = Creature {
            instance_id,
            card_id: CardId(0), // Token marker
            owner,
            slot,
            attack: into.attack as i8,
            current_health: into.health as i8,
            max_health: into.health as i8,
            base_attack: into.attack,
            base_health: into.health,
            keywords,
            status: Default::default(),
            turn_played: state.current_turn,
            frenzy_stacks: 0,
        };

        // Check if the transformed creature has <= 0 health before adding (edge case)
        let is_dead = creature.current_health <= 0;
        
        state.players[owner.index()].creatures.push(creature);
        
        if is_dead && !self.pending_deaths.contains(&(owner, slot)) {
            self.pending_deaths.push((owner, slot));
            self.accumulated_result.target_died = true;
        }
        
        // Note: Transform doesn't trigger OnDeath or OnPlay
    }

    /// Copy target creature to an empty slot
    fn apply_copy(
        &mut self,
        target: EffectTarget,
        copy_owner: PlayerId,
        state: &mut GameState,
    ) {
        let EffectTarget::Creature { owner, slot } = target else {
            return;
        };

        // Get the source creature's stats
        let source = match state.players[owner.index()].get_creature(slot) {
            Some(c) => c,
            None => return,
        };

        // Copy base stats (not current stats)
        let card_id = source.card_id;
        let base_attack = source.base_attack;
        let base_health = source.base_health;
        let keywords = source.keywords;

        // Find empty slot for the copy
        let target_slot = match state.players[copy_owner.index()].find_empty_creature_slot() {
            Some(s) => s,
            None => return, // No empty slot
        };

        // Create the copy
        let instance_id = state.next_creature_instance_id();

        let creature = Creature {
            instance_id,
            card_id,
            owner: copy_owner,
            slot: target_slot,
            attack: base_attack as i8,
            current_health: base_health as i8,
            max_health: base_health as i8,
            base_attack,
            base_health,
            keywords,
            status: Default::default(),
            turn_played: state.current_turn,
            frenzy_stacks: 0,
        };

        state.players[copy_owner.index()].creatures.push(creature);
        
        // Check if the copied creature has <= 0 health (can happen if copying damaged creature)
        if let Some(creature) = state.players[copy_owner.index()].get_creature(target_slot) {
            if creature.current_health <= 0
                && !self.pending_deaths.contains(&(copy_owner, target_slot)) {
                self.pending_deaths.push((copy_owner, target_slot));
                self.accumulated_result.target_died = true;
            }
        }
        
        // Note: Copy doesn't trigger OnPlay since it's not played from hand
    }

    /// Grant a keyword to a target
    fn apply_grant_keyword(
        &mut self,
        target: EffectTarget,
        keyword: u16,
        filter: Option<&CreatureFilter>,
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
                        if filter.is_none_or(|f| f.matches(creature.current_health, creature.keywords.0)) {
                            creature.keywords.add(keyword);
                        }
                    }
                }
            }
            EffectTarget::AllAllyCreatures(player) => {
                for creature in &mut state.players[player.index()].creatures {
                    if filter.is_none_or(|f| f.matches(creature.current_health, creature.keywords.0)) {
                        creature.keywords.add(keyword);
                    }
                }
            }
            EffectTarget::AllEnemyCreatures(player) => {
                let enemy = player.opponent();
                for creature in &mut state.players[enemy.index()].creatures {
                    if filter.is_none_or(|f| f.matches(creature.current_health, creature.keywords.0)) {
                        creature.keywords.add(keyword);
                    }
                }
            }
            _ => {}
        }
    }

    /// Remove a keyword from a target
    fn apply_remove_keyword(
        &mut self,
        target: EffectTarget,
        keyword: u16,
        filter: Option<&CreatureFilter>,
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
                        if filter.is_none_or(|f| f.matches(creature.current_health, creature.keywords.0)) {
                            creature.keywords.remove(keyword);
                        }
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
        _filter: Option<&CreatureFilter>,
        state: &mut GameState,
    ) {
        // Note: filter not used for single-target silence, but accepted for API consistency
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

    /// Return a creature to its owner's hand
    fn apply_bounce(
        &mut self,
        target: EffectTarget,
        filter: Option<&CreatureFilter>,
        state: &mut GameState,
    ) {
        match target {
            EffectTarget::Creature { owner, slot } => {
                self.bounce_creature(owner, slot, filter, state);
            }
            EffectTarget::AllEnemyCreatures(player) => {
                let enemy = player.opponent();
                let creatures: Vec<_> = state.players[enemy.index()]
                    .creatures
                    .iter()
                    .filter(|c| filter.is_none_or(|f| f.matches(c.current_health, c.keywords.0)))
                    .map(|c| c.slot)
                    .collect();
                for slot in creatures {
                    self.bounce_creature(enemy, slot, None, state);
                }
            }
            EffectTarget::AllAllyCreatures(player) => {
                let creatures: Vec<_> = state.players[player.index()]
                    .creatures
                    .iter()
                    .filter(|c| filter.is_none_or(|f| f.matches(c.current_health, c.keywords.0)))
                    .map(|c| c.slot)
                    .collect();
                for slot in creatures {
                    self.bounce_creature(player, slot, None, state);
                }
            }
            EffectTarget::AllCreatures => {
                for player in [PlayerId::PLAYER_ONE, PlayerId::PLAYER_TWO] {
                    let creatures: Vec<_> = state.players[player.index()]
                        .creatures
                        .iter()
                        .filter(|c| filter.is_none_or(|f| f.matches(c.current_health, c.keywords.0)))
                        .map(|c| c.slot)
                        .collect();
                    for slot in creatures {
                        self.bounce_creature(player, slot, None, state);
                    }
                }
            }
            _ => {} // Player targets don't make sense for bounce
        }
    }

    /// Bounce a single creature (return to owner's hand)
    fn bounce_creature(
        &mut self,
        owner: PlayerId,
        slot: Slot,
        filter: Option<&CreatureFilter>,
        state: &mut GameState,
    ) {
        let player_state = &mut state.players[owner.index()];
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
            EffectDefinition::Damage { amount, filter } => {
                // Default to targeting all enemy creatures for triggered abilities
                Some(Effect::Damage {
                    target: EffectTarget::AllEnemyCreatures(source_owner),
                    amount: *amount,
                    filter: filter.clone(),
                })
            }
            EffectDefinition::Heal { amount, filter } => {
                Some(Effect::Heal {
                    target: EffectTarget::Creature { owner: source_owner, slot: source_slot },
                    amount: *amount,
                    filter: filter.clone(),
                })
            }
            EffectDefinition::Draw { count } => {
                Some(Effect::Draw {
                    player: source_owner,
                    count: *count,
                })
            }
            EffectDefinition::BuffStats { attack, health, filter } => {
                Some(Effect::BuffStats {
                    target: EffectTarget::Creature { owner: source_owner, slot: source_slot },
                    attack: *attack,
                    health: *health,
                    filter: filter.clone(),
                })
            }
            EffectDefinition::Destroy { filter: _ } => {
                // Would need targeting info from ability definition
                None
            }
            EffectDefinition::GrantKeyword { keyword, filter } => {
                let kw = Keywords::from_names(&[keyword.as_str()]);
                Some(Effect::GrantKeyword {
                    target: EffectTarget::Creature { owner: source_owner, slot: source_slot },
                    keyword: kw.0,
                    filter: filter.clone(),
                })
            }
            EffectDefinition::RemoveKeyword { keyword, filter } => {
                let kw = Keywords::from_names(&[keyword.as_str()]);
                Some(Effect::RemoveKeyword {
                    target: EffectTarget::Creature { owner: source_owner, slot: source_slot },
                    keyword: kw.0,
                    filter: filter.clone(),
                })
            }
            EffectDefinition::Silence { filter } => {
                Some(Effect::Silence {
                    target: EffectTarget::Creature { owner: source_owner, slot: source_slot },
                    filter: filter.clone(),
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
            EffectDefinition::Bounce { filter } => {
                // For triggered abilities, bounce targets enemy creatures
                Some(Effect::Bounce {
                    target: EffectTarget::AllEnemyCreatures(source_owner),
                    filter: filter.clone(),
                })
            }
            EffectDefinition::SummonToken { token } => {
                Some(Effect::SummonToken {
                    owner: source_owner,
                    token: token.to_token_definition(),
                    slot: None,
                })
            }
            EffectDefinition::Transform { .. } => {
                // Transform needs specific targeting - not supported in this context
                None
            }
            EffectDefinition::Copy => {
                // Copy needs specific targeting - not supported in this context
                None
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
