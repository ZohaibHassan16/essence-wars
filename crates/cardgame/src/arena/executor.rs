//! Match execution strategies for arena.
//!
//! Provides parallel and sequential execution modes using the shared
//! `execution` module infrastructure.

use std::time::{Duration, Instant};

use crate::arena::config::{MatchConfig, SequentialConfig};
use crate::arena::logger::{ActionLogger, ActionRecord, StateSnapshot};
use crate::arena::stats::MatchStats;
use crate::bots::{create_bot, BotType};
use crate::cards::CardDatabase;
use crate::core::tracing::{CombatTracer, EffectTracer};
use crate::engine::GameEngine;
use crate::execution::{run_batch_parallel, BatchConfig, GameOutcome, GameSeeds, ProgressStyle};
use crate::types::PlayerId;

/// Run a match in parallel using the shared execution infrastructure.
///
/// This is the high-performance mode for running many games quickly.
/// Does not support logging, invariant checking, or tracing.
pub fn run_match_parallel(card_db: &CardDatabase, config: &MatchConfig) -> MatchStats {
    let batch_config = if config.show_progress {
        BatchConfig::new(config.games, config.seed).with_progress(ProgressStyle::Rich)
    } else {
        BatchConfig::new(config.games, config.seed)
    };

    let result = run_batch_parallel(&batch_config, |seeds| {
        run_single_game_parallel(card_db, config, seeds)
    });

    // Convert BatchResult to MatchStats
    let mut stats = MatchStats::new(
        config.bot1_type.name().to_string(),
        config.bot2_type.name().to_string(),
    );

    for outcome in &result.outcomes {
        stats.record_game(outcome.winner, outcome.turns, outcome.duration);
    }
    stats.set_wall_clock_time(result.wall_clock_time);

    stats
}

/// Run a single game for parallel execution.
fn run_single_game_parallel(
    card_db: &CardDatabase,
    config: &MatchConfig,
    seeds: GameSeeds,
) -> GameOutcome {
    let start = Instant::now();

    // Create bots using the factory
    let mut bot1 = create_bot(
        card_db,
        &config.bot1_type,
        config.weights1.as_ref(),
        &config.mcts_config,
        seeds.bot1,
    );
    let mut bot2 = create_bot(
        card_db,
        &config.bot2_type,
        config.weights2.as_ref(),
        &config.mcts_config,
        seeds.bot2,
    );

    // Reset bots
    bot1.reset();
    bot2.reset();

    // Create and start game engine
    let mut engine = GameEngine::new(card_db);
    engine.start_game_with_mode(config.deck1.clone(), config.deck2.clone(), seeds.game, config.game_mode);

    // Main game loop
    let max_actions = 1000;
    let mut action_count = 0;

    while !engine.is_game_over() && action_count < max_actions {
        let current_player = engine.current_player();

        // Select action using appropriate bot
        let action = if current_player == PlayerId::PLAYER_ONE {
            select_action_for_bot(&mut *bot1, &config.bot1_type, &engine)
        } else {
            select_action_for_bot(&mut *bot2, &config.bot2_type, &engine)
        };

        // Apply action
        if engine.apply_action(action).is_err() {
            break;
        }

        action_count += 1;
    }

    GameOutcome::new(engine.winner(), engine.turn_number() as u32, start.elapsed())
}

/// Run a match sequentially with logging and tracing support.
///
/// This mode supports debugging features like logging, invariant checking,
/// and combat/effect tracing.
pub fn run_match_sequential(
    card_db: &CardDatabase,
    config: &MatchConfig,
    seq_config: &SequentialConfig,
    logger: &mut Option<ActionLogger>,
) -> MatchStats {
    let mut stats = MatchStats::new(
        config.bot1_type.name().to_string(),
        config.bot2_type.name().to_string(),
    );

    let start_time = Instant::now();
    let mut last_progress = 0;

    for i in 0..config.games {
        let seeds = GameSeeds::for_game(config.seed, i);

        let (winner, turns, duration) = run_single_game_sequential(
            card_db,
            config,
            seq_config,
            logger,
            seeds,
        );

        stats.record_game(winner, turns, duration);

        // Progress reporting
        if config.show_progress {
            let progress = ((i + 1) * 100) / config.games;
            if progress > last_progress {
                let elapsed = start_time.elapsed().as_secs_f64();
                let rate = (i + 1) as f64 / elapsed.max(0.001);
                let eta = if rate > 0.0 {
                    (config.games - i - 1) as f64 / rate
                } else {
                    0.0
                };
                eprint!(
                    "\rProgress: {:3}% ({}/{}) | {:.0} games/sec | ETA: {:.1}s    ",
                    progress,
                    i + 1,
                    config.games,
                    rate,
                    eta
                );
                last_progress = progress;
            }
        }
    }

    if config.show_progress {
        eprintln!(
            "\rProgress: 100% ({}/{}) | Done!                              ",
            config.games, config.games
        );
    }

    stats
}

