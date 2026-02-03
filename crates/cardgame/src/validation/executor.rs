//! Validation execution engine.
//!
//! Runs matchup games using the shared execution infrastructure.
//! Now uses `UnifiedMatchup` which preserves commander information from deck definitions.

use std::sync::atomic::Ordering;
use std::time::Instant;

use rayon::prelude::*;

use crate::bots::{create_bot, AlphaBetaConfig, BotType, MctsConfig};
use crate::cards::CardDatabase;
use crate::engine::{GameEngine, GameInitError};
use crate::execution::{GameSeeds, ProgressReporter, ProgressStyle, UnifiedMatchup, MAX_ACTIONS_PER_GAME};
use crate::types::PlayerId;

use super::game_diagnostics::{GameDiagnosticCollector, GameDiagnosticData};
use super::types::{
    DirectionDiagnostics, DirectionResults, MatchupDiagnostics, MatchupResult,
};
use super::ArchetypeWeights;

/// Executor for running validation matchups.
pub struct ValidationExecutor<'a> {
    card_db: &'a CardDatabase,
    bot_type: BotType,
    mcts_config: MctsConfig,
    alphabeta_config: AlphaBetaConfig,
    show_progress: bool,
}

impl<'a> ValidationExecutor<'a> {
    /// Create a new validation executor.
    /// Default bot type is AlphaBeta (depth 6) for faster validation.
    /// Use `with_bot_type(BotType::Mcts)` for MCTS-based validation.
    pub fn new(card_db: &'a CardDatabase, mcts_sims: u32) -> Self {
        Self {
            card_db,
            bot_type: BotType::AlphaBeta, // Default to AlphaBeta for speed
            mcts_config: MctsConfig {
                simulations: mcts_sims,
                exploration: 1.414,
                max_rollout_depth: 100,
                parallel_trees: 1,
                leaf_rollouts: 1,
            },
            alphabeta_config: AlphaBetaConfig::with_depth(6),
            show_progress: false,
        }
    }

    /// Set the bot type for validation.
    pub fn with_bot_type(mut self, bot_type: BotType) -> Self {
        self.bot_type = bot_type;
        self
    }

    /// Set the alpha-beta search depth.
    pub fn with_alphabeta_depth(mut self, depth: u32) -> Self {
        self.alphabeta_config = AlphaBetaConfig::with_depth(depth);
        self
    }

    /// Enable or disable progress display.
    pub fn with_progress(mut self, show: bool) -> Self {
        self.show_progress = show;
        self
    }

    /// Run all matchups and return results.
    /// Uses parallel execution to maximize CPU utilization.
    ///
    /// # Errors
    /// Returns `GameInitError` if any commander is not found in the card database.
    pub fn run_all(
        &self,
        matchups: &[UnifiedMatchup],
        archetype_weights: &ArchetypeWeights,
        games_per_matchup: usize,
        base_seed: u64,
    ) -> Result<Vec<MatchupResult>, GameInitError> {
        // Validate all commanders exist before starting any games
        self.validate_matchup_commanders(matchups)?;

        // Set up progress reporting for matchups
        // Use Rich style to show rate and ETA for long-running benchmarks
        let progress = if self.show_progress {
            Some(
                ProgressReporter::new(matchups.len())
                    .with_style(ProgressStyle::Rich)
                    .start(),
            )
        } else {
            None
        };

        let counter = progress.as_ref().map(|p| p.counter());

        // Run matchups in parallel
        let mut results: Vec<(usize, MatchupResult)> = matchups
            .par_iter()
            .enumerate()
            .map(|(matchup_idx, matchup)| {
                let matchup_seed = base_seed.wrapping_add((matchup_idx * 1_000_000) as u64);

                let result =
                    self.run_matchup(matchup, archetype_weights, games_per_matchup, matchup_seed);

                // Increment progress counter
                if let Some(ref c) = counter {
                    c.fetch_add(1, Ordering::Relaxed);
                }

                (matchup_idx, result)
            })
            .collect();

        // Finish progress reporting
        if let Some(p) = progress {
            p.finish();
        }

        // Sort by original index to maintain deterministic order
        results.sort_by_key(|(idx, _)| *idx);
        Ok(results.into_iter().map(|(_, result)| result).collect())
    }

    /// Validate that all commanders in the matchups exist in the card database.
    fn validate_matchup_commanders(&self, matchups: &[UnifiedMatchup]) -> Result<(), GameInitError> {
        for matchup in matchups {
            let commander1 = matchup.commander1();
            let commander2 = matchup.commander2();

            if self.card_db.get_commander(commander1).is_none() {
                return Err(GameInitError::CommanderNotFound {
                    commander_id: commander1.0,
                    player: 1,
                });
            }
            if self.card_db.get_commander(commander2).is_none() {
                return Err(GameInitError::CommanderNotFound {
                    commander_id: commander2.0,
                    player: 2,
                });
            }
        }
        Ok(())
    }

