//! Effect context and target resolution utilities.
//!
//! This module provides the shared context passed to effect handlers
//! and centralized target resolution logic.

use std::collections::VecDeque;

use crate::core::effects::{CreatureFilter, Effect, EffectResult, EffectSource, EffectTarget, PendingEffect};
use crate::core::cards::CardDatabase;
use crate::core::state::GameState;
use crate::core::types::{PlayerId, Slot};

/// Context passed to effect handlers containing shared mutable state.
///
/// This struct bundles all the state that handlers need to modify,
/// providing a clean interface and centralizing common operations
/// like death tracking.
pub struct EffectContext<'a> {
    /// The game state to modify
    pub state: &'a mut GameState,
    /// Card database for looking up card definitions
    pub card_db: &'a CardDatabase,
    /// Creatures marked for death (processed after each effect)
    pending_deaths: &'a mut Vec<(PlayerId, Slot)>,
    /// Accumulated result from effect resolution (for conditional triggers)
    accumulated_result: &'a mut EffectResult,
    /// The effect queue for adding triggered effects
    queue: &'a mut VecDeque<PendingEffect>,
}

impl<'a> EffectContext<'a> {
    /// Create a new effect context.
    pub fn new(
        state: &'a mut GameState,
        card_db: &'a CardDatabase,
        pending_deaths: &'a mut Vec<(PlayerId, Slot)>,
        accumulated_result: &'a mut EffectResult,
        queue: &'a mut VecDeque<PendingEffect>,
    ) -> Self {
        Self {
            state,
            card_db,
            pending_deaths,
            accumulated_result,
            queue,
        }
    }

    /// Mark a creature for death processing.
    ///
    /// This centralizes the death-marking logic that was previously
    /// duplicated across 11+ locations in effect_queue.rs.
    /// Automatically sets `accumulated_result.target_died = true`.
    pub fn mark_for_death(&mut self, owner: PlayerId, slot: Slot) {
        if !self.pending_deaths.contains(&(owner, slot)) {
            self.pending_deaths.push((owner, slot));
            self.accumulated_result.target_died = true;
        }
    }

    /// Check if a creature is already marked for death.
    pub fn is_marked_for_death(&self, owner: PlayerId, slot: Slot) -> bool {
        self.pending_deaths.contains(&(owner, slot))
    }

    /// Queue an additional effect (for triggered abilities).
    pub fn push_effect(&mut self, effect: Effect, source: EffectSource) {
        self.queue.push_back(PendingEffect::new(effect, source));
    }

    /// Get access to pending deaths (for handlers that need to read it).
    pub fn pending_deaths(&self) -> &[(PlayerId, Slot)] {
        self.pending_deaths
    }

    /// Get a reference to the accumulated result.
    pub fn accumulated_result(&self) -> &EffectResult {
        self.accumulated_result
    }
}

/// Result of resolving an effect target to concrete positions.
#[derive(Debug, Clone)]
pub enum ResolvedTargets {
    /// Single creature target
    Single { owner: PlayerId, slot: Slot },
    /// Multiple creature targets (for AoE effects)
    Multiple(Vec<(PlayerId, Slot)>),
    /// Player target (for face damage/healing)
    Player(PlayerId),
    /// No valid targets
    None,
}

/// Utility for resolving effect targets to concrete creature/player positions.
///
/// This centralizes the target resolution logic that was previously
/// duplicated in 7+ apply_* methods in effect_queue.rs.
pub struct TargetResolver;

impl TargetResolver {
    /// Resolve an EffectTarget to concrete creature positions.
    ///
    /// Applies the optional filter to AoE targets.
    /// Returns `ResolvedTargets` which can be single, multiple, player, or none.
    pub fn resolve(
        target: &EffectTarget,
        filter: Option<&CreatureFilter>,
        state: &GameState,
    ) -> ResolvedTargets {
        match target {
            EffectTarget::Creature { owner, slot } => {
                if state.players[owner.index()].get_creature(*slot).is_some() {
                    ResolvedTargets::Single { owner: *owner, slot: *slot }
                } else {
                    ResolvedTargets::None
                }
            }
            EffectTarget::Player(player) => {
                ResolvedTargets::Player(*player)
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
                ResolvedTargets::Multiple(creatures)
            }
            EffectTarget::AllAllyCreatures(player) => {
                let creatures: Vec<_> = state.players[player.index()]
                    .creatures.iter()
                    .filter(|c| filter.is_none_or(|f| f.matches(c.current_health, c.keywords.0)))
                    .map(|c| (*player, c.slot))
                    .collect();
                ResolvedTargets::Multiple(creatures)
            }
            EffectTarget::AllEnemyCreatures(player) => {
                let enemy = player.opponent();
                let creatures: Vec<_> = state.players[enemy.index()]
                    .creatures.iter()
                    .filter(|c| filter.is_none_or(|f| f.matches(c.current_health, c.keywords.0)))
                    .map(|c| (enemy, c.slot))
                    .collect();
                ResolvedTargets::Multiple(creatures)
            }
            EffectTarget::TriggerSource | EffectTarget::None => {
                ResolvedTargets::None
            }
        }
    }

