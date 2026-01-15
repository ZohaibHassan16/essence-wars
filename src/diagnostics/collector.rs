//! Diagnostic data collection.
//!
//! Captures per-turn snapshots and game events for P1/P2 asymmetry analysis.

use crate::bots::{create_bot, BotType, MctsConfig};
use crate::cards::CardDatabase;
use crate::engine::GameEngine;
use crate::execution::GameSeeds;
use crate::types::{CardId, PlayerId};

/// Snapshot of game state at a specific point.
#[derive(Clone, Debug)]
pub struct TurnSnapshot {
    /// Current turn number.
    pub turn: u32,
    /// Which player is active.
    pub active_player: PlayerId,
    /// Player 1 life total.
    pub p1_life: i32,
    /// Player 2 life total.
    pub p2_life: i32,
    /// Number of creatures Player 1 has.
    pub p1_creatures: usize,
    /// Number of creatures Player 2 has.
    pub p2_creatures: usize,
    /// Total attack power on Player 1's board.
    pub p1_total_attack: i32,
    /// Total attack power on Player 2's board.
    pub p2_total_attack: i32,
    /// Total health on Player 1's board.
    pub p1_total_health: i32,
    /// Total health on Player 2's board.
    pub p2_total_health: i32,
    /// Cards in Player 1's hand.
    pub p1_hand_size: usize,
    /// Cards in Player 2's hand.
    pub p2_hand_size: usize,
    /// Player 1's current essence.
    pub p1_essence: u8,
    /// Player 2's current essence.
    pub p2_essence: u8,
    /// Player 1's maximum essence.
    pub p1_max_essence: u8,
    /// Player 2's maximum essence.
    pub p2_max_essence: u8,
    /// Player 1's action points.
    pub p1_action_points: u8,
    /// Player 2's action points.
    pub p2_action_points: u8,
}

impl TurnSnapshot {
    /// Capture a snapshot from the current engine state.
    pub fn capture(engine: &GameEngine) -> Self {
        let state = &engine.state;
        let p1 = &state.players[0];
        let p2 = &state.players[1];

        Self {
            turn: engine.turn_number() as u32,
            active_player: state.active_player,
            p1_life: p1.life as i32,
            p2_life: p2.life as i32,
            p1_creatures: p1.creatures.len(),
            p2_creatures: p2.creatures.len(),
            p1_total_attack: p1.creatures.iter().map(|c| c.attack as i32).sum(),
            p2_total_attack: p2.creatures.iter().map(|c| c.attack as i32).sum(),
            p1_total_health: p1.creatures.iter().map(|c| c.current_health as i32).sum(),
            p2_total_health: p2.creatures.iter().map(|c| c.current_health as i32).sum(),
            p1_hand_size: p1.hand.len(),
            p2_hand_size: p2.hand.len(),
            p1_essence: p1.current_essence,
            p2_essence: p2.current_essence,
            p1_max_essence: p1.max_essence,
            p2_max_essence: p2.max_essence,
            p1_action_points: p1.action_points,
            p2_action_points: p2.action_points,
        }
    }
}

/// Statistics for a single game.
#[derive(Clone, Debug)]
pub struct GameDiagnostics {
    /// Random seed used for this game.
    pub seed: u64,
    /// Winner of the game (None = draw).
    pub winner: Option<PlayerId>,
    /// Total turns played.
    pub total_turns: u32,
    /// Per-turn snapshots.
    pub snapshots: Vec<TurnSnapshot>,
    /// Turn when Player 1 first took damage.
    pub first_damage_to_p1_turn: Option<u32>,
    /// Turn when Player 2 first took damage.
    pub first_damage_to_p2_turn: Option<u32>,
    /// Turn when the first creature died.
    pub first_creature_death_turn: Option<u32>,
    /// Total actions taken by Player 1.
    pub p1_actions: usize,
    /// Total actions taken by Player 2.
    pub p2_actions: usize,
}