/// Run a single game with full logging and tracing support.
fn run_single_game_sequential(
    card_db: &CardDatabase,
    config: &MatchConfig,
    seq_config: &SequentialConfig,
    logger: &mut Option<ActionLogger>,
    seeds: GameSeeds,
) -> (Option<PlayerId>, u32, Duration) {
    let start = Instant::now();

    // Create tracers if enabled
    let mut combat_tracer = CombatTracer::new(seq_config.trace_combat);
    let mut effect_tracer = EffectTracer::new(seq_config.trace_effects);
    let tracing_enabled = seq_config.tracing_enabled();

    // Create bots using the factory
    let mut bot1 = create_bot(
        card_db,
        &config.bot1_type,
        config.weights1.as_ref(),
        &config.mcts_config,
        seeds.bot1,
    );
    let mut bot2 = create_bot(
        card_db,
        &config.bot2_type,
        config.weights2.as_ref(),
        &config.mcts_config,
        seeds.bot2,
    );

    // Reset bots
    bot1.reset();
    bot2.reset();

    // Create and start game engine
    let mut engine = GameEngine::new(card_db);
    engine.start_game_with_mode(config.deck1.clone(), config.deck2.clone(), seeds.game, config.game_mode);

    // Log game start
    if let Some(ref mut l) = logger {
        let _ = l.log_game_start(seeds.game, config.bot1_type.name(), config.bot2_type.name());
    }

    // Main game loop
    let max_actions = 1000;
    let mut action_count = 0;

    while !engine.is_game_over() && action_count < max_actions {
        let action_start = Instant::now();
        let current_player = engine.current_player();
        let turn = engine.turn_number() as u32;

        // Select action using appropriate bot
        let action = if current_player == PlayerId::PLAYER_ONE {
            select_action_for_bot(&mut *bot1, &config.bot1_type, &engine)
        } else {
            select_action_for_bot(&mut *bot2, &config.bot2_type, &engine)
        };

        let thinking_time = action_start.elapsed();

        // Log action
        if let Some(ref mut l) = logger {
            let record = ActionRecord {
                turn,
                player: current_player,
                action,
                thinking_time_us: thinking_time.as_micros() as u64,
                state_snapshot: if l.is_verbose() {
                    Some(StateSnapshot::from_state(&engine.state))
                } else {
                    None
                },
            };
            let _ = l.log_action(&record);
        }

        // Apply action - use traced version when tracing is enabled
        let result = if tracing_enabled {
            engine.apply_action_with_tracers(
                action,
                if seq_config.trace_combat {
                    Some(&mut combat_tracer)
                } else {
                    None
                },
                if seq_config.trace_effects {
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

        // Check invariants if enabled
        if seq_config.check_invariants {
            verify_invariants(&engine, seeds.game, action_count);
        }

        action_count += 1;
    }

    // Log game end
    if let Some(ref mut l) = logger {
        let _ = l.log_game_end(
            engine.winner(),
            engine.turn_number() as u32,
            engine.state.players[0].life,
            engine.state.players[1].life,
        );
    }

    (
        engine.winner(),
        engine.turn_number() as u32,
        start.elapsed(),
    )
}

/// Select action based on bot type.
///
/// Uses the appropriate method depending on whether the bot needs engine access.
fn select_action_for_bot(
    bot: &mut dyn crate::bots::Bot,
    bot_type: &BotType,
    engine: &GameEngine,
) -> crate::actions::Action {
    match bot_type {
        BotType::Random => {
            let state_tensor = engine.get_state_tensor();
            let legal_mask = engine.get_legal_action_mask();
            let legal_actions = engine.get_legal_actions();
            bot.select_action(&state_tensor, &legal_mask, &legal_actions)
        }
        _ => bot.select_action_with_engine(engine),
    }
}

/// Verify game state invariants for debugging.
fn verify_invariants(engine: &GameEngine, seed: u64, action_count: usize) {
    let state = &engine.state;
    let context = format!("seed={}, action={}", seed, action_count);

    // Basic life checks
    for (i, player) in state.players.iter().enumerate() {
        let player_name = if i == 0 { "P1" } else { "P2" };

        // Creatures should have valid health
        for (slot_idx, creature) in player.creatures.iter().enumerate() {
            assert!(
                creature.current_health > 0,
                "[{}] {} creature in slot {} has non-positive health: {}",
                context,
                player_name,
                slot_idx,
                creature.current_health
            );
        }

        // Hand size should be reasonable
        assert!(
            player.hand.len() <= 20,
            "[{}] {} hand size {} exceeds max 20",
            context,
            player_name,
            player.hand.len()
        );
    }

    // If game is over, result should be set
    if state.players[0].life <= 0 || state.players[1].life <= 0 {
        assert!(
            state.result.is_some(),
            "[{}] Player dead but game result not set (P1: {}, P2: {})",
            context,
            state.players[0].life,
            state.players[1].life
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bots::BotType;
    use crate::types::CardId;

    fn test_card_db() -> CardDatabase {
        let cards_path = crate::data_dir().join("cards/core_set");
        CardDatabase::load_from_directory(cards_path).unwrap()
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
    fn test_parallel_execution() {
        let card_db = test_card_db();
        let config = MatchConfig::new(
            BotType::Random,
            BotType::Random,
            test_deck(),
            test_deck(),
            5,
            42,
        );

        let stats = run_match_parallel(&card_db, &config);
        assert_eq!(stats.overall.games, 5);
    }

    #[test]
    fn test_sequential_execution() {
        let card_db = test_card_db();
        let config = MatchConfig::new(
            BotType::Random,
            BotType::Random,
            test_deck(),
            test_deck(),
            3,
            42,
        );
        let seq_config = SequentialConfig::new();
        let mut logger = None;

        let stats = run_match_sequential(&card_db, &config, &seq_config, &mut logger);
        assert_eq!(stats.overall.games, 3);
    }
}
