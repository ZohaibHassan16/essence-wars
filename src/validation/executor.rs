//! Validation execution engine.
//!
//! Runs matchup games using the shared execution infrastructure.

use std::time::Instant;

use crate::bots::{create_bot, BotType, MctsConfig};
use crate::cards::CardDatabase;
use crate::engine::GameEngine;
use crate::execution::{run_batch_parallel, BatchConfig, GameSeeds, ProgressStyle};
use crate::types::PlayerId;

use super::types::{DirectionResults, FactionWeights, MatchupDefinition, MatchupResult};

/// Executor for running validation matchups.
pub struct ValidationExecutor<'a> {
    card_db: &'a CardDatabase,
    mcts_config: MctsConfig,
    show_progress: bool,
}

impl<'a> ValidationExecutor<'a> {
    /// Create a new validation executor.
    pub fn new(card_db: &'a CardDatabase, mcts_sims: u32) -> Self {
        Self {
            card_db,
            mcts_config: MctsConfig {
                simulations: mcts_sims,
                exploration: 1.414,
                max_rollout_depth: 100,
                parallel_trees: 1,
                leaf_rollouts: 1,
            },
            show_progress: false,
        }
    }

    /// Enable or disable progress display.
    pub fn with_progress(mut self, show: bool) -> Self {
        self.show_progress = show;
        self
    }

    /// Run all matchups and return results.
    pub fn run_all(
        &self,
        matchups: &[MatchupDefinition],
        faction_weights: &FactionWeights,
        games_per_matchup: usize,
        base_seed: u64,
    ) -> Vec<MatchupResult> {
        let mut results = Vec::new();

        for (matchup_idx, matchup) in matchups.iter().enumerate() {
            let matchup_seed = base_seed.wrapping_add((matchup_idx * 1_000_000) as u64);

            // Always announce matchup
            println!(
                "{} vs {} ({} games each direction)...",
                matchup.faction1.display_name(),
                matchup.faction2.display_name(),
                games_per_matchup
            );

            let result = self.run_matchup(matchup, faction_weights, games_per_matchup, matchup_seed);
            results.push(result);

            if self.show_progress {
                println!();
            }
        }

        results
    }

    /// Run a single matchup (both player orders).
    pub fn run_matchup(
        &self,
        matchup: &MatchupDefinition,
        faction_weights: &FactionWeights,
        games_per_side: usize,
        base_seed: u64,
    ) -> MatchupResult {
        let weights1 = faction_weights.get(matchup.faction1);
        let weights2 = faction_weights.get(matchup.faction2);

        // Run F1 as P1 vs F2 as P2
        let f1_p1_results = self.run_direction(
            &matchup.deck1_cards,
            &matchup.deck2_cards,
            weights1,
            weights2,
            games_per_side,
            base_seed,
        );

        // Run F2 as P1 vs F1 as P2
        let f2_p1_results = self.run_direction(
            &matchup.deck2_cards,
            &matchup.deck1_cards,
            weights2,
            weights1,
            games_per_side,
            base_seed.wrapping_add(500_000),
        );

        // Calculate combined results
        let f1_as_p1_wins = f1_p1_results.p1_wins;
        let f1_as_p1_games = games_per_side as u32;
        let f1_as_p2_wins = games_per_side as u32 - f2_p1_results.p1_wins - f2_p1_results.draws;
        let f1_as_p2_games = games_per_side as u32;

        let total_games = (games_per_side * 2) as u32;
        let faction1_total_wins = f1_as_p1_wins + f1_as_p2_wins;
        let total_draws = f1_p1_results.draws + f2_p1_results.draws;
        let faction2_total_wins = total_games - faction1_total_wins - total_draws;

        let total_turns = f1_p1_results.total_turns + f2_p1_results.total_turns;
        let total_time = f1_p1_results.duration_secs + f2_p1_results.duration_secs;

        let decisive_games = total_games - total_draws;

        MatchupResult {
            faction1: matchup.faction1.as_tag().to_string(),
            faction2: matchup.faction2.as_tag().to_string(),
            deck1_id: matchup.deck1_id.clone(),
            deck2_id: matchup.deck2_id.clone(),
            f1_as_p1_wins,
            f1_as_p1_games,
            f1_as_p2_wins,
            f1_as_p2_games,
            faction1_total_wins,
            faction2_total_wins,
            draws: total_draws,
            total_games,
            faction1_win_rate: if decisive_games > 0 {
                faction1_total_wins as f64 / decisive_games as f64
            } else {
                0.5
            },
            faction2_win_rate: if decisive_games > 0 {
                faction2_total_wins as f64 / decisive_games as f64
            } else {
                0.5
            },
            avg_turns: total_turns as f64 / total_games as f64,
            total_time_secs: total_time,
        }
    }