/// Configuration for diagnostic runs.
#[derive(Clone, Debug)]
pub struct DiagnosticConfig {
    /// Number of games to run.
    pub num_games: usize,
    /// Deck for Player 1.
    pub deck1: Vec<CardId>,
    /// Deck for Player 2.
    pub deck2: Vec<CardId>,
    /// Bot type for Player 1.
    pub bot1_type: BotType,
    /// Bot type for Player 2.
    pub bot2_type: BotType,
    /// Base seed for reproducibility.
    pub base_seed: u64,
    /// Whether to show progress.
    pub show_progress: bool,
    /// MCTS configuration (if using MCTS bots).
    pub mcts_config: MctsConfig,
}

impl DiagnosticConfig {
    /// Create a new diagnostic config with greedy bots.
    pub fn new(deck: Vec<CardId>, num_games: usize) -> Self {
        Self {
            num_games,
            deck1: deck.clone(),
            deck2: deck,
            bot1_type: BotType::Greedy,
            bot2_type: BotType::Greedy,
            base_seed: 42,
            show_progress: false,
            mcts_config: MctsConfig::default(),
        }
    }

    /// Set decks for both players.
    pub fn with_decks(mut self, deck1: Vec<CardId>, deck2: Vec<CardId>) -> Self {
        self.deck1 = deck1;
        self.deck2 = deck2;
        self
    }

    /// Set bot types.
    pub fn with_bots(mut self, bot1: BotType, bot2: BotType) -> Self {
        self.bot1_type = bot1;
        self.bot2_type = bot2;
        self
    }

    /// Set base seed.
    pub fn with_seed(mut self, seed: u64) -> Self {
        self.base_seed = seed;
        self
    }

    /// Enable progress display.
    pub fn with_progress(mut self, show: bool) -> Self {
        self.show_progress = show;
        self
    }
}

/// Runner for executing diagnostic games.
pub struct DiagnosticRunner<'a> {
    card_db: &'a CardDatabase,
}

impl<'a> DiagnosticRunner<'a> {
    /// Create a new diagnostic runner.
    pub fn new(card_db: &'a CardDatabase) -> Self {
        Self { card_db }
    }

    /// Run diagnostic games and collect data.
    ///
    /// Note: This currently runs sequentially to collect full diagnostic data.
    /// The parallel execution infrastructure (BatchConfig, run_batch_parallel) is
    /// available but returns only GameOutcome, not the full diagnostics we need.
    /// TODO: Optimize with parallel collection if performance is an issue.
    pub fn run(&self, config: &DiagnosticConfig) -> Vec<GameDiagnostics> {
        self.run_sequential(config)
    }

    /// Run games sequentially to collect full diagnostic data.
    fn run_sequential(&self, config: &DiagnosticConfig) -> Vec<GameDiagnostics> {
        let mut results = Vec::with_capacity(config.num_games);

        for i in 0..config.num_games {
            if config.show_progress && i % 50 == 0 {
                eprint!("\rProgress: {}/{}", i, config.num_games);
            }

            let seeds = GameSeeds::for_game(config.base_seed, i);
            let diag = self.run_diagnostic_game(config, seeds);
            results.push(diag);
        }

        if config.show_progress {
            eprintln!("\rProgress: {}/{}", config.num_games, config.num_games);
        }

        results
    }