    /// Run a single matchup (both player orders).
    ///
    /// Uses archetype weights based on deck playstyle.
    pub fn run_matchup(
        &self,
        matchup: &UnifiedMatchup,
        archetype_weights: &ArchetypeWeights,
        games_per_side: usize,
        base_seed: u64,
    ) -> MatchupResult {
        // Get weights for each deck based on archetype
        let weights1 = archetype_weights.get(&matchup.deck1.playstyle);
        let weights2 = archetype_weights.get(&matchup.deck2.playstyle);

        // Run Direction 1: Deck1 as P1 vs Deck2 as P2
        let dir1_results = self.run_direction(
            matchup,
            false, // deck1 as P1
            weights1,
            weights2,
            games_per_side,
            base_seed,
        );

        // Run Direction 2: Deck2 as P1 vs Deck1 as P2
        let dir2_results = self.run_direction(
            matchup,
            true, // reversed: deck2 as P1
            weights2,
            weights1,
            games_per_side,
            base_seed.wrapping_add(500_000),
        );

        // Calculate combined results
        // deck1 wins when it's P1 (direction 1, P1 wins) + when it's P2 (direction 2, P2 wins)
        let deck1_as_p1_wins = dir1_results.p1_wins;
        let deck1_as_p1_games = games_per_side as u32;
        let deck1_as_p2_wins = games_per_side as u32 - dir2_results.p1_wins - dir2_results.draws;
        let deck1_as_p2_games = games_per_side as u32;

        let total_games = (games_per_side * 2) as u32;
        let deck1_total_wins = deck1_as_p1_wins + deck1_as_p2_wins;
        let total_draws = dir1_results.draws + dir2_results.draws;
        let deck2_total_wins = total_games - deck1_total_wins - total_draws;

        let total_turns = dir1_results.total_turns + dir2_results.total_turns;
        let total_time = dir1_results.duration_secs + dir2_results.duration_secs;

        let decisive_games = total_games - total_draws;

        // Build P1/P2 diagnostics
        // Total P1 wins = direction 1 P1 wins + direction 2 P1 wins
        let total_p1_wins = dir1_results.p1_wins + dir2_results.p1_wins;
        let diagnostics = MatchupDiagnostics::from_directions(
            &dir1_results.diagnostics,
            &dir2_results.diagnostics,
            total_games,
            total_p1_wins,
            decisive_games,
        );

        MatchupResult {
            faction1: matchup.faction1_or_neutral().as_tag().to_string(),
            faction2: matchup.faction2_or_neutral().as_tag().to_string(),
            deck1_id: matchup.deck1.id.clone(),
            deck2_id: matchup.deck2.id.clone(),
            commander1_id: matchup.deck1.commander,
            commander2_id: matchup.deck2.commander,
            commander1_name: self
                .card_db
                .get_commander(matchup.commander1())
                .map(|c| c.name.clone())
                .unwrap_or_else(|| format!("Commander {}", matchup.deck1.commander)),
            commander2_name: self
                .card_db
                .get_commander(matchup.commander2())
                .map(|c| c.name.clone())
                .unwrap_or_else(|| format!("Commander {}", matchup.deck2.commander)),
            f1_as_p1_wins: deck1_as_p1_wins,
            f1_as_p1_games: deck1_as_p1_games,
            f1_as_p2_wins: deck1_as_p2_wins,
            f1_as_p2_games: deck1_as_p2_games,
            faction1_total_wins: deck1_total_wins,
            faction2_total_wins: deck2_total_wins,
            draws: total_draws,
            dir1_draws: dir1_results.draws,
            dir2_draws: dir2_results.draws,
            total_games,
            faction1_win_rate: if decisive_games > 0 {
                deck1_total_wins as f64 / decisive_games as f64
            } else {
                0.5
            },
            faction2_win_rate: if decisive_games > 0 {
                deck2_total_wins as f64 / decisive_games as f64
            } else {
                0.5
            },
            avg_turns: total_turns as f64 / total_games as f64,
            total_time_secs: total_time,
            diagnostics,
        }
    }