    /// Run games for a single direction (P1 deck vs P2 deck).
    fn run_direction(
        &self,
        deck1: &[crate::types::CardId],
        deck2: &[crate::types::CardId],
        weights1: Option<&crate::bots::BotWeights>,
        weights2: Option<&crate::bots::BotWeights>,
        games: usize,
        base_seed: u64,
    ) -> DirectionResults {
        let start_time = Instant::now();

        // Configure batch execution
        let batch_config = if self.show_progress {
            BatchConfig::new(games, base_seed).with_progress(ProgressStyle::Simple)
        } else {
            BatchConfig::new(games, base_seed)
        };

        // Run games in parallel
        let result = run_batch_parallel(&batch_config, |seeds| {
            self.run_single_game(deck1, deck2, weights1, weights2, seeds)
        });

        // Aggregate results
        let mut p1_wins = 0u32;
        let mut total_turns = 0u32;
        let mut draws = 0u32;

        for outcome in &result.outcomes {
            match outcome.winner {
                Some(PlayerId::PLAYER_ONE) => p1_wins += 1,
                None => draws += 1,
                _ => {}
            }
            total_turns += outcome.turns;
        }

        DirectionResults {
            p1_wins,
            total_turns,
            draws,
            games: games as u32,
            duration_secs: start_time.elapsed().as_secs_f64(),
        }
    }

    /// Run a single game between two MCTS bots.
    fn run_single_game(
        &self,
        deck1: &[crate::types::CardId],
        deck2: &[crate::types::CardId],
        weights1: Option<&crate::bots::BotWeights>,
        weights2: Option<&crate::bots::BotWeights>,
        seeds: GameSeeds,
    ) -> crate::execution::GameOutcome {
        let start = Instant::now();

        // Create MCTS bots using the factory
        let mut bot1 = create_bot(
            self.card_db,
            &BotType::Mcts,
            weights1,
            &self.mcts_config,
            seeds.bot1,
        );
        let mut bot2 = create_bot(
            self.card_db,
            &BotType::Mcts,
            weights2,
            &self.mcts_config,
            seeds.bot2,
        );

        // Reset bots
        bot1.reset();
        bot2.reset();

        // Create and start game
        let mut engine = GameEngine::new(self.card_db);
        engine.start_game(deck1.to_vec(), deck2.to_vec(), seeds.game);

        // Main game loop
        let max_actions = 1000;
        let mut action_count = 0;

        while !engine.is_game_over() && action_count < max_actions {
            let action = if engine.current_player() == PlayerId::PLAYER_ONE {
                bot1.select_action_with_engine(&engine)
            } else {
                bot2.select_action_with_engine(&engine)
            };

            if engine.apply_action(action).is_err() {
                break;
            }
            action_count += 1;
        }

        crate::execution::GameOutcome::new(
            engine.winner(),
            engine.turn_number() as u32,
            start.elapsed(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decks::Faction;
    use crate::validation::matchup::MatchupBuilder;

    fn test_card_db() -> CardDatabase {
        CardDatabase::load_from_directory("data/cards/core_set").unwrap()
    }

    fn test_deck_registry() -> crate::decks::DeckRegistry {
        crate::decks::DeckRegistry::load_from_directory("data/decks").unwrap()
    }

    #[test]
    fn test_run_single_matchup() {
        let card_db = test_card_db();
        let registry = test_deck_registry();
        let builder = MatchupBuilder::new(&registry, &card_db);

        let matchups = builder.build_faction_matchups();
        let matchup = &matchups[0];

        let executor = ValidationExecutor::new(&card_db, 10); // Low sims for test speed
        let faction_weights = FactionWeights::new();

        let result = executor.run_matchup(matchup, &faction_weights, 2, 42);

        assert_eq!(result.total_games, 4); // 2 games each direction
        assert!(result.faction1_win_rate >= 0.0 && result.faction1_win_rate <= 1.0);
    }

    #[test]
    fn test_faction_weights_loading() {
        let weights = FactionWeights::load_from_directory(std::path::Path::new("data/weights"), true);

        // Should have loaded at least some weights (if files exist)
        // This test just verifies the loading doesn't crash
        assert!(weights.get(Faction::Neutral).is_none());
    }
}
