//! Game runner for executing games between bots.

use std::time::{Duration, Instant};

use crate::arena::logger::{ActionLogger, ActionRecord, StateSnapshot};
use crate::arena::stats::MatchStats;
use crate::bots::Bot;
use crate::cards::CardDatabase;
use crate::engine::GameEngine;
use crate::types::{CardId, PlayerId};

/// Result of a single game.
#[derive(Clone, Debug)]
pub struct GameResult {
    /// Winner of the game (None if draw)
    pub winner: Option<PlayerId>,
    /// Number of turns played
    pub turns: u32,
    /// Seed used for this game
    pub seed: u64,
    /// Time taken to play the game
    pub duration: Duration,
    /// Action records (only if logging was enabled)
    pub actions: Vec<ActionRecord>,
}

/// Runs games between bots.
pub struct GameRunner<'a> {
    card_db: &'a CardDatabase,
    logger: Option<ActionLogger>,
}

impl<'a> GameRunner<'a> {
    /// Create a new game runner.
    pub fn new(card_db: &'a CardDatabase) -> Self {
        Self {
            card_db,
            logger: None,
        }
    }

    /// Enable logging with the given logger.
    pub fn with_logger(mut self, logger: ActionLogger) -> Self {
        self.logger = Some(logger);
        self
    }

    /// Run a single game between two bots.
    ///
    /// # Arguments
    /// * `bot1` - Bot playing as Player 1
    /// * `bot2` - Bot playing as Player 2
    /// * `deck1` - Deck for Player 1
    /// * `deck2` - Deck for Player 2
    /// * `seed` - Random seed for deterministic replay
    ///
    /// # Returns
    /// The result of the game including winner, turns, and timing.
    pub fn run_game(
        &mut self,
        bot1: &mut dyn Bot,
        bot2: &mut dyn Bot,
        deck1: Vec<CardId>,
        deck2: Vec<CardId>,
        seed: u64,
    ) -> GameResult {
        let start = Instant::now();
        let mut actions = Vec::new();

        // Reset bots for new game
        bot1.reset();
        bot2.reset();

        // Create and start game engine
        let mut engine = GameEngine::new(self.card_db);
        engine.start_game(deck1, deck2, seed);

        // Log game start
        if let Some(ref mut logger) = self.logger {
            let _ = logger.log_game_start(seed, bot1.name(), bot2.name());
        }

        // Main game loop
        let max_actions = 1000; // Safety limit
        let mut action_count = 0;

        while !engine.is_game_over() && action_count < max_actions {
            let action_start = Instant::now();

            // Get current state info
            let state_tensor = engine.get_state_tensor();
            let legal_mask = engine.get_legal_action_mask();
            let legal_actions = engine.get_legal_actions();
            let current_player = engine.current_player();
            let turn = engine.turn_number() as u32;

            // Select action from appropriate bot
            let action = if current_player == PlayerId::PLAYER_ONE {
                bot1.select_action(&state_tensor, &legal_mask, &legal_actions)
            } else {
                bot2.select_action(&state_tensor, &legal_mask, &legal_actions)
            };

            let thinking_time = action_start.elapsed();

            // Create action record
            let record = ActionRecord {
                turn,
                player: current_player,
                action,
                thinking_time_us: thinking_time.as_micros() as u64,
                state_snapshot: if self.logger.as_ref().map(|l| l.is_verbose()).unwrap_or(false) {
                    Some(StateSnapshot::from_state(&engine.state))
                } else {
                    None
                },
            };

            // Log action
            if let Some(ref mut logger) = self.logger {
                let _ = logger.log_action(&record);
            }
            actions.push(record);

            // Apply action
            if let Err(e) = engine.apply_action(action) {
                eprintln!("Error applying action {:?}: {:?}", action, e);
                break;
            }

            action_count += 1;
        }

        // Get final result
        let winner = engine.winner();
        let turns = engine.turn_number() as u32;
        let duration = start.elapsed();

        // Log game end
        if let Some(ref mut logger) = self.logger {
            let _ = logger.log_game_end(
                winner,
                turns,
                engine.state.players[0].life,
                engine.state.players[1].life,
            );
        }

        GameResult {
            winner,
            turns,
            seed,
            duration,
            actions,
        }
    }

