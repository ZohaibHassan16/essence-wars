//! Game loop callback for collecting card statistics.

use crate::actions::Action;
use crate::execution::game_loop::{ActionContext, GameLoopCallback, GameLoopResult};

use super::collector::CardStatsCollector;
use super::types::GameCardTracker;

/// Callback that collects card play statistics during game execution.
pub struct CardStatsCallback<'a> {
    /// Tracker for the current game.
    pub tracker: GameCardTracker,
    /// Collector for aggregating across games.
    pub collector: &'a mut CardStatsCollector,
}

impl<'a> CardStatsCallback<'a> {
    /// Create a new callback with the given collector.
    pub fn new(collector: &'a mut CardStatsCollector) -> Self {
        Self {
            tracker: GameCardTracker::new(),
            collector,
        }
    }

    /// Reset the tracker for a new game.
    pub fn reset(&mut self) {
        self.tracker.reset();
    }
}

impl GameLoopCallback for CardStatsCallback<'_> {
    fn on_action(&mut self, ctx: &ActionContext) -> bool {
        // Only track PlayCard actions
        if let Action::PlayCard { hand_index, slot: _ } = ctx.action {
            // Get the card from the player's hand before the action is applied
            let player_state = &ctx.engine.state.players[ctx.player.index()];

            if let Some(card_instance) = player_state.hand.get(hand_index as usize) {
                let card_id = card_instance.card_id;
                self.tracker.record_play(ctx.player, card_id, ctx.turn);
            }
        }
        true // Continue game
    }

    fn on_complete(&mut self, result: &GameLoopResult) {
        // Aggregate this game's stats
        if let GameLoopResult::Completed { winner, .. } = result {
            self.collector.aggregate_game(&self.tracker, *winner);
        }
        // Reset tracker for next game
        self.tracker.reset();
    }
}
