//! Validation execution engine.
//!
//! Runs matchup games using the shared execution infrastructure.

use std::sync::atomic::Ordering;
use std::time::Instant;

use rayon::prelude::*;

use crate::bots::{create_bot, BotType, MctsConfig};
use crate::cards::CardDatabase;
use crate::engine::GameEngine;
use crate::execution::{GameSeeds, ProgressReporter, ProgressStyle};
use crate::types::PlayerId;

use super::game_diagnostics::{GameDiagnosticCollector, GameDiagnosticData};
use super::types::{
    DirectionDiagnostics, DirectionResults, FactionWeights, MatchupDefinition, MatchupDiagnostics,
    MatchupResult,
};

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
    /// Uses parallel execution to maximize CPU utilization.
    pub fn run_all(
        &self,
        matchups: &[MatchupDefinition],
        faction_weights: &FactionWeights,
        games_per_matchup: usize,
        base_seed: u64,
    ) -> Vec<MatchupResult> {
        use std::sync::Mutex;
        
        // Track completion for progress reporting
        let completed = Mutex::new(0usize);
        let total = matchups.len();
        
        // Run matchups in parallel
        let mut results: Vec<(usize, MatchupResult)> = matchups
            .par_iter()
            .enumerate()
            .map(|(matchup_idx, matchup)| {
                let matchup_seed = base_seed.wrapping_add((matchup_idx * 1_000_000) as u64);
                
                let result = self.run_matchup(matchup, faction_weights, games_per_matchup, matchup_seed);
                
                // Thread-safe progress reporting
                let mut count = completed.lock().unwrap();
                *count += 1;
                let current = *count;
                drop(count); // Release lock before printing
                
                // Print progress with matchup info
                println!(
                    "[{}/{}] {} vs {} - {:.1}% F1 win rate",
                    current,
                    total,
                    matchup.faction1.display_name(),
                    matchup.faction2.display_name(),
                    result.faction1_win_rate * 100.0
                );
                
                (matchup_idx, result)
            })
            .collect();
        
        // Sort by original index to maintain deterministic order
        results.sort_by_key(|(idx, _)| *idx);
        results.into_iter().map(|(_, result)| result).collect()
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

        // Build P1/P2 diagnostics
        // Total P1 wins = F1's P1 wins (when F1 is P1) + F2's P1 wins (when F2 is P1)
        let total_p1_wins = f1_p1_results.p1_wins + f2_p1_results.p1_wins;
        let diagnostics = MatchupDiagnostics::from_directions(
            &f1_p1_results.diagnostics,
            &f2_p1_results.diagnostics,
            total_games,
        )
        .with_p1_stats(total_p1_wins, decisive_games);

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
            diagnostics,
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

        // Set up progress reporting
        let progress = if self.show_progress {
            Some(
                ProgressReporter::new(games)
                    .with_style(ProgressStyle::Simple)
                    .start(),
            )
        } else {
            None
        };

        let counter = progress.as_ref().map(|p| p.counter());

        // Run games in parallel, collecting outcomes and diagnostics
        let results: Vec<(Option<PlayerId>, u32, GameDiagnosticData)> = (0..games)
            .into_par_iter()
            .map(|i| {
                let seeds = GameSeeds::for_game(base_seed, i);
                let (winner, turns, diag) =
                    self.run_single_game_with_diagnostics(deck1, deck2, weights1, weights2, seeds);
                if let Some(ref c) = counter {
                    c.fetch_add(1, Ordering::Relaxed);
                }
                (winner, turns, diag)
            })
            .collect();

        // Finish progress reporting
        if let Some(p) = progress {
            p.finish();
        }

        // Aggregate results and diagnostics
        let mut p1_wins = 0u32;
        let mut total_turns = 0u32;
        let mut draws = 0u32;
        let mut diagnostics = DirectionDiagnostics::default();

        for (winner, turns, diag) in results {
            match winner {
                Some(PlayerId::PLAYER_ONE) => p1_wins += 1,
                None => draws += 1,
                _ => {}
            }
            total_turns += turns;

            // Aggregate diagnostic data
            match diag.first_blood {
                Some(PlayerId::PLAYER_ONE) => diagnostics.p1_first_blood_count += 1,
                Some(PlayerId::PLAYER_TWO) => diagnostics.p2_first_blood_count += 1,
                _ => {}
            }

            diagnostics.total_board_advantage += diag.board_advantage_sum;
            diagnostics.total_turns_p1_ahead += diag.turns_p1_ahead;
            diagnostics.total_turns_p2_ahead += diag.turns_p2_ahead;
            diagnostics.total_turns_even += diag.turns_even;
            diagnostics.total_turn_count += diag.turn_count;
            diagnostics.total_p1_essence += diag.p1_essence_spent;
            diagnostics.total_p2_essence += diag.p2_essence_spent;
            diagnostics.total_p1_face_damage += diag.p1_face_damage;
            diagnostics.total_p2_face_damage += diag.p2_face_damage;
            diagnostics.total_p1_kills += diag.p1_creatures_killed;
            diagnostics.total_p2_kills += diag.p2_creatures_killed;
            diagnostics.total_p1_losses += diag.p1_creatures_lost;
            diagnostics.total_p2_losses += diag.p2_creatures_lost;
            diagnostics.game_lengths.push(turns);
        }

        DirectionResults {
            p1_wins,
            total_turns,
            draws,
            games: games as u32,
            duration_secs: start_time.elapsed().as_secs_f64(),
            diagnostics,
        }
    }

    /// Run a single game between two MCTS bots with diagnostic collection.
    fn run_single_game_with_diagnostics(
        &self,
        deck1: &[crate::types::CardId],
        deck2: &[crate::types::CardId],
        weights1: Option<&crate::bots::BotWeights>,
        weights2: Option<&crate::bots::BotWeights>,
        seeds: GameSeeds,
    ) -> (Option<PlayerId>, u32, GameDiagnosticData) {
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

        // Create diagnostic collector
        let mut collector = GameDiagnosticCollector::new();

        // Create and start game
        let mut engine = GameEngine::new(self.card_db);
        engine.start_game(deck1.to_vec(), deck2.to_vec(), seeds.game);

        // Main game loop
        let max_actions = 1000;
        let mut action_count = 0;
        let mut last_turn = 0;

        while !engine.is_game_over() && action_count < max_actions {
            // Record turn state at start of each new turn
            let current_turn = engine.turn_number();
            if current_turn != last_turn {
                collector.record_turn(&engine);
                last_turn = current_turn;
            }

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

        // Final state capture
        collector.record_turn(&engine);

        (
            engine.winner(),
            engine.turn_number() as u32,
            collector.finalize(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decks::Faction;
    use crate::validation::matchup::MatchupBuilder;

    fn test_card_db() -> CardDatabase {
        let cards_path = crate::data_dir().join("cards/core_set");
        CardDatabase::load_from_directory(cards_path).unwrap()
    }

    fn test_deck_registry() -> crate::decks::DeckRegistry {
        let decks_path = crate::data_dir().join("decks");
        crate::decks::DeckRegistry::load_from_directory(decks_path).unwrap()
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
        let weights_path = crate::data_dir().join("weights");
        let weights = FactionWeights::load_from_directory(&weights_path, true);

        // Should have loaded at least some weights (if files exist)
        // This test just verifies the loading doesn't crash
        assert!(weights.get(Faction::Neutral).is_none());
    }
}
