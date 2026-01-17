//! Game runner for executing games between bots.

use std::time::{Duration, Instant};

use crate::arena::logger::{ActionLogger, ActionRecord, StateSnapshot};
use crate::arena::stats::MatchStats;
use crate::bots::Bot;
use crate::cards::CardDatabase;
use crate::core::state::GameMode;
use crate::core::tracing::{CombatTrace, CombatTracer, EffectEvent, EffectTracer};
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
    /// Combat traces (only if tracing was enabled)
    pub combat_traces: Vec<CombatTrace>,
    /// Effect events (only if tracing was enabled)
    pub effect_events: Vec<EffectEvent>,
}

/// Runs games between bots.
pub struct GameRunner<'a> {
    card_db: &'a CardDatabase,
    logger: Option<ActionLogger>,
    trace_combat: bool,
    trace_effects: bool,
    game_mode: GameMode,
}

impl<'a> GameRunner<'a> {
    /// Create a new game runner.
    pub fn new(card_db: &'a CardDatabase) -> Self {
        Self {
            card_db,
            logger: None,
            trace_combat: false,
            trace_effects: false,
            game_mode: GameMode::default(),
        }
    }

    /// Enable logging with the given logger.
    pub fn with_logger(mut self, logger: ActionLogger) -> Self {
        self.logger = Some(logger);
        self
    }

    /// Set the game mode.
    pub fn with_game_mode(mut self, mode: GameMode) -> Self {
        self.game_mode = mode;
        self
    }

    /// Enable combat and/or effect tracing.
    ///
    /// When enabled, the `GameResult` will include detailed traces of
    /// combat resolution and effect queue processing for debugging.
    pub fn with_tracing(mut self, combat: bool, effects: bool) -> Self {
        self.trace_combat = combat;
        self.trace_effects = effects;
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

        // Create tracers if enabled
        let mut combat_tracer = CombatTracer::new(self.trace_combat);
        let mut effect_tracer = EffectTracer::new(self.trace_effects);
        let tracing_enabled = self.trace_combat || self.trace_effects;

        // Reset bots for new game
        bot1.reset();
        bot2.reset();

        // Create and start game engine
        let mut engine = GameEngine::new(self.card_db);
        engine.start_game_with_mode(deck1, deck2, seed, self.game_mode);

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
            let current_player = engine.current_player();
            let turn = engine.turn_number() as u32;

            // Select action from appropriate bot
            // Use engine-aware method to support bots like MCTS that need simulation
            let action = if current_player == PlayerId::PLAYER_ONE {
                bot1.select_action_with_engine(&engine)
            } else {
                bot2.select_action_with_engine(&engine)
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

            // Apply action - use traced version when tracing is enabled
            let result = if tracing_enabled {
                engine.apply_action_with_tracers(
                    action,
                    if self.trace_combat {
                        Some(&mut combat_tracer)
                    } else {
                        None
                    },
                    if self.trace_effects {
                        Some(&mut effect_tracer)
                    } else {
                        None
                    },
                )
            } else {
                engine.apply_action(action)
            };

            if let Err(e) = result {
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
            combat_traces: combat_tracer.traces,
            effect_events: effect_tracer.events,
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