    /// Run games for a single direction.
    ///
    /// If `reversed` is false, deck1 is P1 and deck2 is P2.
    /// If `reversed` is true, deck2 is P1 and deck1 is P2.
    fn run_direction(
        &self,
        matchup: &UnifiedMatchup,
        reversed: bool,
        weights1: Option<&crate::bots::BotWeights>,
        weights2: Option<&crate::bots::BotWeights>,
        games: usize,
        base_seed: u64,
    ) -> DirectionResults {
        let start_time = Instant::now();

        // NOTE: Games run SEQUENTIALLY here because parallelism happens at the matchup level.
        // With 48+ matchups running in parallel, this avoids nested parallelism overhead
        // that was causing only 30% CPU utilization instead of 96-100%.

        // Run games sequentially within each matchup
        let results: Vec<(Option<PlayerId>, u32, GameDiagnosticData)> = (0..games)
            .map(|i| {
                let seeds = GameSeeds::for_game(base_seed, i);
                let (winner, turns, diag) =
                    self.run_single_game_with_diagnostics(matchup, reversed, weights1, weights2, seeds);
                (winner, turns, diag)
            })
            .collect();

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

    /// Run a single game between two bots with diagnostic collection.
    fn run_single_game_with_diagnostics(
        &self,
        matchup: &UnifiedMatchup,
        reversed: bool,
        weights1: Option<&crate::bots::BotWeights>,
        weights2: Option<&crate::bots::BotWeights>,
        seeds: GameSeeds,
    ) -> (Option<PlayerId>, u32, GameDiagnosticData) {
        // Create bots using the factory with configured bot type
        let mut bot1 = create_bot(
            self.card_db,
            &self.bot_type,
            weights1,
            &self.mcts_config,
            &self.alphabeta_config,
            seeds.bot1,
        );
        let mut bot2 = create_bot(
            self.card_db,
            &self.bot_type,
            weights2,
            &self.mcts_config,
            &self.alphabeta_config,
            seeds.bot2,
        );

        // Reset bots
        bot1.reset();
        bot2.reset();

        // Create diagnostic collector
        let mut collector = GameDiagnosticCollector::new();

        // Create and start game using deck definitions with commanders
        let mut engine = GameEngine::new(self.card_db);

        // Use the appropriate deck order based on direction
        if reversed {
            engine
                .start_game(&matchup.deck2, &matchup.deck1, seeds.game)
                .expect("Failed to start game - commander not found in card database");
        } else {
            engine
                .start_game(&matchup.deck1, &matchup.deck2, seeds.game)
                .expect("Failed to start game - commander not found in card database");
        }

        // Main game loop with safety limit from execution::game_loop
        let mut action_count = 0;
        let mut last_turn = 0;

        while !engine.is_game_over() && action_count < MAX_ACTIONS_PER_GAME {
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

        // Log warning if action limit was hit without game completion
        if action_count >= MAX_ACTIONS_PER_GAME && !engine.is_game_over() {
            log::warn!(
                "Game exceeded {} action limit without completion (seed: {})",
                MAX_ACTIONS_PER_GAME,
                seeds.game
            );
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
    use crate::execution::{GameData, MatchupBuilder};

    fn load_test_data() -> GameData {
        GameData::load_default(true).expect("Failed to load test data")
    }

    #[test]
    fn test_run_single_matchup() {
        let data = load_test_data();
        let builder = MatchupBuilder::new(&data.deck_registry);

        let matchups = builder.build_faction_matchups(Faction::Argentum, Faction::Symbiote);
        let matchup = &matchups[0];

        // Use GreedyBot for fast test execution
        let executor = ValidationExecutor::new(&data.card_db, 10).with_bot_type(BotType::Greedy);
        let archetype_weights = ArchetypeWeights::new();

        let result = executor.run_matchup(matchup, &archetype_weights, 2, 42);

        assert_eq!(result.total_games, 4); // 2 games each direction
        assert!(result.faction1_win_rate >= 0.0 && result.faction1_win_rate <= 1.0);

        // Verify commander info is populated
        assert!(result.commander1_id > 0);
        assert!(result.commander2_id > 0);
        assert!(!result.commander1_name.is_empty());
        assert!(!result.commander2_name.is_empty());
    }

    #[test]
    fn test_commanders_are_used() {
        let data = load_test_data();

        // Get specific decks with known commanders
        let argentum_decks = data.deck_registry.decks_for_faction(Faction::Argentum);
        let symbiote_decks = data.deck_registry.decks_for_faction(Faction::Symbiote);

        assert!(!argentum_decks.is_empty());
        assert!(!symbiote_decks.is_empty());

        let matchup = UnifiedMatchup::new(
            argentum_decks[0].clone(),
            symbiote_decks[0].clone(),
        );

        // Verify commanders match deck definitions
        assert_eq!(matchup.commander1().0, argentum_decks[0].commander);
        assert_eq!(matchup.commander2().0, symbiote_decks[0].commander);

        // Run a game and verify result has correct commander info
        let executor = ValidationExecutor::new(&data.card_db, 10).with_bot_type(BotType::Greedy);
        let archetype_weights = ArchetypeWeights::new();
        let result = executor.run_matchup(&matchup, &archetype_weights, 1, 42);

        assert_eq!(result.commander1_id, argentum_decks[0].commander);
        assert_eq!(result.commander2_id, symbiote_decks[0].commander);
    }

    #[test]
    fn test_archetype_weights_loading() {
        let weights_path = crate::data_dir().join("weights");
        let weights = ArchetypeWeights::load_from_directory(&weights_path, true);

        // Should have loaded at least some archetype weights
        // This test verifies the loading doesn't crash
        assert!(weights.get("aggro").is_some() || weights.get("control").is_some());
    }
}
