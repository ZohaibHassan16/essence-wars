//! Effect queue for processing game effects in FIFO order.
//!
//! When an effect triggers another effect, the new effect goes to the back
//! of the queue. This avoids recursion and makes resolution predictable.

use std::collections::VecDeque;
use crate::core::cards::{CardDatabase, CardType, EffectDefinition};
use crate::core::effects::{Effect, EffectSource, EffectTarget, PendingEffect, Trigger};
use crate::core::keywords::Keywords;
use crate::core::state::{Creature, GameResult, GameState, WinReason};
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
            // Heal up to max health (use i16 to avoid overflow)
            creature.current_health = ((creature.current_health as i16) + (amount as i16))
                .min(creature.max_health as i16) as i8;
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
            // Use saturating arithmetic to prevent overflow
            creature.attack = creature.attack.saturating_add(attack);
            creature.current_health = creature.current_health.saturating_add(health);
            if health > 0 {
                creature.max_health = creature.max_health.saturating_add(health);
            }

            // Check for death from negative health buff
            if creature.current_health <= 0
                && !self.pending_deaths.contains(&(owner, slot)) {
                self.pending_deaths.push((owner, slot));
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
                if state.players[owner.index()].get_creature(slot).is_some()
                    && !self.pending_deaths.contains(&(owner, slot)) {
                    self.pending_deaths.push((owner, slot));
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
            status: Default::default(),
            turn_played: state.current_turn,
            frenzy_stacks: 0,
        };

        state.players[owner.index()].creatures.push(creature);

        // Queue OnPlay triggers
        self.check_creature_triggers(Trigger::OnPlay, owner, target_slot, state, card_db);
    }

    /// Grant a keyword to a target
    fn apply_grant_keyword(
        &mut self,
        target: EffectTarget,
        keyword: u16,
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
        keyword: u16,
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