    /// Run a match (multiple games) between two bots.
    ///
    /// # Arguments
    /// * `bot1` - Bot playing as Player 1
    /// * `bot2` - Bot playing as Player 2
    /// * `deck1` - Deck for Player 1
    /// * `deck2` - Deck for Player 2
    /// * `games` - Number of games to play
    /// * `base_seed` - Base seed (each game uses base_seed + game_index)
    ///
    /// # Returns
    /// Statistics for all games played.
    pub fn run_match(
        &mut self,
        bot1: &mut dyn Bot,
        bot2: &mut dyn Bot,
        deck1: Vec<CardId>,
        deck2: Vec<CardId>,
        games: usize,
        base_seed: u64,
    ) -> MatchStats {
        let mut stats = MatchStats::new(bot1.name().to_string(), bot2.name().to_string());

        for i in 0..games {
            let seed = base_seed.wrapping_add(i as u64);
            let result = self.run_game(bot1, bot2, deck1.clone(), deck2.clone(), seed);
            stats.record_game(result.winner, result.turns, result.duration);
        }

        stats
    }

    /// Run a match without a logger (for performance).
    pub fn run_match_silent(
        card_db: &CardDatabase,
        bot1: &mut dyn Bot,
        bot2: &mut dyn Bot,
        deck1: Vec<CardId>,
        deck2: Vec<CardId>,
        games: usize,
        base_seed: u64,
    ) -> MatchStats {
        let mut runner = GameRunner::new(card_db);
        runner.run_match(bot1, bot2, deck1, deck2, games, base_seed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bots::RandomBot;

    fn test_deck() -> Vec<CardId> {
        // Simple deck with starter set cards
        let valid_ids = [1, 2, 3, 4, 5, 6, 7, 8, 11, 12, 13, 15, 32, 33, 40];
        (0..20)
            .map(|i| CardId(valid_ids[i % valid_ids.len()] as u16))
            .collect()
    }

    #[test]
    fn test_run_single_game() {
        let card_db = CardDatabase::load_from_directory("data/cards/sets")
            .expect("Failed to load cards");

        let mut runner = GameRunner::new(&card_db);
        let mut bot1 = RandomBot::new(42);
        let mut bot2 = RandomBot::new(43);

        let result = runner.run_game(
            &mut bot1,
            &mut bot2,
            test_deck(),
            test_deck(),
            12345,
        );

        assert!(result.turns > 0);
        assert_eq!(result.seed, 12345);
        assert!(!result.actions.is_empty());
    }

    #[test]
    fn test_game_determinism() {
        let card_db = CardDatabase::load_from_directory("data/cards/sets")
            .expect("Failed to load cards");

        let mut runner = GameRunner::new(&card_db);

        // Run same game twice
        let mut bot1a = RandomBot::new(100);
        let mut bot2a = RandomBot::new(200);
        let result1 = runner.run_game(
            &mut bot1a,
            &mut bot2a,
            test_deck(),
            test_deck(),
            12345,
        );

        let mut bot1b = RandomBot::new(100);
        let mut bot2b = RandomBot::new(200);
        let result2 = runner.run_game(
            &mut bot1b,
            &mut bot2b,
            test_deck(),
            test_deck(),
            12345,
        );

        assert_eq!(result1.winner, result2.winner);
        assert_eq!(result1.turns, result2.turns);
        assert_eq!(result1.actions.len(), result2.actions.len());
    }

    #[test]
    fn test_run_match() {
        let card_db = CardDatabase::load_from_directory("data/cards/sets")
            .expect("Failed to load cards");

        let mut runner = GameRunner::new(&card_db);
        let mut bot1 = RandomBot::new(42);
        let mut bot2 = RandomBot::new(43);

        let stats = runner.run_match(
            &mut bot1,
            &mut bot2,
            test_deck(),
            test_deck(),
            10,
            1000,
        );

        assert_eq!(stats.overall.games, 10);
        assert_eq!(
            stats.overall.bot1_wins + stats.overall.bot2_wins + stats.overall.draws,
            10
        );
    }
}