    /// Run a single diagnostic game with full data collection.
    fn run_diagnostic_game(&self, config: &DiagnosticConfig, seeds: GameSeeds) -> GameDiagnostics {
        let mut bot1 = create_bot(
            self.card_db,
            &config.bot1_type,
            None,
            &config.mcts_config,
            seeds.bot1,
        );
        let mut bot2 = create_bot(
            self.card_db,
            &config.bot2_type,
            None,
            &config.mcts_config,
            seeds.bot2,
        );

        bot1.reset();
        bot2.reset();

        let mut engine = GameEngine::new(self.card_db);
        engine.start_game(config.deck1.clone(), config.deck2.clone(), seeds.game);

        let mut snapshots = Vec::new();
        let mut first_damage_to_p1_turn = None;
        let mut first_damage_to_p2_turn = None;
        let mut first_creature_death_turn = None;
        let mut p1_actions = 0;
        let mut p2_actions = 0;

        let initial_p1_life = engine.state.players[0].life as i32;
        let initial_p2_life = engine.state.players[1].life as i32;
        let mut last_turn = 0;

        // Capture initial state
        snapshots.push(TurnSnapshot::capture(&engine));

        let max_actions = 1000;
        let mut action_count = 0;

        while !engine.is_game_over() && action_count < max_actions {
            let current_player = engine.current_player();
            let current_turn = engine.turn_number() as u32;

            // Capture state at start of new turn
            if current_turn != last_turn {
                snapshots.push(TurnSnapshot::capture(&engine));
                last_turn = current_turn;
            }

            // Track creature count before action
            let p1_creatures_before = engine.state.players[0].creatures.len();
            let p2_creatures_before = engine.state.players[1].creatures.len();

            // Select and apply action
            let action = if current_player == PlayerId::PLAYER_ONE {
                p1_actions += 1;
                bot1.select_action_with_engine(&engine)
            } else {
                p2_actions += 1;
                bot2.select_action_with_engine(&engine)
            };

            if engine.apply_action(action).is_err() {
                break;
            }

            // Check for first damage
            if first_damage_to_p1_turn.is_none()
                && (engine.state.players[0].life as i32) < initial_p1_life
            {
                first_damage_to_p1_turn = Some(current_turn);
            }
            if first_damage_to_p2_turn.is_none()
                && (engine.state.players[1].life as i32) < initial_p2_life
            {
                first_damage_to_p2_turn = Some(current_turn);
            }

            // Check for first creature death
            if first_creature_death_turn.is_none() {
                let p1_creatures_after = engine.state.players[0].creatures.len();
                let p2_creatures_after = engine.state.players[1].creatures.len();
                if p1_creatures_after < p1_creatures_before
                    || p2_creatures_after < p2_creatures_before
                {
                    first_creature_death_turn = Some(current_turn);
                }
            }

            action_count += 1;
        }

        // Final snapshot
        snapshots.push(TurnSnapshot::capture(&engine));

        GameDiagnostics {
            seed: seeds.game,
            winner: engine.winner(),
            total_turns: engine.turn_number() as u32,
            snapshots,
            first_damage_to_p1_turn,
            first_damage_to_p2_turn,
            first_creature_death_turn,
            p1_actions,
            p2_actions,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_card_db() -> CardDatabase {
        CardDatabase::load_from_directory("data/cards/core_set").unwrap()
    }

    fn test_deck() -> Vec<CardId> {
        vec![
            CardId(1000),
            CardId(1000),
            CardId(1001),
            CardId(1001),
            CardId(1002),
            CardId(1002),
            CardId(1003),
            CardId(1003),
            CardId(1004),
            CardId(1004),
        ]
    }

    #[test]
    fn test_turn_snapshot_capture() {
        let card_db = test_card_db();
        let mut engine = GameEngine::new(&card_db);
        engine.start_game(test_deck(), test_deck(), 42);

        let snapshot = TurnSnapshot::capture(&engine);

        assert_eq!(snapshot.turn, 1);
        assert_eq!(snapshot.p1_life, 30);
        assert_eq!(snapshot.p2_life, 30);
    }

    #[test]
    fn test_diagnostic_runner() {
        let card_db = test_card_db();
        let config = DiagnosticConfig::new(test_deck(), 3).with_seed(42);

        let runner = DiagnosticRunner::new(&card_db);
        let results = runner.run(&config);

        assert_eq!(results.len(), 3);
        for diag in &results {
            assert!(!diag.snapshots.is_empty());
            assert!(diag.total_turns > 0);
        }
    }
}