    /// Convenience method to iterate over resolved creature targets.
    ///
    /// Calls the callback for each resolved creature position.
    /// Handles both single and multiple targets uniformly.
    pub fn for_each_creature<F>(
        target: &EffectTarget,
        filter: Option<&CreatureFilter>,
        state: &GameState,
        mut callback: F,
    ) where
        F: FnMut(PlayerId, Slot),
    {
        match Self::resolve(target, filter, state) {
            ResolvedTargets::Single { owner, slot } => callback(owner, slot),
            ResolvedTargets::Multiple(creatures) => {
                for (owner, slot) in creatures {
                    callback(owner, slot);
                }
            }
            ResolvedTargets::Player(_) | ResolvedTargets::None => {}
        }
    }

    /// Resolve targets and collect as a Vec for cases where we need
    /// to iterate multiple times or need ownership.
    pub fn collect_creature_targets(
        target: &EffectTarget,
        filter: Option<&CreatureFilter>,
        state: &GameState,
    ) -> Vec<(PlayerId, Slot)> {
        match Self::resolve(target, filter, state) {
            ResolvedTargets::Single { owner, slot } => vec![(owner, slot)],
            ResolvedTargets::Multiple(creatures) => creatures,
            ResolvedTargets::Player(_) | ResolvedTargets::None => vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::state::Creature;
    use crate::core::keywords::Keywords;
    use crate::core::types::{CardId, CreatureInstanceId};

    fn create_test_creature(owner: PlayerId, slot: Slot) -> Creature {
        Creature {
            instance_id: CreatureInstanceId(1),
            card_id: CardId(1000),
            owner,
            slot,
            attack: 2,
            current_health: 3,
            max_health: 3,
            base_attack: 2,
            base_health: 3,
            keywords: Keywords::none(),
            status: Default::default(),
            turn_played: 1,
            frenzy_stacks: 0,
        }
    }

    #[test]
    fn test_resolve_single_creature() {
        let mut state = GameState::default();
        state.players[0].creatures.push(create_test_creature(PlayerId::PLAYER_ONE, Slot(0)));

        let target = EffectTarget::Creature { owner: PlayerId::PLAYER_ONE, slot: Slot(0) };
        let result = TargetResolver::resolve(&target, None, &state);

        match result {
            ResolvedTargets::Single { owner, slot } => {
                assert_eq!(owner, PlayerId::PLAYER_ONE);
                assert_eq!(slot, Slot(0));
            }
            _ => unreachable!("test helper - expected Single target"),
        }
    }

    #[test]
    fn test_resolve_nonexistent_creature() {
        let state = GameState::default();
        let target = EffectTarget::Creature { owner: PlayerId::PLAYER_ONE, slot: Slot(0) };
        let result = TargetResolver::resolve(&target, None, &state);

        assert!(matches!(result, ResolvedTargets::None));
    }

    #[test]
    fn test_resolve_player() {
        let state = GameState::default();
        let target = EffectTarget::Player(PlayerId::PLAYER_TWO);
        let result = TargetResolver::resolve(&target, None, &state);

        match result {
            ResolvedTargets::Player(player) => assert_eq!(player, PlayerId::PLAYER_TWO),
            _ => unreachable!("test helper - expected Player target"),
        }
    }

    #[test]
    fn test_resolve_all_creatures() {
        let mut state = GameState::default();
        state.players[0].creatures.push(create_test_creature(PlayerId::PLAYER_ONE, Slot(0)));
        state.players[0].creatures.push(create_test_creature(PlayerId::PLAYER_ONE, Slot(1)));
        state.players[1].creatures.push(create_test_creature(PlayerId::PLAYER_TWO, Slot(0)));

        let target = EffectTarget::AllCreatures;
        let result = TargetResolver::resolve(&target, None, &state);

        match result {
            ResolvedTargets::Multiple(creatures) => {
                assert_eq!(creatures.len(), 3);
            }
            _ => unreachable!("test helper - expected Multiple targets"),
        }
    }

    #[test]
    fn test_resolve_ally_creatures() {
        let mut state = GameState::default();
        state.players[0].creatures.push(create_test_creature(PlayerId::PLAYER_ONE, Slot(0)));
        state.players[0].creatures.push(create_test_creature(PlayerId::PLAYER_ONE, Slot(1)));
        state.players[1].creatures.push(create_test_creature(PlayerId::PLAYER_TWO, Slot(0)));

        let target = EffectTarget::AllAllyCreatures(PlayerId::PLAYER_ONE);
        let result = TargetResolver::resolve(&target, None, &state);

        match result {
            ResolvedTargets::Multiple(creatures) => {
                assert_eq!(creatures.len(), 2);
                assert!(creatures.iter().all(|(owner, _)| *owner == PlayerId::PLAYER_ONE));
            }
            _ => unreachable!("test helper - expected Multiple targets"),
        }
    }

    #[test]
    fn test_resolve_enemy_creatures() {
        let mut state = GameState::default();
        state.players[0].creatures.push(create_test_creature(PlayerId::PLAYER_ONE, Slot(0)));
        state.players[1].creatures.push(create_test_creature(PlayerId::PLAYER_TWO, Slot(0)));
        state.players[1].creatures.push(create_test_creature(PlayerId::PLAYER_TWO, Slot(1)));

        let target = EffectTarget::AllEnemyCreatures(PlayerId::PLAYER_ONE);
        let result = TargetResolver::resolve(&target, None, &state);

        match result {
            ResolvedTargets::Multiple(creatures) => {
                assert_eq!(creatures.len(), 2);
                assert!(creatures.iter().all(|(owner, _)| *owner == PlayerId::PLAYER_TWO));
            }
            _ => unreachable!("test helper - expected Multiple targets"),
        }
    }
}
